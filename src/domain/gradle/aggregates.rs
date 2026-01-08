//! Aggregates for Gradle context

// Placeholder for Gradle aggregates:
// - GradleProject (root aggregate)
//   Contains: Tasks, Configurations, SourceSets, Plugins
//   Invariants: Valid task graph (no cycles), valid configurations

use crate::domain::artifact::value_objects::{ArtifactCoordinates, Scope};
use crate::domain::shared::value_objects::{FilePath, JavaVersion, Version};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// GradleProject is the aggregate root for Gradle projects
/// It maintains consistency boundaries and enforces business invariants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleProject {
    // Identity
    name: String,
    group: String,
    version: Version,
    
    // Project metadata
    description: Option<String>,
    
    // Build configuration
    base_directory: FilePath,
    build_directory: FilePath,
    source_sets: Vec<SourceSet>,
    
    // Java configuration
    java_version: JavaVersion,
    
    // Dependencies (part of aggregate)
    configurations: HashMap<String, Configuration>,
    
    // Tasks (part of aggregate)
    tasks: HashMap<String, GradleTask>,
    
    // Plugins
    plugins: Vec<String>,
    
    // Properties
    properties: HashMap<String, String>,
    
    // Subprojects (for multi-project builds)
    subprojects: Vec<String>,
}

impl GradleProject {
    /// Creates a new GradleProject with required fields
    pub fn new(
        name: impl Into<String>,
        group: impl Into<String>,
        version: impl Into<String>,
        base_directory: impl Into<PathBuf>,
    ) -> Result<Self> {
        let name = name.into();
        let group = group.into();
        let version_str = version.into();
        let base_path = base_directory.into();
        
        // Validate inputs
        if name.trim().is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }
        if group.trim().is_empty() {
            return Err(anyhow!("Project group cannot be empty"));
        }
        
        let mut configurations = HashMap::new();
        configurations.insert("implementation".to_string(), Configuration::new("implementation"));
        configurations.insert("testImplementation".to_string(), Configuration::new("testImplementation"));
        configurations.insert("compileOnly".to_string(), Configuration::new("compileOnly"));
        configurations.insert("runtimeOnly".to_string(), Configuration::new("runtimeOnly"));
        
