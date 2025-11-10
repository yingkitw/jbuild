use std::collections::{HashMap, HashSet};

use crate::artifact::Artifact;
use crate::model::Dependency;
use crate::resolver::DependencyResolver;

/// Transitive dependency resolver
pub struct TransitiveDependencyResolver {
    resolver: DependencyResolver,
    resolved_cache: HashMap<String, Vec<Artifact>>,
}

impl TransitiveDependencyResolver {
    pub fn new(resolver: DependencyResolver) -> Self {
        Self {
            resolver,
            resolved_cache: HashMap::new(),
        }
    }

    /// Resolve all transitive dependencies for a dependency
    pub fn resolve_transitive(
        &mut self,
        dependency: &Dependency,
    ) -> anyhow::Result<Vec<Artifact>> {
        let key = format!("{}:{}:{}", 
            dependency.group_id, 
            dependency.artifact_id,
            dependency.version.as_deref().unwrap_or("")
        );

        // Check cache
        if let Some(cached) = self.resolved_cache.get(&key) {
            return Ok(cached.clone());
        }

        let mut resolved = Vec::new();
        let mut visited = HashSet::new();

        self.resolve_recursive(dependency, &mut resolved, &mut visited)?;

        // Cache result
        self.resolved_cache.insert(key, resolved.clone());
        Ok(resolved)
    }

    fn resolve_recursive(
        &mut self,
        dependency: &Dependency,
        resolved: &mut Vec<Artifact>,
        visited: &mut HashSet<String>,
    ) -> anyhow::Result<()> {
        let dep_key = format!("{}:{}", dependency.group_id, dependency.artifact_id);
        
        if visited.contains(&dep_key) {
            return Ok(());
        }
        visited.insert(dep_key.clone());

        // Resolve the dependency itself
        if let Some(artifact) = self.resolver.resolve_dependency(dependency)? {
            resolved.push(artifact.clone());

            // TODO: Load POM for this artifact to get its dependencies
            // For now, we'll just resolve the direct dependency
            // In a full implementation, we would:
            // 1. Download the POM for this artifact
            // 2. Parse it to get dependencies
            // 3. Recursively resolve those dependencies
        }

        Ok(())
    }

    /// Resolve all transitive dependencies for a list of dependencies
    pub fn resolve_all_transitive(
        &mut self,
        dependencies: &[Dependency],
    ) -> anyhow::Result<Vec<Artifact>> {
        let mut all_resolved = Vec::new();
        let mut seen = HashSet::new();

        for dependency in dependencies {
            let transitive = self.resolve_transitive(dependency)?;
            for artifact in transitive {
                let artifact_key = artifact.coordinates.full_id();
                if !seen.contains(&artifact_key) {
                    seen.insert(artifact_key);
                    all_resolved.push(artifact);
                }
            }
        }

        Ok(all_resolved)
    }
}

