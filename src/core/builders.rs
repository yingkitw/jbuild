use crate::core::MavenExecutionRequest;
use std::path::PathBuf;
use std::collections::HashMap;

/// Builder for MavenExecutionRequest
/// Provides a fluent API for constructing execution requests
#[derive(Debug, Clone)]
pub struct ExecutionRequestBuilder {
    base_directory: PathBuf,
    goals: Vec<String>,
    pom_file: Option<PathBuf>,
    system_properties: HashMap<String, String>,
    active_profiles: Vec<String>,
    reactor_active: bool,
}

impl ExecutionRequestBuilder {
    /// Create a new builder with base directory
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            base_directory,
            goals: vec![],
            pom_file: None,
            system_properties: HashMap::new(),
            active_profiles: vec![],
            reactor_active: true,
        }
    }

    /// Set goals
    pub fn with_goals(mut self, goals: Vec<String>) -> Self {
        self.goals = goals;
        self
    }

    /// Add a single goal
    pub fn add_goal(mut self, goal: String) -> Self {
        self.goals.push(goal);
        self
    }

    /// Set POM file
    pub fn with_pom_file(mut self, pom_file: PathBuf) -> Self {
        self.pom_file = Some(pom_file);
        self
    }

    /// Add system property
    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.system_properties.insert(key, value);
        self
    }

    /// Add multiple system properties
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.system_properties.extend(properties);
        self
    }

    /// Activate profile
    pub fn with_profile(mut self, profile: String) -> Self {
        self.active_profiles.push(profile);
        self
    }

    /// Set reactor active
    pub fn reactor_active(mut self, active: bool) -> Self {
        self.reactor_active = active;
        self
    }

    /// Build the execution request
    pub fn build(self) -> MavenExecutionRequest {
        let mut request = MavenExecutionRequest::new(self.base_directory);
        request.goals = self.goals;
        request.pom_file = self.pom_file;
        request.system_properties = self.system_properties;
        request.active_profiles = self.active_profiles;
        request.reactor_active = self.reactor_active;
        request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_request_builder_basic() {
        let base_dir = PathBuf::from("/project");
        let request = ExecutionRequestBuilder::new(base_dir.clone())
            .with_goals(vec!["compile".to_string()])
            .build();

        assert_eq!(request.base_directory, base_dir);
        assert_eq!(request.goals, vec!["compile".to_string()]);
    }

    #[test]
    fn test_execution_request_builder_with_properties() {
        let base_dir = PathBuf::from("/project");
        let request = ExecutionRequestBuilder::new(base_dir)
            .with_property("maven.compiler.source".to_string(), "11".to_string())
            .with_property("maven.compiler.target".to_string(), "11".to_string())
            .build();

        assert_eq!(request.system_properties.len(), 2);
        assert_eq!(
            request.system_properties.get("maven.compiler.source"),
            Some(&"11".to_string())
        );
    }

    #[test]
    fn test_execution_request_builder_fluent() {
        let base_dir = PathBuf::from("/project");
        let request = ExecutionRequestBuilder::new(base_dir)
            .add_goal("clean".to_string())
            .add_goal("compile".to_string())
            .add_goal("test".to_string())
            .reactor_active(false)
            .with_profile("dev".to_string())
            .build();

        assert_eq!(request.goals.len(), 3);
        assert!(!request.reactor_active);
        assert_eq!(request.active_profiles.len(), 1);
    }
}
