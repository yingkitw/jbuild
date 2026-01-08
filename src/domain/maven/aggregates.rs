//! Aggregates for Maven context

use crate::domain::artifact::value_objects::{ArtifactCoordinates, Scope};
use crate::domain::maven::value_objects::LifecyclePhase;
use crate::domain::shared::value_objects::{FilePath, JavaVersion, Version};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// MavenProject is the aggregate root for Maven projects
/// It maintains consistency boundaries and enforces business invariants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenProject {
    // Identity
    coordinates: ArtifactCoordinates,
    
    // Project metadata
    name: Option<String>,
    description: Option<String>,
    url: Option<String>,
    
    // Build configuration
    base_directory: FilePath,
    source_directory: FilePath,
    test_source_directory: FilePath,
    output_directory: FilePath,
    test_output_directory: FilePath,
    
    // Java configuration
    java_version: JavaVersion,
    
    // Dependencies (part of aggregate)
    dependencies: Vec<MavenDependency>,
    dependency_management: Vec<MavenDependency>,
    
    // Plugins (part of aggregate)
    plugins: Vec<MavenPlugin>,
    plugin_management: Vec<MavenPlugin>,
    
    // Properties
    properties: HashMap<String, String>,
    
    // Parent relationship
    parent: Option<ParentReference>,
    
    // Modules (for multi-module projects)
    modules: Vec<String>,
    
    // Packaging type
    packaging: PackagingType,
}

impl MavenProject {
    /// Creates a new MavenProject with required fields
    pub fn new(
        coordinates: ArtifactCoordinates,
        base_directory: impl Into<PathBuf>,
    ) -> Result<Self> {
        let base_path = base_directory.into();
        
        // Validate coordinates
        if !coordinates.is_valid() {
            return Err(anyhow!("Invalid artifact coordinates"));
        }
        
        Ok(Self {
            coordinates,
            name: None,
            description: None,
            url: None,
            base_directory: FilePath::new(base_path.clone()),
            source_directory: FilePath::new(base_path.join("src/main/java")),
            test_source_directory: FilePath::new(base_path.join("src/test/java")),
            output_directory: FilePath::new(base_path.join("target/classes")),
            test_output_directory: FilePath::new(base_path.join("target/test-classes")),
            java_version: JavaVersion::new(17, 0, 0), // Default to Java 17
            dependencies: Vec::new(),
            dependency_management: Vec::new(),
            plugins: Vec::new(),
            plugin_management: Vec::new(),
            properties: HashMap::new(),
            parent: None,
            modules: Vec::new(),
            packaging: PackagingType::Jar,
        })
    }
    
    /// Returns the project coordinates
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
    
    /// Returns the project version
    pub fn version(&self) -> Version {
        Version::new(self.coordinates.version())
    }
    
    /// Sets project metadata
    pub fn with_metadata(
        mut self,
        name: Option<String>,
        description: Option<String>,
        url: Option<String>,
    ) -> Self {
        self.name = name;
        self.description = description;
        self.url = url;
        self
    }
    
    /// Sets the Java version
    pub fn with_java_version(mut self, version: JavaVersion) -> Self {
        self.java_version = version;
        self
    }
    
    /// Sets the packaging type
    pub fn with_packaging(mut self, packaging: PackagingType) -> Self {
        self.packaging = packaging;
        self
    }
    
    /// Adds a dependency with validation
    pub fn add_dependency(&mut self, dependency: MavenDependency) -> Result<()> {
        // Invariant: No duplicate dependencies with same coordinates
        if self.dependencies.iter().any(|d| d.coordinates == dependency.coordinates) {
            return Err(anyhow!(
                "Dependency {} already exists",
                dependency.coordinates.gav()
            ));
        }
        
        self.dependencies.push(dependency);
        Ok(())
    }
    
    /// Adds a plugin with validation
    pub fn add_plugin(&mut self, plugin: MavenPlugin) -> Result<()> {
        // Invariant: No duplicate plugins with same coordinates
        if self.plugins.iter().any(|p| p.coordinates == plugin.coordinates) {
            return Err(anyhow!(
                "Plugin {} already exists",
                plugin.coordinates.gav()
            ));
        }
        
        self.plugins.push(plugin);
        Ok(())
    }
    
