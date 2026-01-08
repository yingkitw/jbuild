//! Gradle Configuration
//!
//! Implements Gradle's configuration model for dependency management.
//! Configurations are named sets of dependencies with specific purposes.

use std::collections::HashMap;

/// Dependency configuration (e.g., implementation, testImplementation)
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Configuration name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Whether this configuration can be consumed by other projects
    pub can_be_consumed: bool,
    /// Whether this configuration can be resolved
    pub can_be_resolved: bool,
    /// Configurations this extends from
    pub extends_from: Vec<String>,
    /// Dependencies in this configuration
    pub dependencies: Vec<ConfigurationDependency>,
}

/// A dependency within a configuration
#[derive(Debug, Clone)]
pub struct ConfigurationDependency {
    /// Dependency notation (group:artifact:version)
    pub notation: String,
    /// Parsed group
    pub group: Option<String>,
    /// Parsed artifact/name
    pub name: Option<String>,
    /// Parsed version
    pub version: Option<String>,
    /// Whether this is a project dependency
    pub is_project: bool,
    /// Project path if this is a project dependency
    pub project_path: Option<String>,
}

impl Configuration {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            can_be_consumed: true,
            can_be_resolved: true,
            extends_from: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn extends_from(mut self, config: impl Into<String>) -> Self {
        self.extends_from.push(config.into());
        self
    }

    pub fn consumable(mut self, value: bool) -> Self {
        self.can_be_consumed = value;
        self
    }

    pub fn resolvable(mut self, value: bool) -> Self {
        self.can_be_resolved = value;
        self
    }

    pub fn add_dependency(&mut self, dep: ConfigurationDependency) {
        self.dependencies.push(dep);
    }
}

impl ConfigurationDependency {
    /// Parse a dependency notation like "group:artifact:version"
    pub fn from_notation(notation: impl Into<String>) -> Self {
        let notation = notation.into();

        // Check if it's a project dependency
        if notation.starts_with("project(") || notation.starts_with(":") {
            let project_path = notation
                .trim_start_matches("project(")
                .trim_end_matches(')')
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();

            return Self {
                notation: notation.clone(),
                group: None,
                name: None,
                version: None,
                is_project: true,
                project_path: Some(project_path),
            };
        }

        // Parse GAV notation
        let parts: Vec<&str> = notation.split(':').collect();
        let (group, name, version) = match parts.len() {
            3 => (Some(parts[0].to_string()), Some(parts[1].to_string()), Some(parts[2].to_string())),
            2 => (Some(parts[0].to_string()), Some(parts[1].to_string()), None),
            1 => (None, Some(parts[0].to_string()), None),
            _ => (None, None, None),
        };

        Self {
            notation,
            group,
            name,
            version,
            is_project: false,
            project_path: None,
        }
    }

    /// Create a project dependency
    pub fn project(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            notation: format!("project('{path}')"),
            group: None,
            name: None,
            version: None,
            is_project: true,
            project_path: Some(path),
        }
    }
}

/// Configuration container - manages all configurations for a project
#[derive(Debug, Default)]
pub struct ConfigurationContainer {
    configurations: HashMap<String, Configuration>,
}

