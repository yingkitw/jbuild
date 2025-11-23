use crate::core::{MavenProject, MavenSession};
use crate::error::MavenResult;
use std::path::Path;

/// Trait for building Maven projects from POM files
/// Enables dependency injection and testing with mock implementations
pub trait ProjectBuildStrategy {
    /// Build a single project from a POM file
    fn build(&self, pom_file: &Path) -> MavenResult<MavenProject>;

    /// Build a reactor (multi-module project) from a POM file
    fn build_reactor(&self, pom_file: &Path) -> MavenResult<Vec<MavenProject>>;
}

/// Trait for executing lifecycle phases
/// Enables testing with mock implementations
pub trait LifecycleExecutionStrategy {
    /// Execute lifecycle phases for a project
    fn execute(
        &self,
        session: &MavenSession,
        goals: &[String],
    ) -> MavenResult<LifecycleExecutionResult>;
}

/// Result of lifecycle execution
#[derive(Debug, Clone)]
pub struct LifecycleExecutionResult {
    pub success: bool,
    pub exceptions: Vec<String>,
}

impl LifecycleExecutionResult {
    pub fn success() -> Self {
        Self {
            success: true,
            exceptions: vec![],
        }
    }

    pub fn failure(reason: String) -> Self {
        Self {
            success: false,
            exceptions: vec![reason],
        }
    }

    pub fn add_exception(&mut self, exception: String) {
        self.exceptions.push(exception);
        self.success = false;
    }
}

/// Trait for resolving dependencies
/// Enables testing with mock repositories
pub trait DependencyResolutionStrategy {
    /// Resolve a single dependency
    fn resolve_dependency(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> MavenResult<Option<String>>;

    /// Resolve multiple dependencies
    fn resolve_dependencies(
        &self,
        dependencies: &[(String, String, String)],
    ) -> MavenResult<Vec<String>>;
}

/// Trait for artifact repository operations
/// Enables testing with in-memory repositories
pub trait ArtifactRepository: Send + Sync {
    /// Check if artifact exists
    fn exists(&self, group_id: &str, artifact_id: &str, version: &str) -> bool;

    /// Get artifact path
    fn get_path(&self, group_id: &str, artifact_id: &str, version: &str) -> MavenResult<String>;

    /// Store artifact
    fn store(&self, group_id: &str, artifact_id: &str, version: &str, path: &Path) -> MavenResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_execution_result_success() {
        let result = LifecycleExecutionResult::success();
        assert!(result.success);
        assert!(result.exceptions.is_empty());
    }

    #[test]
    fn test_lifecycle_execution_result_failure() {
        let result = LifecycleExecutionResult::failure("Test error".to_string());
        assert!(!result.success);
        assert_eq!(result.exceptions.len(), 1);
    }

    #[test]
    fn test_lifecycle_execution_result_add_exception() {
        let mut result = LifecycleExecutionResult::success();
        result.add_exception("Error 1".to_string());
        assert!(!result.success);
        assert_eq!(result.exceptions.len(), 1);
    }
}
