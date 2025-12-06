//! Gradle core execution engine
//! 
//! Implements task execution and build lifecycle for Gradle projects.

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::build::{BuildExecutor, BuildSystem, ExecutionRequest, ExecutionResult};
use crate::gradle::model::{GradleProject, Task};
use crate::gradle::parse_gradle_build_script;

/// Gradle executor implementation
pub struct GradleExecutor;

impl GradleExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Load a Gradle project from a directory
    pub fn load_project(&self, base_dir: &PathBuf) -> Result<GradleProject> {
        // Find build.gradle or build.gradle.kts
        let build_file_gradle = base_dir.join("build.gradle");
        let build_file_kts = base_dir.join("build.gradle.kts");
        
        let build_file = if build_file_gradle.exists() {
            build_file_gradle
        } else if build_file_kts.exists() {
            build_file_kts
        } else {
            return Err(anyhow::anyhow!("No build.gradle or build.gradle.kts found in {:?}", base_dir));
        };

        parse_gradle_build_script(&build_file, base_dir)
            .with_context(|| format!("Failed to parse Gradle build script: {:?}", build_file))
    }

    /// Execute a task
    fn execute_task(&self, project: &GradleProject, task_name: &str) -> Result<()> {
        let task = project.find_task(task_name)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_name))?;

        // Execute task dependencies first
        for dep_name in &task.depends_on {
            self.execute_task(project, dep_name)?;
        }

        // Execute the task itself
        match task.name.as_str() {
            "clean" => self.execute_clean_task(project)?,
            "compileJava" => self.execute_compile_java_task(project)?,
            "test" => self.execute_test_task(project)?,
            "jar" => self.execute_jar_task(project)?,
            "build" => self.execute_build_task(project)?,
            _ => {
                // For custom tasks, we'd need to execute the task actions
                // For now, just log that we're executing it
                tracing::info!("Executing task: {}", task_name);
            }
        }

        Ok(())
    }

    /// Execute clean task
    fn execute_clean_task(&self, project: &GradleProject) -> Result<()> {
        let build_dir = project.base_dir.join("build");
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir)
                .with_context(|| format!("Failed to remove build directory: {:?}", build_dir))?;
            tracing::info!("Cleaned build directory");
        }
        Ok(())
    }

    /// Execute compile Java task
    fn execute_compile_java_task(&self, project: &GradleProject) -> Result<()> {
        // Use the shared Java compiler
        use crate::compiler::java_compiler::{JavaCompiler, CompilerConfig};
        use crate::compiler::classpath::ClasspathBuilder;
        
        let source_dir = project.base_dir.join("src/main/java");
        let output_dir = project.base_dir.join("build/classes/java/main");
        
        std::fs::create_dir_all(&output_dir)?;
        
        let mut classpath = ClasspathBuilder::new();
        // Add dependencies to classpath using shared resolver
        for dep in &project.dependencies {
            if dep.configuration == "implementation" || dep.configuration == "compile" {
                // Resolve dependency using shared resolver
                if let (Some(group), Some(artifact)) = (&dep.group, &dep.artifact) {
                    if let Ok(resolved) = self.resolve_dependency(group, artifact, dep.version.as_deref()) {
                        if let Some(jar_path) = resolved {
                            classpath = classpath.add_jar(jar_path);
                        }
                    }
                }
            }
        }
        
        // Build compiler config
        let mut config = CompilerConfig::new(output_dir.clone())
            .with_source_roots(vec![source_dir])
            .with_classpath(classpath);
        
        // Set source/target compatibility if specified
        if let Some(ref source_compat) = project.source_compatibility {
            config = config.with_source_version(source_compat.clone());
        }
        if let Some(ref target_compat) = project.target_compatibility {
            config = config.with_target_version(target_compat.clone());
        }
        
        let result = JavaCompiler::compile(&config)?;
        
        if !result.success {
            return Err(anyhow::anyhow!("Compilation failed: {}", result.error_output));
        }
        
        tracing::info!("Compiled {} Java source file(s)", result.compiled_files);
        Ok(())
    }

    /// Execute test task
    fn execute_test_task(&self, project: &GradleProject) -> Result<()> {
        // First compile test sources
        self.execute_compile_test_java_task(project)?;
        
        // Then run tests using shared test runner
        use crate::testing::runner::TestRunner;
        use crate::testing::discovery::TestDiscovery;
        
        let test_classes_dir = project.base_dir.join("build/classes/java/test");
        let main_classes_dir = project.base_dir.join("build/classes/java/main");
        
        // Discover test classes
        let test_classes = TestDiscovery::discover_tests(&test_classes_dir)?;
        
        if test_classes.is_empty() {
            tracing::info!("No test classes found");
            return Ok(());
        }
        
        // Build test runner with classpath
        let mut runner = TestRunner::new(test_classes_dir.clone());
        runner = runner.add_to_classpath(main_classes_dir);
        // Add test dependencies to classpath (would use dependency resolver)
        
        // Run each test class
        let mut all_passed = true;
        for test_class in &test_classes {
            let result = runner.run_test(test_class)?;
            if !result.success {
                all_passed = false;
                tracing::error!("Test {} failed: {}", test_class.class_name, result.error_output);
            } else {
                tracing::info!(
                    "Test {} passed: {}/{} tests",
                    test_class.class_name,
                    result.tests_passed,
                    result.tests_run
                );
            }
        }
        
        if !all_passed {
            return Err(anyhow::anyhow!("Some tests failed"));
        }
        
        tracing::info!("All tests passed");
        Ok(())
    }

    /// Execute compile test Java task
    fn execute_compile_test_java_task(&self, project: &GradleProject) -> Result<()> {
        use crate::compiler::java_compiler::{JavaCompiler, CompilerConfig};
        use crate::compiler::classpath::ClasspathBuilder;
        
        let source_dir = project.base_dir.join("src/test/java");
        let output_dir = project.base_dir.join("build/classes/java/test");
        let main_classes_dir = project.base_dir.join("build/classes/java/main");
        
        std::fs::create_dir_all(&output_dir)?;
        
        let mut classpath = ClasspathBuilder::new();
        // Add main classes to classpath
        if main_classes_dir.exists() {
            classpath = classpath.add_directory(main_classes_dir);
        }
        // Add test dependencies (would use dependency resolver)
        
        // Build compiler config
        let mut config = CompilerConfig::new(output_dir.clone())
            .with_source_roots(vec![source_dir])
            .with_classpath(classpath);
        
        // Set source/target compatibility if specified
        if let Some(ref source_compat) = project.source_compatibility {
            config = config.with_source_version(source_compat.clone());
        }
        if let Some(ref target_compat) = project.target_compatibility {
            config = config.with_target_version(target_compat.clone());
        }
        
        let result = JavaCompiler::compile_tests(&config)?;
        
        if !result.success {
            return Err(anyhow::anyhow!("Test compilation failed: {}", result.error_output));
        }
        
        tracing::info!("Compiled {} test Java source file(s)", result.compiled_files);
        Ok(())
    }

    /// Execute jar task
    fn execute_jar_task(&self, project: &GradleProject) -> Result<()> {
        // Use shared JAR packaging
        use crate::packaging::jar::JarBuilder;
        use crate::packaging::resources::ResourceFilter;
        
        let classes_dir = project.base_dir.join("build/classes/java/main");
        let resources_dir = project.base_dir.join("src/main/resources");
        let output_jar = project.base_dir.join(format!("build/libs/{}-{}.jar",
            project.name,
            project.version.as_deref().unwrap_or("unspecified")
        ));
        
        std::fs::create_dir_all(output_jar.parent().unwrap())?;
        
        let mut jar_builder = JarBuilder::new()
            .with_classes_dir(classes_dir);
        
        // Add resources if they exist
        if resources_dir.exists() {
            jar_builder = jar_builder.add_resources_from_dir(
                resources_dir,
                None,
                ResourceFilter::default(),
            )?;
        }
        
        jar_builder.build(&output_jar)?;
        
        tracing::info!("Created JAR: {:?}", output_jar);
        Ok(())
    }

    /// Execute build task
    fn execute_build_task(&self, project: &GradleProject) -> Result<()> {
        // Build typically means: compile, test, jar
        self.execute_compile_java_task(project)?;
        self.execute_test_task(project)?;
        self.execute_jar_task(project)?;
        Ok(())
    }

    /// Resolve a dependency to a JAR file path
    fn resolve_dependency(
        &self,
        group: &str,
        artifact: &str,
        version: Option<&str>,
    ) -> Result<Option<PathBuf>> {
        use crate::resolver::resolver::DependencyResolver;
        use crate::artifact::artifact::Artifact;
        use crate::artifact::repository::{LocalRepository, DefaultLocalRepository};
        
        // Create artifact
        let version = version.ok_or_else(|| {
            anyhow::anyhow!("Dependency {group}:{artifact} has no version specified")
        })?;
        
        let artifact = Artifact::new(group, artifact, version);
        
        // Use default local repository
        let local_repo = DefaultLocalRepository::default();
        
        // Check local repository first
        if local_repo.artifact_exists(&artifact) {
            return Ok(Some(local_repo.artifact_path(&artifact)));
        }
        
        // Try to resolve using dependency resolver (will download if needed)
        let local_repo_box: Box<dyn LocalRepository> = Box::new(DefaultLocalRepository::default());
        let resolver = DependencyResolver::new(local_repo_box);
        
        // Convert Gradle dependency to Maven Dependency format
        use crate::model::Dependency as MavenDependency;
        let maven_dep = MavenDependency {
            group_id: group.to_string(),
            artifact_id: artifact.to_string(),
            version: Some(version.to_string()),
            type_: Some("jar".to_string()),
            classifier: None,
            scope: Some("compile".to_string()),
            optional: Some(false),
            exclusions: None,
        };
        
        // Resolve dependency
        if let Ok(Some(resolved_artifact)) = resolver.resolve_dependency(&maven_dep) {
            if let Some(file_path) = resolved_artifact.file {
                return Ok(Some(file_path));
            }
        }
        
        tracing::warn!(
            "Dependency {group}:{artifact}:{version} could not be resolved"
        );
        
        Ok(None)
    }
}

impl BuildExecutor for GradleExecutor {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        // Load the Gradle project
        let project = self.load_project(&request.base_directory)?;

        let mut errors = Vec::new();
        let mut success = true;

        // Execute requested tasks
        for goal in &request.goals {
            match self.execute_task(&project, goal) {
                Ok(()) => {
                    tracing::info!("Successfully executed task: {}", goal);
                }
                Err(e) => {
                    let error_msg = format!("Failed to execute task '{}': {}", goal, e);
                    errors.push(error_msg.clone());
                    tracing::error!("{}", error_msg);
                    success = false;
                    
                    if request.show_errors {
                        eprintln!("[ERROR] {}", error_msg);
                    }
                }
            }
        }

        Ok(ExecutionResult {
            success,
            errors,
        })
    }

    fn build_system(&self) -> BuildSystem {
        BuildSystem::Gradle
    }
}
