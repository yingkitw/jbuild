use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

use crate::plugin_api::{Plugin, PluginDescriptor};
use crate::artifact::{Artifact, LocalRepository};
use crate::resolver::{resolver::DependencyResolver, repository::RemoteRepository, downloader::ArtifactDownloader};
use crate::model::Dependency;
use crate::model::parser::parse_pom;

/// Plugin registry - manages loaded plugins
pub struct PluginRegistry {
    plugins: Arc<Mutex<HashMap<String, Arc<dyn Plugin>>>>,
    local_repository: Box<dyn LocalRepository>,
    resolver: DependencyResolver,
    downloader: ArtifactDownloader,
    remote_repositories: Vec<RemoteRepository>,
}

impl PluginRegistry {
    pub fn new(local_repository: Box<dyn LocalRepository>, resolver: DependencyResolver) -> Self {
        let remote_repos = resolver.remote_repositories().to_vec();
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            local_repository,
            resolver,
            downloader: ArtifactDownloader::new(),
            remote_repositories: remote_repos,
        }
    }

    pub fn with_remote_repositories(mut self, repositories: Vec<RemoteRepository>) -> Self {
        self.remote_repositories = repositories;
        self
    }

    /// Get a plugin by coordinates
    pub fn get_plugin(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Option<Arc<dyn Plugin>>> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        
        // Check cache first
        {
            let plugins = self.plugins.lock().unwrap();
            if let Some(plugin) = plugins.get(&key) {
                return Ok(Some(plugin.clone()));
            }
        }

        // Load plugin
        let plugin = self.load_plugin(group_id, artifact_id, version)?;
        
        if let Some(ref plugin) = plugin {
            // Cache the plugin
            let mut plugins = self.plugins.lock().unwrap();
            plugins.insert(key, plugin.clone());
        }
        
        Ok(plugin)
    }

    /// Load a plugin from repository
    fn load_plugin(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Option<Arc<dyn Plugin>>> {
        let dependency = Dependency {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: Some(version.to_string()),
            type_: None,
            classifier: None,
            scope: Some("compile".to_string()),
            optional: Some(false),
            exclusions: None,
        };

        // Resolve plugin artifact
        let artifact = self.resolver.resolve_dependency(&dependency)
            .context("Failed to resolve plugin artifact")?;

        let artifact = artifact.ok_or_else(|| {
            anyhow::anyhow!("Plugin {}:{}:{} not found", group_id, artifact_id, version)
        })?;

        let jar_path = artifact.file.ok_or_else(|| {
            anyhow::anyhow!("Plugin artifact has no file path")
        })?;

        // Parse plugin descriptor
        let mut descriptor = PluginDescriptor::from_jar(&jar_path)
            .with_context(|| format!("Failed to parse plugin descriptor from {:?}", jar_path))?;

        // Resolve plugin dependencies from plugin POM
        let plugin_dependencies = self.resolve_plugin_dependencies(group_id, artifact_id, version)?;
        descriptor.dependencies = plugin_dependencies;

        // Resolve dependency artifacts
        let dependency_artifacts = self.resolver.resolve_dependencies(&descriptor.dependencies)
            .context("Failed to resolve plugin dependencies")?;

        // Create plugin instance with dependencies
        let plugin: Arc<dyn Plugin> = Arc::new(LoadedPlugin {
            descriptor,
            jar_path,
            dependency_artifacts,
        });

        Ok(Some(plugin))
    }

    /// Resolve plugin dependencies by parsing the plugin's POM
    fn resolve_plugin_dependencies(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Vec<Dependency>> {
        // Create plugin artifact for POM
        let plugin_artifact = Artifact::new(group_id, artifact_id, version);
        
        // Check if POM exists locally
        let mut pom_artifact = plugin_artifact.clone();
        pom_artifact.coordinates.packaging = Some("pom".to_string());
        
        let pom_path = self.local_repository.artifact_path(&pom_artifact);
        
        // Download POM if not exists
        if !pom_path.exists() {
            let pom_path_parent = pom_path.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid POM path"))?;
            std::fs::create_dir_all(pom_path_parent)
                .context("Failed to create POM directory")?;
            
            self.downloader.download_pom(&plugin_artifact, &self.remote_repositories[0], &pom_path)
                .context("Failed to download plugin POM")?;
        }

        // Parse POM to get dependencies
        let model = parse_pom(&std::fs::read_to_string(&pom_path)
            .context("Failed to read plugin POM")?)
            .map_err(|e| anyhow::anyhow!("Failed to parse plugin POM: {}", e))?;

        // Extract dependencies from build section or root
        let mut dependencies = Vec::new();
        
        if let Some(ref build) = model.build {
            if let Some(ref plugins) = build.plugins {
                // Look for this plugin in the plugins list
                for plugin in plugins {
                    if plugin.artifact_id == artifact_id {
                        if let Some(ref deps) = plugin.dependencies {
                            dependencies.extend(deps.clone());
                        }
                    }
                }
            }
        }

        // Also check root-level dependencies
        dependencies.extend(model.dependencies_vec());

        Ok(dependencies)
    }
}

/// Concrete plugin implementation loaded from a JAR
struct LoadedPlugin {
    descriptor: PluginDescriptor,
    jar_path: PathBuf,
    dependency_artifacts: Vec<Artifact>,
}

impl LoadedPlugin {
    /// Get the classpath for this plugin (plugin JAR + dependencies)
    pub fn classpath(&self) -> String {
        use crate::compiler::ClasspathBuilder;
        
        let mut builder = ClasspathBuilder::new();
        builder = builder.add_jar(self.jar_path.clone());
        
        for dep_artifact in &self.dependency_artifacts {
            if let Some(ref file) = dep_artifact.file {
                builder = builder.add_jar(file.clone());
            }
        }
        
        builder.build()
    }
}

impl Plugin for LoadedPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn get_mojo(&self, goal: &str) -> Option<Box<dyn crate::plugin_api::Mojo>> {
        // Find the goal in the descriptor
        let goal_info = self.descriptor.goals.iter()
            .find(|g| g.name == goal)?;

        // Return a Java mojo executor that can run the plugin via external Java process
        Some(Box::new(JavaMojo {
            plugin_jar: self.jar_path.clone(),
            classpath: self.classpath(),
            goal: goal.to_string(),
            phase: goal_info.phase.clone(),
            description: goal_info.description.clone(),
            plugin_group_id: self.descriptor.group_id.clone(),
            plugin_artifact_id: self.descriptor.artifact_id.clone(),
            plugin_version: self.descriptor.version.clone(),
        }))
    }
}

