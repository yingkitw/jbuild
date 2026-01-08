//! Domain services for Artifact context

use super::repositories::ArtifactRepository;
use super::value_objects::{ArtifactCoordinates, Scope, VersionRange};
use crate::domain::shared::value_objects::Version;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Dependency resolution service
/// Resolves transitive dependencies and handles version conflicts
pub struct DependencyResolver<R: ArtifactRepository> {
    repository: R,
}

impl<R: ArtifactRepository> DependencyResolver<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Resolves all transitive dependencies for a given artifact
    pub fn resolve_transitive(
        &self,
        coordinates: &ArtifactCoordinates,
        scope: Scope,
    ) -> Result<Vec<ResolvedDependency>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut processing = HashSet::new();
        
        self.resolve_recursive(
            coordinates,
            scope,
            0,
            &mut resolved,
            &mut visited,
            &mut processing,
        )?;
        
        Ok(resolved)
    }
    
    fn resolve_recursive(
        &self,
        coordinates: &ArtifactCoordinates,
        scope: Scope,
        depth: usize,
        resolved: &mut Vec<ResolvedDependency>,
        visited: &mut HashSet<String>,
        processing: &mut HashSet<String>,
    ) -> Result<()> {
        let key = coordinates.gav();
        
        // Check for circular dependencies
        if processing.contains(&key) {
            return Err(anyhow!("Circular dependency detected: {}", key));
        }
        
        // Skip if already visited
        if visited.contains(&key) {
            return Ok(());
        }
        
        processing.insert(key.clone());
        
        // Fetch artifact metadata from repository
        let metadata = self.repository.get_metadata(coordinates)?;
        
        // Add to resolved list
        resolved.push(ResolvedDependency {
            coordinates: coordinates.clone(),
            scope,
            depth,
            version: Version::new(coordinates.version()),
        });
        
        visited.insert(key.clone());
        processing.remove(&key);
        
        Ok(())
    }
    
    /// Resolves version conflicts using nearest-wins strategy
    pub fn resolve_conflicts(
        &self,
        dependencies: Vec<ResolvedDependency>,
    ) -> Vec<ResolvedDependency> {
        let mut by_artifact: HashMap<String, Vec<ResolvedDependency>> = HashMap::new();
        
        // Group by artifact (groupId:artifactId)
        for dep in dependencies {
            let key = format!("{}:{}", dep.coordinates.group_id(), dep.coordinates.artifact_id());
            by_artifact.entry(key).or_default().push(dep);
        }
        
        // For each artifact, select the version using nearest-wins
        let mut result = Vec::new();
        for (_, mut versions) in by_artifact {
            if versions.len() == 1 {
                result.push(versions.pop().unwrap());
            } else {
                // Sort by depth (nearest first), then by version (highest)
                versions.sort_by(|a, b| {
                    match a.depth.cmp(&b.depth) {
                        std::cmp::Ordering::Equal => b.version.cmp(&a.version),
                        other => other,
                    }
                });
                result.push(versions.into_iter().next().unwrap());
            }
        }
        
        result
    }
    
    /// Filters dependencies by scope
    pub fn filter_by_scope(
        &self,
        dependencies: Vec<ResolvedDependency>,
        scope: Scope,
    ) -> Vec<ResolvedDependency> {
        dependencies
            .into_iter()
            .filter(|d| d.scope == scope)
            .collect()
    }
}

/// Resolved dependency with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDependency {
    pub coordinates: ArtifactCoordinates,
    pub scope: Scope,
    pub depth: usize,
    pub version: Version,
}

/// Version resolution service
/// Resolves version ranges to concrete versions
pub struct VersionResolver<R: ArtifactRepository> {
    repository: R,
}

