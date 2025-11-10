use anyhow::Result;

use crate::core::session::MavenSession;
use crate::core::project::MavenProject;
use crate::core::lifecycle::LifecyclePhase;
use crate::core::mojo_executor::{MojoExecutor, MojoExecution};
use crate::plugin_api::registry::PluginRegistry;

/// Lifecycle executor - executes lifecycle phases with plugin bindings
pub struct LifecycleExecutor {
    mojo_executor: MojoExecutor,
}

impl LifecycleExecutor {
    pub fn new() -> Self {
        Self {
            mojo_executor: MojoExecutor::new(),
        }
    }

    /// Execute a lifecycle phase for a project
    pub fn execute_phase(
        &self,
        session: &MavenSession,
        project: &MavenProject,
        phase: &LifecyclePhase,
    ) -> Result<()> {
        tracing::info!("Executing phase: {} for project {}", phase, project.id());

        // Get plugin bindings for this phase
        let mojo_executions = self.get_phase_mojos(project, phase)?;

        // Execute all mojos for this phase
        self.mojo_executor.execute(session, &mojo_executions)?;

        Ok(())
    }

    /// Get mojo executions for a lifecycle phase
    fn get_phase_mojos(
        &self,
        _project: &MavenProject,
        phase: &LifecyclePhase,
    ) -> Result<Vec<MojoExecution>> {
        let mut executions = Vec::new();

        // Get default plugin bindings for standard lifecycle phases
        match phase {
            LifecyclePhase::Compile => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-compiler-plugin".to_string(),
                    version: None, // Will be resolved by MojoExecutor
                    goal: "compile".to_string(),
                    phase: Some("compile".to_string()),
                    configuration: None,
                });
            }
            LifecyclePhase::TestCompile => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-compiler-plugin".to_string(),
                    version: None,
                    goal: "testCompile".to_string(),
                    phase: Some("test-compile".to_string()),
                    configuration: None,
                });
            }
            LifecyclePhase::Test => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-surefire-plugin".to_string(),
                    version: None,
                    goal: "test".to_string(),
                    phase: Some("test".to_string()),
                    configuration: None,
                });
            }
            LifecyclePhase::Package => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-jar-plugin".to_string(),
                    version: None,
                    goal: "jar".to_string(),
                    phase: Some("package".to_string()),
                    configuration: None,
                });
            }
            LifecyclePhase::Install => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-install-plugin".to_string(),
                    version: None,
                    goal: "install".to_string(),
                    phase: Some("install".to_string()),
                    configuration: None,
                });
            }
            LifecyclePhase::Deploy => {
                executions.push(MojoExecution {
                    group_id: "org.apache.maven.plugins".to_string(),
                    artifact_id: "maven-deploy-plugin".to_string(),
                    version: None,
                    goal: "deploy".to_string(),
                    phase: Some("deploy".to_string()),
                    configuration: None,
                });
            }
            _ => {
                // Other phases may not have default bindings
                // Clean is a separate lifecycle, not part of default lifecycle
            }
        }

        // TODO: Also check project.build.plugins for custom bindings
        // TODO: Check pluginManagement for version resolution

        Ok(executions)
    }

    /// Execute lifecycle up to a given phase
    pub fn execute_to_phase(
        &self,
        session: &MavenSession,
        project: &MavenProject,
        target_phase: &LifecyclePhase,
    ) -> Result<()> {
        let all_phases = LifecyclePhase::all();
        
        for phase in all_phases {
            if phase == *target_phase {
                self.execute_phase(session, project, &phase)?;
                break;
            }
            self.execute_phase(session, project, &phase)?;
        }

        Ok(())
    }

    pub fn with_plugin_registry(self, _registry: PluginRegistry) -> Self {
        // TODO: Store registry and use it for plugin loading
        self
    }
}

impl Default for LifecycleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