    /// Sets a property
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }
    
    /// Gets a property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
    
    /// Adds a module (for multi-module projects)
    pub fn add_module(&mut self, module: String) -> Result<()> {
        // Invariant: No duplicate modules
        if self.modules.contains(&module) {
            return Err(anyhow!("Module {module} already exists"));
        }
        
        self.modules.push(module);
        Ok(())
    }
    
    /// Returns all dependencies for a given scope
    pub fn dependencies_for_scope(&self, scope: Scope) -> Vec<&MavenDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.scope == scope)
            .collect()
    }
    
    /// Returns all compile dependencies (compile + provided + system)
    pub fn compile_dependencies(&self) -> Vec<&MavenDependency> {
        self.dependencies
            .iter()
            .filter(|d| matches!(d.scope, Scope::Compile | Scope::Provided | Scope::System))
            .collect()
    }
    
    /// Returns all test dependencies
    pub fn test_dependencies(&self) -> Vec<&MavenDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.scope == Scope::Test)
            .collect()
    }
    
    /// Checks if this is a multi-module project
    pub fn is_multi_module(&self) -> bool {
        !self.modules.is_empty()
    }
    
    /// Validates the project state
    pub fn validate(&self) -> Result<()> {
        // Invariant: Valid coordinates
        if !self.coordinates.is_valid() {
            return Err(anyhow!("Invalid project coordinates"));
        }
        
        // Invariant: Multi-module projects must have POM packaging
        if self.is_multi_module() && self.packaging != PackagingType::Pom {
            return Err(anyhow!(
                "Multi-module projects must have POM packaging, found {:?}",
                self.packaging
            ));
        }
        
        // Invariant: No circular module dependencies (simplified check)
        let module_set: std::collections::HashSet<_> = self.modules.iter().collect();
        if module_set.len() != self.modules.len() {
            return Err(anyhow!("Duplicate modules detected"));
        }
        
        Ok(())
    }
    
    // Getters
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    pub fn base_directory(&self) -> &FilePath {
        &self.base_directory
    }
    
    pub fn source_directory(&self) -> &FilePath {
        &self.source_directory
    }
    
    pub fn output_directory(&self) -> &FilePath {
        &self.output_directory
    }
    
    pub fn java_version(&self) -> &JavaVersion {
        &self.java_version
    }
    
    pub fn dependencies(&self) -> &[MavenDependency] {
        &self.dependencies
    }
    
    pub fn plugins(&self) -> &[MavenPlugin] {
        &self.plugins
    }
    
    pub fn modules(&self) -> &[String] {
        &self.modules
    }
    
    pub fn packaging(&self) -> &PackagingType {
        &self.packaging
    }
}

/// Maven dependency entity (part of MavenProject aggregate)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MavenDependency {
    coordinates: ArtifactCoordinates,
    scope: Scope,
    optional: bool,
    exclusions: Vec<ArtifactCoordinates>,
}

impl MavenDependency {
    pub fn new(coordinates: ArtifactCoordinates, scope: Scope) -> Self {
        Self {
            coordinates,
            scope,
            optional: false,
            exclusions: Vec::new(),
        }
    }
    
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
    
    pub fn add_exclusion(&mut self, exclusion: ArtifactCoordinates) {
        self.exclusions.push(exclusion);
    }
    
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
    
    pub fn scope(&self) -> Scope {
        self.scope
    }
    
    pub fn is_optional(&self) -> bool {
        self.optional
    }
    
    pub fn exclusions(&self) -> &[ArtifactCoordinates] {
        &self.exclusions
    }
}

/// Maven plugin entity (part of MavenProject aggregate)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MavenPlugin {
    coordinates: ArtifactCoordinates,
    executions: Vec<PluginExecution>,
    configuration: HashMap<String, String>,
}

impl MavenPlugin {
    pub fn new(coordinates: ArtifactCoordinates) -> Self {
        Self {
            coordinates,
            executions: Vec::new(),
            configuration: HashMap::new(),
        }
    }
    
    pub fn add_execution(&mut self, execution: PluginExecution) {
        self.executions.push(execution);
    }
    
    pub fn set_configuration(&mut self, key: String, value: String) {
        self.configuration.insert(key, value);
    }
    
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
    
    pub fn executions(&self) -> &[PluginExecution] {
        &self.executions
    }
}

/// Plugin execution configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginExecution {
    id: String,
    pub phase: Option<LifecyclePhase>,
    pub goals: Vec<String>,
}

impl PluginExecution {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            phase: None,
            goals: Vec::new(),
        }
    }
    
    pub fn with_phase(mut self, phase: LifecyclePhase) -> Self {
        self.phase = Some(phase);
        self
    }
    
    pub fn add_goal(&mut self, goal: String) {
        self.goals.push(goal);
    }
}

