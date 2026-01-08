//! Domain services for Maven context

use super::aggregates::{MavenProject, MavenPlugin};
use super::value_objects::LifecyclePhase;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Maven lifecycle execution service
/// Orchestrates the execution of Maven lifecycle phases
pub struct LifecycleExecutor {
    phase_bindings: HashMap<LifecyclePhase, Vec<String>>,
}

impl LifecycleExecutor {
    /// Creates a new lifecycle executor with default phase bindings
    pub fn new() -> Self {
        let mut phase_bindings = HashMap::new();
        
        // Default lifecycle bindings for JAR packaging
        phase_bindings.insert(
            LifecyclePhase::ProcessResources,
            vec!["resources:resources".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::Compile,
            vec!["compiler:compile".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::ProcessTestResources,
            vec!["resources:testResources".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::TestCompile,
            vec!["compiler:testCompile".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::Test,
            vec!["surefire:test".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::Package,
            vec!["jar:jar".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::Install,
            vec!["install:install".to_string()],
        );
        phase_bindings.insert(
            LifecyclePhase::Deploy,
            vec!["deploy:deploy".to_string()],
        );
        
        Self { phase_bindings }
    }
    
    /// Executes a lifecycle phase and all preceding phases
    pub fn execute_phase(
        &self,
        project: &MavenProject,
        phase: LifecyclePhase,
    ) -> Result<ExecutionPlan> {
        // Get all phases up to and including the target phase
        let phases_to_execute = phase.phases_up_to();
        
        let mut plan = ExecutionPlan::new();
        
        for exec_phase in phases_to_execute {
            // Get plugin goals bound to this phase
            if let Some(goals) = self.phase_bindings.get(&exec_phase) {
                for goal in goals {
                    plan.add_step(ExecutionStep {
                        phase: exec_phase,
                        goal: goal.clone(),
                        plugin: self.parse_plugin_goal(goal)?,
                    });
                }
            }
            
            // Add custom plugin executions bound to this phase
            for plugin in project.plugins() {
                for execution in plugin.executions() {
                    if execution.phase == Some(exec_phase) {
                        for goal in &execution.goals {
                            plan.add_step(ExecutionStep {
                                phase: exec_phase,
                                goal: format!("{}:{}", plugin.coordinates().artifact_id(), goal),
                                plugin: plugin.coordinates().clone(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(plan)
    }
    
    /// Executes a specific plugin goal
    pub fn execute_goal(
        &self,
        _project: &MavenProject,
        goal: &str,
    ) -> Result<()> {
        let _plugin = self.parse_plugin_goal(goal)?;
        
        // In a real implementation, this would invoke the plugin
        // For now, we just validate the goal format
        Ok(())
    }
    
    fn parse_plugin_goal(&self, goal: &str) -> Result<super::super::artifact::value_objects::ArtifactCoordinates> {
        let parts: Vec<&str> = goal.split(':').collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid goal format: {}", goal));
        }
        
        // For built-in plugins, use default group
        let group = if parts[0].contains('.') {
            parts[0]
        } else {
            "org.apache.maven.plugins"
        };
        
        let artifact = format!("maven-{}-plugin", parts[0]);
        
        super::super::artifact::value_objects::ArtifactCoordinates::new(
            group,
            artifact,
            "LATEST",
        )
    }
}

impl Default for LifecycleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution plan containing ordered steps
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    steps: Vec<ExecutionStep>,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }
    
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }
    
    pub fn steps(&self) -> &[ExecutionStep] {
        &self.steps
    }
    
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Single execution step in the plan
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub phase: LifecyclePhase,
    pub goal: String,
    pub plugin: super::super::artifact::value_objects::ArtifactCoordinates,
}

/// Plugin execution service
/// Manages plugin loading and execution
pub struct PluginExecutor;

impl PluginExecutor {
    /// Executes a plugin with the given configuration
    pub fn execute(
        _plugin: &MavenPlugin,
        _goal: &str,
        _configuration: &HashMap<String, String>,
    ) -> Result<()> {
        // In a real implementation, this would:
        // 1. Load the plugin JAR
        // 2. Instantiate the Mojo
        // 3. Configure the Mojo
        // 4. Execute the Mojo
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artifact::value_objects::ArtifactCoordinates;

    #[test]
    fn test_lifecycle_executor_creation() {
        let executor = LifecycleExecutor::new();
        assert!(!executor.phase_bindings.is_empty());
    }

    #[test]
    fn test_execute_compile_phase() {
        let executor = LifecycleExecutor::new();
        let coords = ArtifactCoordinates::new("com.example", "test", "1.0.0").unwrap();
        let project = MavenProject::new(coords, "/tmp/test").unwrap();
        
        let plan = executor.execute_phase(&project, LifecyclePhase::Compile);
        assert!(plan.is_ok());
        
        let plan = plan.unwrap();
        assert!(!plan.is_empty());
        
        // Should include process-resources and compile
        assert!(plan.len() >= 2);
    }

    #[test]
    fn test_execute_test_phase() {
        let executor = LifecycleExecutor::new();
        let coords = ArtifactCoordinates::new("com.example", "test", "1.0.0").unwrap();
        let project = MavenProject::new(coords, "/tmp/test").unwrap();
        
        let plan = executor.execute_phase(&project, LifecyclePhase::Test);
        assert!(plan.is_ok());
        
        let plan = plan.unwrap();
        // Should include all phases up to test
        assert!(plan.len() >= 5);
    }

    #[test]
    fn test_execute_package_phase() {
        let executor = LifecycleExecutor::new();
        let coords = ArtifactCoordinates::new("com.example", "test", "1.0.0").unwrap();
        let project = MavenProject::new(coords, "/tmp/test").unwrap();
        
        let plan = executor.execute_phase(&project, LifecyclePhase::Package);
        assert!(plan.is_ok());
        
        let plan = plan.unwrap();
        // Should include all phases up to package
        assert!(plan.len() >= 6);
    }

    #[test]
    fn test_parse_plugin_goal() {
        let executor = LifecycleExecutor::new();
        
        let result = executor.parse_plugin_goal("compiler:compile");
        assert!(result.is_ok());
        
        let coords = result.unwrap();
        assert_eq!(coords.group_id(), "org.apache.maven.plugins");
        assert_eq!(coords.artifact_id(), "maven-compiler-plugin");
    }

    #[test]
    fn test_execution_plan() {
        let mut plan = ExecutionPlan::new();
        assert!(plan.is_empty());
        
        let coords = ArtifactCoordinates::new("org.apache.maven.plugins", "maven-compiler-plugin", "3.11.0").unwrap();
        plan.add_step(ExecutionStep {
            phase: LifecyclePhase::Compile,
            goal: "compiler:compile".to_string(),
            plugin: coords,
        });
        
        assert!(!plan.is_empty());
        assert_eq!(plan.len(), 1);
    }
}
