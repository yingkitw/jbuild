//! Maven Execution Plan
//!
//! Implements Maven's MavenExecutionPlan for calculating and managing
//! the execution plan for a build.

use std::collections::HashMap;
use crate::core::lifecycle::LifecyclePhase;

/// An execution plan item representing a mojo to execute
#[derive(Debug, Clone)]
pub struct ExecutionPlanItem {
    /// Plugin group ID
    pub plugin_group_id: String,
    /// Plugin artifact ID
    pub plugin_artifact_id: String,
    /// Plugin version
    pub plugin_version: String,
    /// Goal name
    pub goal: String,
    /// Execution ID
    pub execution_id: String,
    /// Lifecycle phase this is bound to
    pub lifecycle_phase: Option<LifecyclePhase>,
    /// Configuration for this execution
    pub configuration: HashMap<String, String>,
    /// Whether this is a direct invocation (not lifecycle-bound)
    pub direct_invocation: bool,
}

impl ExecutionPlanItem {
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
        goal: impl Into<String>,
    ) -> Self {
        Self {
            plugin_group_id: group_id.into(),
            plugin_artifact_id: artifact_id.into(),
            plugin_version: version.into(),
            goal: goal.into(),
            execution_id: "default".to_string(),
            lifecycle_phase: None,
            configuration: HashMap::new(),
            direct_invocation: false,
        }
    }

    pub fn with_execution_id(mut self, id: impl Into<String>) -> Self {
        self.execution_id = id.into();
        self
    }

    pub fn with_phase(mut self, phase: LifecyclePhase) -> Self {
        self.lifecycle_phase = Some(phase);
        self
    }

    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.configuration.insert(key.into(), value.into());
        self
    }

    /// Get the full plugin key (groupId:artifactId)
    pub fn plugin_key(&self) -> String {
        format!("{}:{}", self.plugin_group_id, self.plugin_artifact_id)
    }

    /// Get the full mojo descriptor key
    pub fn mojo_key(&self) -> String {
        format!("{}:{}:{}:{}", 
            self.plugin_group_id, 
            self.plugin_artifact_id, 
            self.plugin_version,
            self.goal
        )
    }
}

/// Maven execution plan containing all mojos to execute
#[derive(Debug, Default)]
pub struct MavenExecutionPlan {
    /// Ordered list of execution items
    items: Vec<ExecutionPlanItem>,
    /// Phase to items mapping for quick lookup
    phase_items: HashMap<LifecyclePhase, Vec<usize>>,
}

impl MavenExecutionPlan {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an execution item to the plan
    pub fn add(&mut self, item: ExecutionPlanItem) {
        let index = self.items.len();
        if let Some(phase) = &item.lifecycle_phase {
            self.phase_items
                .entry(phase.clone())
                .or_default()
                .push(index);
        }
        self.items.push(item);
    }

    /// Get all items in execution order
    pub fn items(&self) -> &[ExecutionPlanItem] {
        &self.items
    }

    /// Get items for a specific phase
    pub fn items_for_phase(&self, phase: &LifecyclePhase) -> Vec<&ExecutionPlanItem> {
        self.phase_items
            .get(phase)
            .map(|indices| indices.iter().map(|&i| &self.items[i]).collect())
            .unwrap_or_default()
    }

    /// Get the number of items in the plan
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the plan is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get all phases in the plan
    pub fn phases(&self) -> Vec<LifecyclePhase> {
        let mut phases: Vec<_> = self.phase_items.keys().cloned().collect();
        phases.sort_by_key(|p| p.order());
        phases
    }

    /// Calculate the plan for executing up to a target phase
    pub fn calculate_for_phase(target_phase: LifecyclePhase) -> Self {
        let mut plan = Self::new();

        // Add default plugin bindings for each phase up to target
        for phase in LifecyclePhase::phases_up_to(&target_phase) {
            for binding in default_bindings_for_phase(&phase) {
                plan.add(binding);
            }
        }

        plan
    }
}

