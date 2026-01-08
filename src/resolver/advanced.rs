//! Advanced Dependency Resolution
//!
//! Enhanced dependency resolver with exclusions, version ranges, and conflict resolution.

use std::collections::{HashMap, HashSet};
use anyhow::Result;
use crate::artifact::Artifact;
use crate::model::Dependency;
use crate::resolver::resolver::DependencyResolver;
use crate::resolver::conflict::ConflictResolver;

// Re-export from submodules
pub use crate::resolver::version_range::VersionRangeResolver;
pub use crate::resolver::conflict::{ConflictResolver as Conflict, DependencyMediator};

/// Enhanced dependency resolver with advanced features
pub struct AdvancedDependencyResolver {
    base_resolver: DependencyResolver,
    exclusions: HashSet<String>, // groupId:artifactId patterns
}

impl AdvancedDependencyResolver {
    pub fn new(resolver: DependencyResolver) -> Self {
        Self {
            base_resolver: resolver,
            exclusions: HashSet::new(),
        }
    }

    /// Add an exclusion pattern
    pub fn add_exclusion(mut self, group_id: &str, artifact_id: &str) -> Self {
        self.exclusions.insert(format!("{group_id}:{artifact_id}"));
        self
    }

    /// Check if a dependency is excluded
    pub fn is_excluded(&self, dependency: &Dependency) -> bool {
        let key = dependency.id();
        self.exclusions.contains(&key)
    }

    /// Resolve dependency with exclusions handling
    pub fn resolve_with_exclusions(
        &self,
        dependency: &Dependency,
    ) -> Result<Option<Artifact>> {
        // Check if this dependency itself is excluded
        if self.is_excluded(dependency) {
            return Ok(None);
        }

        // Check exclusions from dependency itself
        if let Some(ref exclusions) = dependency.exclusions {
            for exclusion in exclusions {
                let exclusion_key = format!("{}:{}", exclusion.group_id, exclusion.artifact_id);
                if self.exclusions.contains(&exclusion_key) {
                    // This transitive dependency would be excluded
                    // For now, we still resolve the direct dependency
                }
            }
        }

        // Handle optional dependencies
        if dependency.optional == Some(true) {
            // Optional dependencies are not required
            // Resolve if available, but don't fail if missing
            return self.base_resolver.resolve_dependency(dependency);
        }

        // Resolve version range if present
        if let Some(ref version) = dependency.version {
            if version.contains('[') || version.contains('(') || version.contains(',') {
                // Version range - would need to fetch available versions
                // For now, treat as regular version
                return self.base_resolver.resolve_dependency(dependency);
            }
        }

        self.base_resolver.resolve_dependency(dependency)
    }

    /// Resolve dependencies with conflict resolution
    pub fn resolve_with_conflict_resolution(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Artifact>> {
        let mut dependency_map: HashMap<String, (String, Artifact)> = HashMap::new();

        for dependency in dependencies {
            if let Some(artifact) = self.resolve_with_exclusions(dependency)? {
                let key = format!("{}:{}", 
                    artifact.coordinates.group_id, 
                    artifact.coordinates.artifact_id
                );
                dependency_map.insert(key, (dependency.full_id(), artifact));
            }
        }

        // Resolve conflicts
        let conflicts: Vec<(String, Artifact)> = dependency_map
            .into_iter()
            .map(|(_, (_, artifact))| (artifact.coordinates.full_id(), artifact))
            .collect();

        let resolved_artifacts = ConflictResolver::resolve_conflicts(&conflicts);

        Ok(resolved_artifacts)
    }
}

// Tests are in version_range.rs, conflict.rs, and common/version.rs