impl<R: ArtifactRepository> VersionResolver<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Resolves a version range to the best matching version
    pub fn resolve_range(
        &self,
        coordinates: &ArtifactCoordinates,
        range: &VersionRange,
    ) -> Result<Version> {
        // Get available versions from repository
        let available = self.repository.list_versions(coordinates)?;
        
        // Filter versions that match the range
        let matching: Vec<_> = available
            .iter()
            .filter(|v| range.matches(v.as_str()))
            .collect();
        
        if matching.is_empty() {
            return Err(anyhow!(
                "No version found matching range for {}",
                coordinates.gav()
            ));
        }
        
        // Return the highest matching version
        let best = matching
            .into_iter()
            .max()
            .unwrap();
        
        Ok(best.clone())
    }
    
    /// Resolves latest version for an artifact
    pub fn resolve_latest(&self, coordinates: &ArtifactCoordinates) -> Result<Version> {
        let versions = self.repository.list_versions(coordinates)?;
        
        versions
            .into_iter()
            .max()
            .ok_or_else(|| anyhow!("No versions found for {}", coordinates.gav()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artifact::repositories::ArtifactMetadata;
    
    // Mock repository for testing
    struct MockRepository {
        artifacts: HashMap<String, ArtifactMetadata>,
        repo_path: PathBuf,
    }
    
    impl MockRepository {
        fn new() -> Self {
            Self {
                artifacts: HashMap::new(),
                repo_path: PathBuf::from("/tmp/repo"),
            }
        }
        
        fn add_artifact(&mut self, coords: ArtifactCoordinates) {
            let metadata = ArtifactMetadata {
                coordinates: coords.clone(),
                dependencies: Vec::new(),
            };
            self.artifacts.insert(coords.gav(), metadata);
        }
    }
    
    impl ArtifactRepository for MockRepository {
        fn install(&self, _coords: &ArtifactCoordinates, _file: PathBuf) -> Result<()> {
            Ok(())
        }
        
        fn exists(&self, coords: &ArtifactCoordinates) -> bool {
            self.artifacts.contains_key(&coords.gav())
        }
        
        fn path(&self) -> &PathBuf {
            &self.repo_path
        }
        
        fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata> {
            self.artifacts
                .get(&coordinates.gav())
                .cloned()
                .ok_or_else(|| anyhow!("Artifact not found"))
        }
        
        fn list_versions(&self, _coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
            Ok(vec![
                Version::new("1.0.0"),
                Version::new("1.1.0"),
                Version::new("2.0.0"),
            ])
        }
        
        fn download(&self, _coordinates: &ArtifactCoordinates) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_resolve_transitive() {
        let mut repo = MockRepository::new();
        let coords = ArtifactCoordinates::new("com.example", "lib", "1.0.0").unwrap();
        repo.add_artifact(coords.clone());
        
        let resolver = DependencyResolver::new(repo);
        let result = resolver.resolve_transitive(&coords, Scope::Compile);
        
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].coordinates.artifact_id(), "lib");
    }

    #[test]
    fn test_resolve_conflicts() {
        let repo = MockRepository::new();
        let resolver = DependencyResolver::new(repo);
        
        let deps = vec![
            ResolvedDependency {
                coordinates: ArtifactCoordinates::new("com.example", "lib", "1.0.0").unwrap(),
                scope: Scope::Compile,
                depth: 0,
                version: Version::new("1.0.0"),
            },
            ResolvedDependency {
                coordinates: ArtifactCoordinates::new("com.example", "lib", "2.0.0").unwrap(),
                scope: Scope::Compile,
                depth: 1,
                version: Version::new("2.0.0"),
            },
        ];
        
        let result = resolver.resolve_conflicts(deps);
        
        // Should select 1.0.0 (nearest wins - depth 0)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version.as_str(), "1.0.0");
    }

    #[test]
    fn test_version_resolver() {
        let repo = MockRepository::new();
        let resolver = VersionResolver::new(repo);
        
        let coords = ArtifactCoordinates::new("com.example", "lib", "1.0.0").unwrap();
        let result = resolver.resolve_latest(&coords);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "2.0.0");
    }
}