/// Get default plugin bindings for a lifecycle phase
fn default_bindings_for_phase(phase: &LifecyclePhase) -> Vec<ExecutionPlanItem> {
    match phase {
        LifecyclePhase::Validate => vec![],
        LifecyclePhase::Initialize => vec![],
        LifecyclePhase::GenerateSources => vec![],
        LifecyclePhase::ProcessSources => vec![],
        LifecyclePhase::GenerateResources => vec![],
        LifecyclePhase::ProcessResources => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-resources-plugin",
                "3.3.1",
                "resources",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::Compile => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-compiler-plugin",
                "3.11.0",
                "compile",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::ProcessClasses => vec![],
        LifecyclePhase::GenerateTestSources => vec![],
        LifecyclePhase::ProcessTestSources => vec![],
        LifecyclePhase::GenerateTestResources => vec![],
        LifecyclePhase::ProcessTestResources => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-resources-plugin",
                "3.3.1",
                "testResources",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::TestCompile => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-compiler-plugin",
                "3.11.0",
                "testCompile",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::ProcessTestClasses => vec![],
        LifecyclePhase::Test => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-surefire-plugin",
                "3.1.2",
                "test",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::PreparePackage => vec![],
        LifecyclePhase::Package => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-jar-plugin",
                "3.3.0",
                "jar",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::PreIntegrationTest => vec![],
        LifecyclePhase::IntegrationTest => vec![],
        LifecyclePhase::PostIntegrationTest => vec![],
        LifecyclePhase::Verify => vec![],
        LifecyclePhase::Install => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-install-plugin",
                "3.1.1",
                "install",
            ).with_phase(phase.clone()),
        ],
        LifecyclePhase::Deploy => vec![
            ExecutionPlanItem::new(
                "org.apache.maven.plugins",
                "maven-deploy-plugin",
                "3.1.1",
                "deploy",
            ).with_phase(phase.clone()),
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan_item() {
        let item = ExecutionPlanItem::new(
            "org.apache.maven.plugins",
            "maven-compiler-plugin",
            "3.11.0",
            "compile",
        ).with_phase(LifecyclePhase::Compile);

        assert_eq!(item.plugin_key(), "org.apache.maven.plugins:maven-compiler-plugin");
        assert_eq!(item.lifecycle_phase, Some(LifecyclePhase::Compile));
    }

    #[test]
    fn test_execution_plan_add() {
        let mut plan = MavenExecutionPlan::new();
        plan.add(ExecutionPlanItem::new("g", "a", "1.0", "goal"));

        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[test]
    fn test_execution_plan_phases() {
        let mut plan = MavenExecutionPlan::new();
        plan.add(ExecutionPlanItem::new("g", "a", "1.0", "compile")
            .with_phase(LifecyclePhase::Compile));
        plan.add(ExecutionPlanItem::new("g", "a", "1.0", "test")
            .with_phase(LifecyclePhase::Test));

        let phases = plan.phases();
        assert_eq!(phases.len(), 2);
    }

    #[test]
    fn test_calculate_for_phase() {
        let plan = MavenExecutionPlan::calculate_for_phase(LifecyclePhase::Compile);

        assert!(!plan.is_empty());
        // Should have resources and compile plugins
        let phases = plan.phases();
        assert!(phases.contains(&LifecyclePhase::ProcessResources));
        assert!(phases.contains(&LifecyclePhase::Compile));
    }

    #[test]
    fn test_items_for_phase() {
        let mut plan = MavenExecutionPlan::new();
        plan.add(ExecutionPlanItem::new("g", "a", "1.0", "compile")
            .with_phase(LifecyclePhase::Compile));
        plan.add(ExecutionPlanItem::new("g", "b", "1.0", "compile2")
            .with_phase(LifecyclePhase::Compile));

        let compile_items = plan.items_for_phase(&LifecyclePhase::Compile);
        assert_eq!(compile_items.len(), 2);
    }
}
