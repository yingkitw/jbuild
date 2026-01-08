use anyhow::{Context, Result};

use crate::core::execution::{MavenExecutionRequest, MavenExecutionResult};
use crate::core::project_builder::ProjectBuilder;
use crate::core::lifecycle_starter::LifecycleStarter;
use crate::core::reactor::Reactor;
use crate::core::session::MavenSession;
use crate::settings::Settings;
use crate::plugin_api::registry::PluginRegistry;

/// Default Maven implementation - main execution engine
pub struct DefaultMaven {
    project_builder: ProjectBuilder,
    lifecycle_starter: LifecycleStarter,
}

impl DefaultMaven {
    pub fn new() -> Self {
        Self {
            project_builder: ProjectBuilder::new(),
            lifecycle_starter: LifecycleStarter::new(),
        }
    }

    /// Execute a Maven build
    pub fn execute(&self, request: MavenExecutionRequest) -> Result<MavenExecutionResult> {
        // Load settings
        let settings = request.settings.unwrap_or_else(Settings::default);

        // Determine POM file
        let pom_file = request.pom_file.unwrap_or_else(|| {
            request.base_directory.join("pom.xml")
        });

        // Build projects (reactor)
        let projects = if request.reactor_active {
            self.project_builder
                .build_reactor(&pom_file)
                .context("Failed to build reactor")?
        } else {
            vec![self.project_builder
                .build(&pom_file)
                .context("Failed to build project")?]
        };

        // Create session
        let mut session = MavenSession::new(request.base_directory.clone(), settings);
        
        // Set projects in session
        for project in &projects {
            if session.current_project.is_none() {
                session.current_project = Some(project.clone());
            }
            session.projects.push(project.clone());
        }

        // Create reactor
        let reactor = Reactor::new(projects.clone());

        // Execute lifecycle for each project in build order
        let mut result = MavenExecutionResult::new();
        
        for project in reactor.build_order() {
            session.current_project = Some(project.clone());
            
            match self.lifecycle_starter.execute(&session, &request.goals) {
                Ok(exec_result) => {
                    if !exec_result.success {
                        result.add_exception(anyhow::anyhow!(
                            "Build failed for project {}",
                            project.id()
                        ));
                    }
                }
                Err(e) => {
                    result.add_exception(e.context(format!(
                        "Failed to execute lifecycle for project {}",
                        project.id()
                    )));
                }
            }
        }

        result.projects = projects;
        Ok(result)
    }

    pub fn with_plugin_registry(mut self, registry: PluginRegistry) -> Self {
        self.lifecycle_starter = self.lifecycle_starter.with_plugin_registry(registry);
        self
    }
}

impl Default for DefaultMaven {
    fn default() -> Self {
        Self::new()
    }
}