/// Java mojo implementation that executes plugins via external Java process
struct JavaMojo {
    plugin_jar: PathBuf,
    classpath: String,
    goal: String,
    phase: Option<String>,
    description: Option<String>,
    plugin_group_id: String,
    plugin_artifact_id: String,
    plugin_version: String,
}

impl crate::plugin_api::Mojo for JavaMojo {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "Executing Java mojo {}:{}:{}:{} (phase: {:?})",
            self.plugin_group_id,
            self.plugin_artifact_id,
            self.plugin_version,
            self.goal,
            self.phase
        );

        // Try JNI execution first (if available)
        #[cfg(feature = "jni")]
        {
            if let Ok(jni_executor) = crate::plugin_api::jni_executor::JniMojoExecutor::new(
                self.plugin_jar.clone(),
                self.classpath.clone(),
            ) {
                // Try to get Mojo class name from plugin descriptor
                // For now, use a generic approach
                if let Err(e) = jni_executor.execute_mojo("org.apache.maven.plugin.Mojo", &serde_json::json!({})) {
                    tracing::warn!("JNI execution failed, falling back to external process: {}", e);
                } else {
                    return Ok(());
                }
            }
        }

        // Fall back to external Maven process
        use crate::plugin_api::external_executor::ExternalMavenExecutor;
        let executor = ExternalMavenExecutor;
        
        // Get current working directory (project directory)
        let project_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        executor.execute_plugin_goal(
            &self.plugin_group_id,
            &self.plugin_artifact_id,
            &self.plugin_version,
            &self.goal,
            &project_dir,
        )?;

        Ok(())
    }
}

impl JavaMojo {
    /// Find Java executable
    fn find_java() -> Option<PathBuf> {
        // Try JAVA_HOME first
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let java_path = PathBuf::from(java_home)
                .join("bin")
                .join(if cfg!(windows) { "java.exe" } else { "java" });
            
            if java_path.exists() {
                return Some(java_path);
            }
        }

        // Try to find java in PATH
        which::which("java").ok()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        use crate::artifact::repository::DefaultLocalRepository;
        let local_repo = Box::new(DefaultLocalRepository::default());
        let local_repo_for_resolver: Box<dyn LocalRepository> = Box::new(DefaultLocalRepository::default());
        let resolver = DependencyResolver::new(local_repo_for_resolver);
        Self::new(local_repo, resolver)
    }
}