impl ConfigurationContainer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create standard Java configurations
    pub fn with_java_defaults() -> Self {
        let mut container = Self::new();

        // API configuration (java-library)
        container.add(Configuration::new("api")
            .with_description("API dependencies for this library")
            .consumable(true)
            .resolvable(false));

        // Implementation configuration
        container.add(Configuration::new("implementation")
            .with_description("Implementation only dependencies")
            .extends_from("api")
            .consumable(false)
            .resolvable(false));

        // Compile classpath (resolved)
        container.add(Configuration::new("compileClasspath")
            .with_description("Compile classpath for source set 'main'")
            .extends_from("implementation")
            .consumable(false)
            .resolvable(true));

        // Runtime classpath (resolved)
        container.add(Configuration::new("runtimeClasspath")
            .with_description("Runtime classpath for source set 'main'")
            .extends_from("implementation")
            .consumable(false)
            .resolvable(true));

        // Test implementation
        container.add(Configuration::new("testImplementation")
            .with_description("Implementation only dependencies for tests")
            .extends_from("implementation")
            .consumable(false)
            .resolvable(false));

        // Test compile classpath
        container.add(Configuration::new("testCompileClasspath")
            .with_description("Compile classpath for source set 'test'")
            .extends_from("testImplementation")
            .extends_from("compileClasspath")
            .consumable(false)
            .resolvable(true));

        // Test runtime classpath
        container.add(Configuration::new("testRuntimeClasspath")
            .with_description("Runtime classpath for source set 'test'")
            .extends_from("testImplementation")
            .extends_from("runtimeClasspath")
            .consumable(false)
            .resolvable(true));

        // Compile only
        container.add(Configuration::new("compileOnly")
            .with_description("Compile only dependencies")
            .consumable(false)
            .resolvable(false));

        // Runtime only
        container.add(Configuration::new("runtimeOnly")
            .with_description("Runtime only dependencies")
            .consumable(false)
            .resolvable(false));

        // Annotation processor
        container.add(Configuration::new("annotationProcessor")
            .with_description("Annotation processors and their dependencies")
            .consumable(false)
            .resolvable(true));

        container
    }

    /// Add a configuration
    pub fn add(&mut self, config: Configuration) {
        self.configurations.insert(config.name.clone(), config);
    }

    /// Get a configuration by name
    pub fn get(&self, name: &str) -> Option<&Configuration> {
        self.configurations.get(name)
    }

    /// Get a mutable configuration by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Configuration> {
        self.configurations.get_mut(name)
    }

    /// Check if a configuration exists
    pub fn has(&self, name: &str) -> bool {
        self.configurations.contains_key(name)
    }

    /// Get all configuration names
    pub fn names(&self) -> Vec<String> {
        self.configurations.keys().cloned().collect()
    }

    /// Resolve all dependencies for a configuration (including extended configs)
    pub fn resolve_dependencies(&self, name: &str) -> Vec<ConfigurationDependency> {
        let mut deps = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_dependencies(name, &mut deps, &mut visited);
        deps
    }

    fn collect_dependencies(
        &self,
        name: &str,
        deps: &mut Vec<ConfigurationDependency>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(config) = self.configurations.get(name) {
            // Add dependencies from extended configurations first
            for extended in &config.extends_from {
                self.collect_dependencies(extended, deps, visited);
            }

            // Add this configuration's dependencies
            deps.extend(config.dependencies.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_creation() {
        let config = Configuration::new("implementation")
            .with_description("Implementation dependencies")
            .consumable(false)
            .resolvable(false);

        assert_eq!(config.name, "implementation");
        assert!(!config.can_be_consumed);
        assert!(!config.can_be_resolved);
    }

    #[test]
    fn test_configuration_extends() {
        let config = Configuration::new("testImplementation")
            .extends_from("implementation");

        assert_eq!(config.extends_from, vec!["implementation".to_string()]);
    }

    #[test]
    fn test_dependency_from_notation() {
        let dep = ConfigurationDependency::from_notation("org.junit:junit:4.13.2");

        assert_eq!(dep.group, Some("org.junit".to_string()));
        assert_eq!(dep.name, Some("junit".to_string()));
        assert_eq!(dep.version, Some("4.13.2".to_string()));
        assert!(!dep.is_project);
    }

    #[test]
    fn test_project_dependency() {
        let dep = ConfigurationDependency::project(":core");

        assert!(dep.is_project);
        assert_eq!(dep.project_path, Some(":core".to_string()));
    }

    #[test]
    fn test_configuration_container_java_defaults() {
        let container = ConfigurationContainer::with_java_defaults();

        assert!(container.has("implementation"));
        assert!(container.has("testImplementation"));
        assert!(container.has("compileClasspath"));
        assert!(container.has("runtimeClasspath"));
        assert!(container.has("api"));
    }

    #[test]
    fn test_resolve_dependencies() {
        let mut container = ConfigurationContainer::new();

        let mut api = Configuration::new("api");
        api.add_dependency(ConfigurationDependency::from_notation("com.google:guava:31.0"));
        container.add(api);

        let mut impl_config = Configuration::new("implementation").extends_from("api");
        impl_config.add_dependency(ConfigurationDependency::from_notation("org.slf4j:slf4j-api:2.0.0"));
        container.add(impl_config);

        let deps = container.resolve_dependencies("implementation");

        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.notation.contains("guava")));
        assert!(deps.iter().any(|d| d.notation.contains("slf4j")));
    }
}