        Ok(Self {
            name,
            group,
            version: Version::new(version_str),
            description: None,
            base_directory: FilePath::new(base_path.clone()),
            build_directory: FilePath::new(base_path.join("build")),
            source_sets: vec![
                SourceSet::main(),
                SourceSet::test(),
            ],
            java_version: JavaVersion::new(17, 0, 0),
            configurations,
            tasks: HashMap::new(),
            plugins: Vec::new(),
            properties: HashMap::new(),
            subprojects: Vec::new(),
        })
    }
    
    /// Returns the project coordinates as Maven-style GAV
    pub fn coordinates(&self) -> Result<ArtifactCoordinates> {
        ArtifactCoordinates::new(&self.group, &self.name, self.version.as_str())
    }
    
    /// Sets project metadata
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Sets the Java version
    pub fn with_java_version(mut self, version: JavaVersion) -> Self {
        self.java_version = version;
        self
    }
    
    /// Applies a plugin
    pub fn apply_plugin(&mut self, plugin_id: String) -> Result<()> {
        // Invariant: No duplicate plugins
        if self.plugins.contains(&plugin_id) {
            return Err(anyhow!("Plugin {} already applied", plugin_id));
        }
        
        self.plugins.push(plugin_id);
        Ok(())
    }
    
    /// Adds a dependency to a configuration
    pub fn add_dependency(&mut self, configuration: &str, dependency: GradleDependency) -> Result<()> {
        let config = self.configurations
            .get_mut(configuration)
            .ok_or_else(|| anyhow!("Configuration {} not found", configuration))?;
        
        config.add_dependency(dependency)?;
        Ok(())
    }
    
    /// Creates a new configuration
    pub fn create_configuration(&mut self, name: String) -> Result<()> {
        // Invariant: No duplicate configurations
        if self.configurations.contains_key(&name) {
            return Err(anyhow!("Configuration {} already exists", name));
        }
        
        self.configurations.insert(name.clone(), Configuration::new(name));
        Ok(())
    }
    
    /// Registers a task
    pub fn register_task(&mut self, task: GradleTask) -> Result<()> {
        let task_name = task.name().to_string();
        
        // Invariant: No duplicate tasks
        if self.tasks.contains_key(&task_name) {
            return Err(anyhow!("Task {} already exists", task_name));
        }
        
        self.tasks.insert(task_name, task);
        Ok(())
    }
    
    /// Adds a task dependency
    pub fn add_task_dependency(&mut self, task_name: &str, depends_on: String) -> Result<()> {
        // Invariant: Dependency task must exist
        if !self.tasks.contains_key(&depends_on) {
            return Err(anyhow!("Dependency task {} not found", depends_on));
        }
        
        let task = self.tasks
            .get_mut(task_name)
            .ok_or_else(|| anyhow!("Task {} not found", task_name))?;
        
        task.add_dependency(depends_on)?;
        Ok(())
    }
    
    /// Validates task dependencies for cycles
    pub fn validate_task_graph(&self) -> Result<()> {
        for (task_name, task) in &self.tasks {
            let mut visited = HashSet::new();
            let mut stack = HashSet::new();
            
            if self.has_cycle(task_name, &mut visited, &mut stack)? {
                return Err(anyhow!("Circular task dependency detected involving {}", task_name));
            }
        }
        
        Ok(())
    }
    
    fn has_cycle(&self, task_name: &str, visited: &mut HashSet<String>, stack: &mut HashSet<String>) -> Result<bool> {
        if stack.contains(task_name) {
            return Ok(true);
        }
        
        if visited.contains(task_name) {
            return Ok(false);
        }
        
        visited.insert(task_name.to_string());
        stack.insert(task_name.to_string());
        
        if let Some(task) = self.tasks.get(task_name) {
            for dep in task.dependencies() {
                if self.has_cycle(dep, visited, stack)? {
                    return Ok(true);
                }
            }
        }
        
        stack.remove(task_name);
        Ok(false)
    }
    
    /// Adds a subproject
    pub fn add_subproject(&mut self, name: String) -> Result<()> {
        // Invariant: No duplicate subprojects
        if self.subprojects.contains(&name) {
            return Err(anyhow!("Subproject {} already exists", name));
        }
        
        self.subprojects.push(name);
        Ok(())
    }
    
    /// Sets a property
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }
    
    /// Checks if this is a multi-project build
    pub fn is_multi_project(&self) -> bool {
        !self.subprojects.is_empty()
    }
    
    /// Validates the project state
    pub fn validate(&self) -> Result<()> {
        // Invariant: Valid name and group
        if self.name.trim().is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }
        if self.group.trim().is_empty() {
            return Err(anyhow!("Project group cannot be empty"));
        }
        
        // Invariant: No circular task dependencies
        self.validate_task_graph()?;
        
        Ok(())
    }
    
    // Getters
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn group(&self) -> &str {
        &self.group
    }
    
    pub fn version(&self) -> &Version {
        &self.version
    }
    
    pub fn base_directory(&self) -> &FilePath {
        &self.base_directory
    }
    
    pub fn java_version(&self) -> &JavaVersion {
        &self.java_version
    }
    
    pub fn configurations(&self) -> &HashMap<String, Configuration> {
        &self.configurations
    }
    
    pub fn tasks(&self) -> &HashMap<String, GradleTask> {
        &self.tasks
    }
    
    pub fn plugins(&self) -> &[String] {
        &self.plugins
    }
    
    pub fn subprojects(&self) -> &[String] {
        &self.subprojects
    }
}

/// Configuration for dependency management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    name: String,
    dependencies: Vec<GradleDependency>,
    extends_from: Vec<String>,
}

impl Configuration {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: Vec::new(),
            extends_from: Vec::new(),
        }
    }
    
    pub fn add_dependency(&mut self, dependency: GradleDependency) -> Result<()> {
        // Invariant: No duplicate dependencies
        if self.dependencies.iter().any(|d| d.coordinates == dependency.coordinates) {
            return Err(anyhow!(
                "Dependency {} already exists in configuration {}",
                dependency.coordinates.gav(),
                self.name
            ));
        }
        
        self.dependencies.push(dependency);
        Ok(())
    }
    
    pub fn extend_from(&mut self, config_name: String) {
        self.extends_from.push(config_name);
    }
    
    pub fn dependencies(&self) -> &[GradleDependency] {
        &self.dependencies
    }
}

/// Gradle dependency entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradleDependency {
    coordinates: ArtifactCoordinates,
    transitive: bool,
    exclude_rules: Vec<ExcludeRule>,
}

impl GradleDependency {
    pub fn new(coordinates: ArtifactCoordinates) -> Self {
        Self {
            coordinates,
            transitive: true,
            exclude_rules: Vec::new(),
        }
    }
    
    pub fn with_transitive(mut self, transitive: bool) -> Self {
        self.transitive = transitive;
        self
    }
    
