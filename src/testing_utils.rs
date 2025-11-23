/// Testing utilities and mock implementations for unit testing
/// This module provides test fixtures and mock implementations to support
/// isolated unit testing without external dependencies

use crate::core::traits::{ArtifactRepository, DependencyResolutionStrategy};
use crate::error::MavenResult;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// In-memory artifact repository for testing
/// Allows tests to verify repository operations without file system access
#[derive(Debug, Clone)]
pub struct MockArtifactRepository {
    artifacts: std::sync::Arc<Mutex<HashMap<String, String>>>,
}

impl MockArtifactRepository {
    /// Create a new mock repository
    pub fn new() -> Self {
        Self {
            artifacts: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add an artifact to the mock repository
    pub fn add_artifact(&self, group_id: &str, artifact_id: &str, version: &str, path: String) {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.artifacts.lock().unwrap().insert(key, path);
    }

    /// Get all stored artifacts (for verification in tests)
    pub fn get_all_artifacts(&self) -> HashMap<String, String> {
        self.artifacts.lock().unwrap().clone()
    }
}

impl Default for MockArtifactRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactRepository for MockArtifactRepository {
    fn exists(&self, group_id: &str, artifact_id: &str, version: &str) -> bool {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.artifacts.lock().unwrap().contains_key(&key)
    }

    fn get_path(&self, group_id: &str, artifact_id: &str, version: &str) -> MavenResult<String> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.artifacts
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| {
                crate::error::MavenError::RepositoryError(format!(
                    "Artifact not found: {}:{}:{}",
                    group_id, artifact_id, version
                ))
            })
    }

    fn store(&self, group_id: &str, artifact_id: &str, version: &str, path: &Path) -> MavenResult<()> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.artifacts
            .lock()
            .unwrap()
            .insert(key, path.to_string_lossy().to_string());
        Ok(())
    }
}

/// Mock dependency resolver for testing
/// Allows tests to control dependency resolution behavior
#[derive(Debug, Clone)]
pub struct MockDependencyResolver {
    resolutions: std::sync::Arc<Mutex<HashMap<String, Option<String>>>>,
}

impl MockDependencyResolver {
    /// Create a new mock resolver
    pub fn new() -> Self {
        Self {
            resolutions: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a successful resolution
    pub fn register_resolution(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        path: String,
    ) {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.resolutions.lock().unwrap().insert(key, Some(path));
    }

    /// Register a failed resolution
    pub fn register_missing(&self, group_id: &str, artifact_id: &str, version: &str) {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.resolutions.lock().unwrap().insert(key, None);
    }

    /// Get all registered resolutions (for verification in tests)
    pub fn get_all_resolutions(&self) -> HashMap<String, Option<String>> {
        self.resolutions.lock().unwrap().clone()
    }
}

impl Default for MockDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolutionStrategy for MockDependencyResolver {
    fn resolve_dependency(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> MavenResult<Option<String>> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        Ok(self.resolutions.lock().unwrap().get(&key).cloned().flatten())
    }

    fn resolve_dependencies(
        &self,
        dependencies: &[(String, String, String)],
    ) -> MavenResult<Vec<String>> {
        let mut resolved = Vec::new();
        for (group_id, artifact_id, version) in dependencies {
            if let Some(path) = self.resolve_dependency(group_id, artifact_id, version)? {
                resolved.push(path);
            }
        }
        Ok(resolved)
    }
}

/// Test fixture builder for creating test projects
#[derive(Debug, Clone)]
pub struct TestProjectBuilder {
    group_id: String,
    artifact_id: String,
    version: String,
    packaging: String,
}

impl TestProjectBuilder {
    /// Create a new test project builder
    pub fn new() -> Self {
        Self {
            group_id: "com.example".to_string(),
            artifact_id: "test-project".to_string(),
            version: "1.0.0".to_string(),
            packaging: "jar".to_string(),
        }
    }

    /// Set group ID
    pub fn with_group_id(mut self, group_id: String) -> Self {
        self.group_id = group_id;
        self
    }

    /// Set artifact ID
    pub fn with_artifact_id(mut self, artifact_id: String) -> Self {
        self.artifact_id = artifact_id;
        self
    }

    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    /// Set packaging
    pub fn with_packaging(mut self, packaging: String) -> Self {
        self.packaging = packaging;
        self
    }

    /// Get the group ID
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    /// Get the artifact ID
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the packaging
    pub fn packaging(&self) -> &str {
        &self.packaging
    }
}

impl Default for TestProjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_artifact_repository() {
        let repo = MockArtifactRepository::new();
        repo.add_artifact("com.example", "lib", "1.0.0", "/path/to/lib.jar".to_string());

        assert!(repo.exists("com.example", "lib", "1.0.0"));
        assert!(!repo.exists("com.example", "other", "1.0.0"));

        let path = repo.get_path("com.example", "lib", "1.0.0").unwrap();
        assert_eq!(path, "/path/to/lib.jar");
    }

    #[test]
    fn test_mock_dependency_resolver() {
        let resolver = MockDependencyResolver::new();
        resolver.register_resolution("com.example", "lib", "1.0.0", "/path/to/lib.jar".to_string());
        resolver.register_missing("com.example", "missing", "1.0.0");

        let result = resolver
            .resolve_dependency("com.example", "lib", "1.0.0")
            .unwrap();
        assert_eq!(result, Some("/path/to/lib.jar".to_string()));

        let missing = resolver
            .resolve_dependency("com.example", "missing", "1.0.0")
            .unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_test_project_builder() {
        let project = TestProjectBuilder::new()
            .with_group_id("org.example".to_string())
            .with_artifact_id("my-app".to_string())
            .with_version("2.0.0".to_string());

        assert_eq!(project.group_id(), "org.example");
        assert_eq!(project.artifact_id(), "my-app");
        assert_eq!(project.version(), "2.0.0");
    }
}
