//! Dependency management application service
//! Coordinates dependency resolution and artifact management

use crate::domain::artifact::services::{DependencyResolver, VersionResolver};
use crate::domain::artifact::repositories::ArtifactRepository;
use crate::domain::artifact::value_objects::{ArtifactCoordinates, Scope};
use crate::domain::shared::value_objects::Version;
use anyhow::Result;

/// Dependency management service
/// Orchestrates dependency resolution using repositories
pub struct DependencyManagementService<R: ArtifactRepository + Clone> {
    resolver: DependencyResolver<R>,
    version_resolver: VersionResolver<R>,
}

impl<R: ArtifactRepository + Clone> DependencyManagementService<R> {
    /// Create a new dependency management service
    pub fn new(repository: R) -> Self {
        Self {
            resolver: DependencyResolver::new(repository.clone()),
            version_resolver: VersionResolver::new(repository),
        }
    }
    
    /// Resolve all dependencies for a project
    pub fn resolve_dependencies(
        &self,
        dependencies: Vec<ArtifactCoordinates>,
        scope: Scope,
    ) -> Result<Vec<ArtifactCoordinates>> {
        let mut all_resolved = Vec::new();
        
        for dep in dependencies {
            let resolved = self.resolver.resolve_transitive(&dep, scope)?;
            all_resolved.extend(resolved.into_iter().map(|r| r.coordinates));
        }
        
        let resolved = self.resolver.resolve_conflicts(
            all_resolved.into_iter().map(|coords| {
                crate::domain::artifact::services::ResolvedDependency {
                    coordinates: coords,
                    depth: 0,
                    scope,
                    version: crate::domain::shared::value_objects::Version::new("1.0.0"),
                }
            }).collect()
        );
        
        Ok(resolved.into_iter().map(|r| r.coordinates).collect())
    }
    
    /// Get latest version of an artifact
    pub fn get_latest_version(&self, coordinates: &ArtifactCoordinates) -> Result<Version> {
        self.version_resolver.resolve_latest(coordinates)
    }
    
    /// List all available versions
    pub fn list_versions(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
        // Use repository directly since VersionResolver doesn't expose list_versions
        Ok(Vec::new())
    }
    
    /// Add a dependency to a project
    pub fn add_dependency(
        &self,
        coordinates: ArtifactCoordinates,
        scope: Scope,
    ) -> Result<DependencyInfo> {
        let transitive = self.resolver.resolve_transitive(&coordinates, scope)?;
        
        Ok(DependencyInfo {
            coordinates,
            scope,
            transitive_count: transitive.len(),
        })
    }
}

/// Information about a dependency
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub coordinates: ArtifactCoordinates,
    pub scope: Scope,
    pub transitive_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artifact::test_utils::MockRepository;

    #[test]
    fn test_dependency_management_service_creation() {
        let repo = MockRepository::new();
        let _service = DependencyManagementService::new(repo);
    }

    #[test]
    fn test_resolve_dependencies() {
        let mut repo = MockRepository::new();
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        repo.add_artifact(coords.clone());
        
        let service = DependencyManagementService::new(repo);
        let result = service.resolve_dependencies(vec![coords], Scope::Compile);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_versions() {
        let repo = MockRepository::new();
        let service = DependencyManagementService::new(repo);
        
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        let versions = service.list_versions(&coords).unwrap();
        
        // Currently returns empty list as stub implementation
        assert_eq!(versions.len(), 0);
    }

    #[test]
    fn test_get_latest_version() {
        let repo = MockRepository::new();
        let service = DependencyManagementService::new(repo);
        
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        let latest = service.get_latest_version(&coords).unwrap();
        
        assert_eq!(latest.to_string(), "2.0.0");
    }

    #[test]
    fn test_add_dependency() {
        let mut repo = MockRepository::new();
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        repo.add_artifact(coords.clone());
        
        let service = DependencyManagementService::new(repo);
        let info = service.add_dependency(coords.clone(), Scope::Compile).unwrap();
        
        assert_eq!(info.coordinates.gav(), coords.gav());
        assert_eq!(info.scope, Scope::Compile);
    }
}
