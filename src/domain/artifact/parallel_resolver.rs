//! Parallel dependency resolver using rayon for concurrent resolution

use rayon::prelude::*;
use super::repositories::ArtifactRepository;
use super::value_objects::{ArtifactCoordinates, Scope};
use super::services::ResolvedDependency;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Parallel dependency resolver with concurrent artifact resolution
pub struct ParallelDependencyResolver<R: ArtifactRepository> {
    repository: R,
    max_concurrency: usize,
}

impl<R: ArtifactRepository + Clone> ParallelDependencyResolver<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            max_concurrency: 8, // Default concurrency
        }
    }

    pub fn with_concurrency(repository: R, max_concurrency: usize) -> Self {
        Self {
            repository,
            max_concurrency,
        }
    }

    /// Resolve multiple dependencies in parallel
    pub fn resolve_all_parallel(
        &self,
        dependencies: Vec<ArtifactCoordinates>,
        scope: Scope,
    ) -> Result<Vec<ResolvedDependency>> {
        let resolved = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Use rayon's parallel iterator with custom thread pool
        dependencies.par_iter().for_each(|coords| {
            match self.resolve_single(coords, scope) {
                Ok(dep) => {
                    if let Ok(mut resolved_guard) = resolved.lock() {
                        resolved_guard.push(dep);
                    }
                }
                Err(e) => {
                    if let Ok(mut errors_guard) = errors.lock() {
                        errors_guard.push((coords.clone(), e));
                    }
                }
            }
        });

        let resolved = resolved.lock().map(|g| g.clone()).unwrap_or_default();
        
        // Check for errors without cloning
        let has_errors = {
            let errors_guard = errors.lock().unwrap();
            !errors_guard.is_empty()
        };

        if has_errors {
            let error_msg = {
                let errors_guard = errors.lock().unwrap();
                errors_guard
                    .iter()
                    .map(|(coords, e)| format!("{}: {}", coords.gav(), e))
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            return Err(anyhow!("Failed to resolve dependencies: {}", error_msg));
        }

        Ok(resolved)
    }

    /// Resolve transitive dependencies in parallel with depth-first optimization
    pub fn resolve_transitive_parallel(
        &self,
        root: &ArtifactCoordinates,
        scope: Scope,
    ) -> Result<Vec<ResolvedDependency>> {
        let visited = Arc::new(Mutex::new(HashSet::new()));
        let resolved = Arc::new(Mutex::new(Vec::new()));
        let queue = Arc::new(Mutex::new(vec![root.clone()]));

        loop {
            let batch: Vec<ArtifactCoordinates> = {
                let mut queue_guard = queue.lock().unwrap();
                if queue_guard.is_empty() {
                    break;
                }
                queue_guard.drain(..).collect()
            };

            let visited_clone = Arc::clone(&visited);
            
            let new_resolved: Vec<ResolvedDependency> = batch
                .par_iter()
                .filter_map(|coords| {
                    let key = coords.gav();
                    
                    // Check and mark as visited atomically
                    let should_process = {
                        let mut visited_guard = visited_clone.lock().unwrap();
                        if visited_guard.contains(&key) {
                            false
                        } else {
                            visited_guard.insert(key.clone());
                            true
                        }
                    };
                    
                    if !should_process {
                        return None;
                    }

                    self.resolve_single(coords, scope).ok()
                })
                .collect();

            // Add transitive dependencies to queue
            for dep in &new_resolved {
                if let Ok(metadata) = self.repository.get_metadata(&dep.coordinates) {
                    let mut queue_guard = queue.lock().unwrap();
                    let visited_guard = visited.lock().unwrap();
                    
                    for transitive_dep in metadata.dependencies {
                        let dep_key = transitive_dep.gav();
                        if !visited_guard.contains(&dep_key) {
                            queue_guard.push(transitive_dep);
                        }
                    }
                }
            }

            resolved.lock().unwrap().extend(new_resolved);
        }

        let result = resolved.lock().unwrap().clone();
        Ok(result)
    }

    /// Batch resolve artifacts with metadata prefetching
    pub fn batch_resolve_with_metadata(
        &self,
        dependencies: Vec<ArtifactCoordinates>,
    ) -> Result<BatchResolution> {
        // Fetch all metadata in parallel
        let metadata_map: HashMap<String, ArtifactMetadata> = dependencies
            .par_iter()
            .filter_map(|coords| {
                self.repository
                    .get_metadata(coords)
                    .ok()
                    .map(|meta| (coords.gav(), ArtifactMetadata::from(meta)))
            })
            .collect();

        // Build dependency graph
        let graph = self.build_dependency_graph(&dependencies, &metadata_map)?;

        Ok(BatchResolution {
            dependencies,
            metadata: metadata_map,
            graph,
        })
    }

    fn resolve_single(
        &self,
        coordinates: &ArtifactCoordinates,
        scope: Scope,
    ) -> Result<ResolvedDependency> {
        // Get metadata from repository
        let _metadata = self.repository.get_metadata(coordinates)?;

        Ok(ResolvedDependency {
            coordinates: coordinates.clone(),
            scope,
            depth: 0,
            version: crate::domain::shared::value_objects::Version::new(coordinates.version()),
        })
    }

    fn build_dependency_graph(
        &self,
        dependencies: &[ArtifactCoordinates],
        metadata_map: &HashMap<String, ArtifactMetadata>,
    ) -> Result<DependencyGraph> {
        let mut graph = HashMap::new();

        for dep in dependencies {
            let key = dep.gav();
            let metadata = metadata_map.get(&key);

            let deps: Vec<String> = metadata
                .and_then(|m| Some(m.dependencies.clone()))
                .unwrap_or_default();

            graph.insert(key, deps);
        }

        Ok(DependencyGraph { graph })
    }
}