    pub fn add_exclude(&mut self, rule: ExcludeRule) {
        self.exclude_rules.push(rule);
    }
    
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
    
    pub fn is_transitive(&self) -> bool {
        self.transitive
    }
}

/// Exclude rule for dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcludeRule {
    group: Option<String>,
    module: Option<String>,
}

impl ExcludeRule {
    pub fn by_group(group: String) -> Self {
        Self {
            group: Some(group),
            module: None,
        }
    }
    
    pub fn by_module(module: String) -> Self {
        Self {
            group: None,
            module: Some(module),
        }
    }
}

/// Gradle task entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleTask {
    name: String,
    task_type: String,
    dependencies: Vec<String>,
    description: Option<String>,
    group: Option<String>,
}

impl GradleTask {
    pub fn new(name: impl Into<String>, task_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            task_type: task_type.into(),
            dependencies: Vec::new(),
            description: None,
            group: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_group(mut self, group: String) -> Self {
        self.group = Some(group);
        self
    }
    
    pub fn add_dependency(&mut self, task_name: String) -> Result<()> {
        // Invariant: No duplicate dependencies
        if self.dependencies.contains(&task_name) {
            return Err(anyhow!("Task dependency {} already exists", task_name));
        }
        
        self.dependencies.push(task_name);
        Ok(())
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

/// Source set configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSet {
    name: String,
    java_src_dirs: Vec<PathBuf>,
    resources_dirs: Vec<PathBuf>,
    output_dir: PathBuf,
}

impl SourceSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            java_src_dirs: Vec::new(),
            resources_dirs: Vec::new(),
            output_dir: PathBuf::new(),
        }
    }
    
    pub fn main() -> Self {
        Self {
            name: "main".to_string(),
            java_src_dirs: vec![PathBuf::from("src/main/java")],
            resources_dirs: vec![PathBuf::from("src/main/resources")],
            output_dir: PathBuf::from("build/classes/java/main"),
        }
    }
    
    pub fn test() -> Self {
        Self {
            name: "test".to_string(),
            java_src_dirs: vec![PathBuf::from("src/test/java")],
            resources_dirs: vec![PathBuf::from("src/test/resources")],
            output_dir: PathBuf::from("build/classes/java/test"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradle_project_creation() {
        let project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        assert_eq!(project.name(), "my-app");
        assert_eq!(project.group(), "com.example");
        assert_eq!(project.version().as_str(), "1.0.0");
    }

    #[test]
    fn test_apply_plugin() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        assert!(project.apply_plugin("java".to_string()).is_ok());
        assert_eq!(project.plugins().len(), 1);
    }

    #[test]
    fn test_duplicate_plugin_rejected() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        assert!(project.apply_plugin("java".to_string()).is_ok());
        assert!(project.apply_plugin("java".to_string()).is_err());
    }

    #[test]
    fn test_add_dependency() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        let coords = ArtifactCoordinates::new("org.junit.jupiter", "junit-jupiter", "5.10.0").unwrap();
        let dep = GradleDependency::new(coords);
        
        assert!(project.add_dependency("testImplementation", dep).is_ok());
    }

    #[test]
    fn test_register_task() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        let task = GradleTask::new("customTask", "DefaultTask");
        assert!(project.register_task(task).is_ok());
        assert_eq!(project.tasks().len(), 1);
    }

    #[test]
    fn test_task_dependency() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        let task1 = GradleTask::new("compile", "JavaCompile");
        let task2 = GradleTask::new("test", "Test");
        
        project.register_task(task1).unwrap();
        project.register_task(task2).unwrap();
        
        assert!(project.add_task_dependency("test", "compile".to_string()).is_ok());
    }

    #[test]
    fn test_circular_task_dependency_detected() {
        let mut project = GradleProject::new("my-app", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        let mut task1 = GradleTask::new("task1", "DefaultTask");
        let mut task2 = GradleTask::new("task2", "DefaultTask");
        
        task1.add_dependency("task2".to_string()).unwrap();
        task2.add_dependency("task1".to_string()).unwrap();
        
        project.register_task(task1).unwrap();
        project.register_task(task2).unwrap();
        
        assert!(project.validate_task_graph().is_err());
    }

    #[test]
    fn test_multi_project() {
        let mut project = GradleProject::new("parent", "com.example", "1.0.0", "/tmp/project").unwrap();
        
        project.add_subproject("module1".to_string()).unwrap();
        project.add_subproject("module2".to_string()).unwrap();
        
        assert!(project.is_multi_project());
        assert_eq!(project.subprojects().len(), 2);
    }
}