/// Parent reference for POM inheritance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentReference {
    coordinates: ArtifactCoordinates,
    relative_path: Option<String>,
}

impl ParentReference {
    pub fn new(coordinates: ArtifactCoordinates) -> Self {
        Self {
            coordinates,
            relative_path: Some("../pom.xml".to_string()),
        }
    }
    
    pub fn with_relative_path(mut self, path: String) -> Self {
        self.relative_path = Some(path);
        self
    }
}

/// Packaging type for Maven projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackagingType {
    Jar,
    War,
    Ear,
    Pom,
    MavenPlugin,
}

impl PackagingType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "jar" => Some(PackagingType::Jar),
            "war" => Some(PackagingType::War),
            "ear" => Some(PackagingType::Ear),
            "pom" => Some(PackagingType::Pom),
            "maven-plugin" => Some(PackagingType::MavenPlugin),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            PackagingType::Jar => "jar",
            PackagingType::War => "war",
            PackagingType::Ear => "ear",
            PackagingType::Pom => "pom",
            PackagingType::MavenPlugin => "maven-plugin",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maven_project_creation() {
        let coords = ArtifactCoordinates::new("com.example", "my-app", "1.0.0").unwrap();
        let project = MavenProject::new(coords, "/tmp/project").unwrap();
        
        assert_eq!(project.coordinates().group_id(), "com.example");
        assert_eq!(project.coordinates().artifact_id(), "my-app");
        assert_eq!(project.version().as_str(), "1.0.0");
    }

    #[test]
    fn test_add_dependency() {
        let coords = ArtifactCoordinates::new("com.example", "my-app", "1.0.0").unwrap();
        let mut project = MavenProject::new(coords, "/tmp/project").unwrap();
        
        let dep_coords = ArtifactCoordinates::new("org.junit", "junit", "4.13").unwrap();
        let dep = MavenDependency::new(dep_coords, Scope::Test);
        
        assert!(project.add_dependency(dep).is_ok());
        assert_eq!(project.dependencies().len(), 1);
    }

    #[test]
    fn test_duplicate_dependency_rejected() {
        let coords = ArtifactCoordinates::new("com.example", "my-app", "1.0.0").unwrap();
        let mut project = MavenProject::new(coords, "/tmp/project").unwrap();
        
        let dep_coords = ArtifactCoordinates::new("org.junit", "junit", "4.13").unwrap();
        let dep1 = MavenDependency::new(dep_coords.clone(), Scope::Test);
        let dep2 = MavenDependency::new(dep_coords, Scope::Test);
        
        assert!(project.add_dependency(dep1).is_ok());
        assert!(project.add_dependency(dep2).is_err());
    }

    #[test]
    fn test_multi_module_validation() {
        let coords = ArtifactCoordinates::new("com.example", "parent", "1.0.0").unwrap();
        let mut project = MavenProject::new(coords, "/tmp/project")
            .unwrap()
            .with_packaging(PackagingType::Pom);
        
        project.add_module("module1".to_string()).unwrap();
        project.add_module("module2".to_string()).unwrap();
        
        assert!(project.validate().is_ok());
        assert!(project.is_multi_module());
    }

    #[test]
    fn test_multi_module_must_be_pom() {
        let coords = ArtifactCoordinates::new("com.example", "parent", "1.0.0").unwrap();
        let mut project = MavenProject::new(coords, "/tmp/project").unwrap();
        
        project.add_module("module1".to_string()).unwrap();
        
        // Should fail validation - multi-module with JAR packaging
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_dependencies_by_scope() {
        let coords = ArtifactCoordinates::new("com.example", "my-app", "1.0.0").unwrap();
        let mut project = MavenProject::new(coords, "/tmp/project").unwrap();
        
        let compile_dep = MavenDependency::new(
            ArtifactCoordinates::new("com.google.guava", "guava", "32.0").unwrap(),
            Scope::Compile
        );
        let test_dep = MavenDependency::new(
            ArtifactCoordinates::new("org.junit", "junit", "4.13").unwrap(),
            Scope::Test
        );
        
        project.add_dependency(compile_dep).unwrap();
        project.add_dependency(test_dep).unwrap();
        
        assert_eq!(project.compile_dependencies().len(), 1);
        assert_eq!(project.test_dependencies().len(), 1);
    }
}
