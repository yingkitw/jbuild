use anyhow::Result;

use crate::core::session::MavenSession;
use crate::core::execution::MavenExecutionResult;
use crate::core::lifecycle::LifecyclePhase;
use crate::core::lifecycle_executor::LifecycleExecutor;
use crate::core::goal_parser::GoalParser;
use crate::plugin_api::registry::PluginRegistry;

/// Lifecycle starter - starts lifecycle execution
#[allow(dead_code)]
pub struct LifecycleStarter {
    lifecycle: crate::core::lifecycle::Lifecycle,
    lifecycle_executor: LifecycleExecutor,
    goal_parser: GoalParser,
}

impl LifecycleStarter {
    pub fn new() -> Self {
        Self {
            lifecycle: crate::core::lifecycle::Lifecycle::default(),
            lifecycle_executor: LifecycleExecutor::new(),
            goal_parser: GoalParser::new(),
        }
    }

    /// Execute lifecycle for a session
    pub fn execute(&self, session: &MavenSession, goals: &[String]) -> Result<MavenExecutionResult> {
        let mut result = crate::core::execution::MavenExecutionResult::new();

        if let Some(project) = &session.current_project {
            // Parse goals to determine target phase
            let target_phase = self.goal_parser.get_target_phase(goals)
                .unwrap_or({
                    // Default to compile if no phase found
                    LifecyclePhase::Compile
                });
            
            tracing::info!("Executing lifecycle up to phase: {}", target_phase);
            
            if let Err(e) = self.lifecycle_executor.execute_to_phase(session, project, &target_phase) {
                result.add_exception(e);
            }
        }

        Ok(result)
    }

    pub fn with_plugin_registry(mut self, registry: PluginRegistry) -> Self {
        self.lifecycle_executor = self.lifecycle_executor.with_plugin_registry(registry);
        self
    }
}

impl Default for LifecycleStarter {
    fn default() -> Self {
        Self::new()
    }
}

