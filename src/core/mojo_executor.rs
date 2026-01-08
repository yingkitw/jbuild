use anyhow::{Context, Result};

use crate::core::session::MavenSession;
use crate::plugin_api::registry::PluginRegistry;

/// Mojo execution request
#[derive(Debug, Clone)]
pub struct MojoExecution {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub goal: String,
    pub phase: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

/// Mojo executor - executes plugin mojos
pub struct MojoExecutor {
    plugin_registry: Option<PluginRegistry>,
}

impl MojoExecutor {
    pub fn new() -> Self {
        Self {
            plugin_registry: None,
        }
    }

    pub fn with_registry(mut self, registry: PluginRegistry) -> Self {
        self.plugin_registry = Some(registry);
        self
    }

    /// Execute a list of mojo executions
    pub fn execute(
        &self,
        session: &MavenSession,
        executions: &[MojoExecution],
    ) -> Result<()> {
        for execution in executions {
            if let Err(e) = self.execute_mojo(session, execution) {
                return Err(anyhow::anyhow!(
                    "Failed to execute mojo {}:{}:{} - {}",
                    execution.group_id,
                    execution.artifact_id,
                    execution.goal,
                    e
                ));
            }
        }
        Ok(())
    }

    fn execute_mojo(&self, session: &MavenSession, execution: &MojoExecution) -> Result<()> {
        let registry = self.plugin_registry.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Plugin registry not configured"))?;

        // Resolve plugin version if not provided
        let version = if let Some(ref v) = execution.version {
            v.clone()
        } else {
            self.resolve_plugin_version(&execution.group_id, &execution.artifact_id, session)?
                .unwrap_or_else(|| "LATEST".to_string())
        };

        // Load plugin
        let plugin = registry.get_plugin(&execution.group_id, &execution.artifact_id, &version)
            .with_context(|| format!(
                "Failed to load plugin {}:{}:{}",
                execution.group_id, execution.artifact_id, version
            ))?;

        let plugin = plugin.ok_or_else(|| {
            anyhow::anyhow!(
                "Plugin {}:{}:{} not found",
                execution.group_id, execution.artifact_id, version
            )
        })?;

        // Get mojo
        let mojo = plugin.get_mojo(&execution.goal)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Goal '{}' not found in plugin {}:{}:{}",
                    execution.goal, execution.group_id, execution.artifact_id, version
                )
            })?;

        tracing::info!(
            "Executing mojo {}:{}:{}:{}",
            execution.group_id,
            execution.artifact_id,
            version,
            execution.goal
        );

        // Execute mojo
        mojo.execute()
            .map_err(|e| anyhow::anyhow!("Mojo execution failed: {e}"))?;

        Ok(())
    }

    /// Resolve plugin version from metadata or use defaults
    fn resolve_plugin_version(
        &self,
        group_id: &str,
        artifact_id: &str,
        session: &MavenSession,
    ) -> Result<Option<String>> {
        // 1. Project's pluginManagement
        if let Some(project) = &session.current_project {
            if let Some(build) = &project.model.build {
                if let Some(plugin_mgmt) = &build.plugin_management {
                    if let Some(plugins) = &plugin_mgmt.plugins {
                        for plugin in plugins {
                            if plugin.artifact_id == artifact_id {
                                if let Some(gid) = &plugin.group_id {
                                    if gid == group_id {
                                        if let Some(version) = &plugin.version {
                                            return Ok(Some(version.clone()));
                                        }
                                    }
                                } else if group_id == "org.apache.maven.plugins" {
                                    // Default groupId for Maven plugins
                                    if let Some(version) = &plugin.version {
                                        return Ok(Some(version.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Repository metadata
        // TODO: Try to resolve from repository metadata

        // 3. Default versions for standard plugins
        let default_versions: std::collections::HashMap<&str, &str> = [
            ("maven-compiler-plugin", "3.11.0"),
            ("maven-surefire-plugin", "3.0.0"),
            ("maven-jar-plugin", "3.3.0"),
            ("maven-install-plugin", "3.1.0"),
            ("maven-deploy-plugin", "3.1.1"),
            ("maven-clean-plugin", "3.2.0"),
            ("maven-resources-plugin", "3.3.1"),
        ].iter().cloned().collect();

        if group_id == "org.apache.maven.plugins" {
            if let Some(version) = default_versions.get(artifact_id) {
                return Ok(Some(version.to_string()));
            }
        }

        Ok(None)
    }
}

impl Default for MojoExecutor {
    fn default() -> Self {
        Self::new()
    }
}