/// Batch resolution result with metadata and dependency graph
pub struct BatchResolution {
    pub dependencies: Vec<ArtifactCoordinates>,
    pub metadata: HashMap<String, ArtifactMetadata>,
    pub graph: DependencyGraph,
}

/// Artifact metadata summary
#[derive(Debug, Clone)]
pub struct ArtifactMetadata {
    pub dependencies: Vec<String>,
    pub version: String,
}

impl From<crate::domain::artifact::repositories::ArtifactMetadata> for ArtifactMetadata {
    fn from(meta: crate::domain::artifact::repositories::ArtifactMetadata) -> Self {
        let dependencies = meta.dependencies.iter().map(|d| d.gav()).collect();
        Self {
            dependencies,
            version: meta.coordinates.version().to_string(),
        }
    }
}

/// Dependency graph for transitive resolution
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    graph: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Get direct dependencies of an artifact
    pub fn dependencies(&self, artifact_id: &str) -> Vec<String> {
        self.graph.get(artifact_id).cloned().unwrap_or_default()
    }

    /// Calculate build levels (BFS levels from roots)
    pub fn calculate_levels(&self) -> HashMap<String, usize> {
        let mut levels = HashMap::new();
        let mut queue = Vec::new();

        // Find roots (no dependencies)
        for (artifact, deps) in &self.graph {
            if deps.is_empty() {
                levels.insert(artifact.clone(), 0);
                queue.push(artifact.clone());
            }
        }

        // BFS to calculate levels
        while let Some(current) = queue.pop() {
            let current_level = levels.get(&current).copied().unwrap_or(0);

            // Find artifacts that depend on current
            for (artifact, deps) in &self.graph {
                if deps.contains(&current) {
                    let new_level = current_level + 1;
                    let entry = levels.entry(artifact.clone()).or_insert(usize::MAX);
                    if new_level < *entry {
                        *entry = new_level;
                        queue.push(artifact.clone());
                    }
                }
            }
        }

        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_levels() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec![]);
        graph.insert("b".to_string(), vec!["a".to_string()]);
        graph.insert("c".to_string(), vec!["a".to_string()]);
        graph.insert("d".to_string(), vec!["b".to_string(), "c".to_string()]);

        let dep_graph = DependencyGraph { graph };
        let levels = dep_graph.calculate_levels();

        assert_eq!(levels.get("a"), Some(&0));
        assert_eq!(levels.get("b"), Some(&1));
        assert_eq!(levels.get("c"), Some(&1));
        assert_eq!(levels.get("d"), Some(&2));
    }
}
