//! Parallel dependency resolution for improved performance
//!
//! Uses rayon for parallel processing of dependency resolution

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use anyhow::Result;

use crate::artifact::Artifact;
use crate::model::Dependency;
use crate::resolver::DependencyResolver;

/// Parallel dependency resolver
pub struct ParallelDependencyResolver {
    resolver: Arc<DependencyResolver>,
    max_parallel: usize,
}

impl ParallelDependencyResolver {
    pub fn new(resolver: DependencyResolver) -> Self {
        Self {
            resolver: Arc::new(resolver),
            max_parallel: num_cpus::get(),
        }
    }

    pub fn with_max_parallel(mut self, max: usize) -> Self {
        self.max_parallel = max;
        self
    }

    /// Resolve dependencies in parallel
    pub fn resolve_parallel(&self, dependencies: &[Dependency]) -> Result<Vec<Artifact>> {
        let resolved = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::new(Mutex::new(HashSet::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Configure rayon thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_parallel)
            .build()
            .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

        pool.install(|| {
            dependencies.par_iter().for_each(|dep| {
                let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                
                // Check if already processed
                {
                    let mut seen_lock = seen.lock().unwrap();
                    if seen_lock.contains(&key) {
                        return;
                    }
                    seen_lock.insert(key.clone());
                }

                // Resolve dependency
                match self.resolver.resolve_dependency(dep) {
                    Ok(Some(artifact)) => {
                        resolved.lock().unwrap().push(artifact);
                    }
                    Ok(None) => {
                        // Dependency not found, log but continue
                        tracing::warn!("Dependency not found: {}", key);
                    }
                    Err(e) => {
                        errors.lock().unwrap().push((key, e));
                    }
                }
            });
        });

        // Check for errors
        let errors = errors.lock().unwrap();
        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(k, e)| format!("{}: {}", k, e))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow::anyhow!("Failed to resolve dependencies: {}", error_msg));
        }

        Arc::try_unwrap(resolved)
            .map(|mutex| Ok(mutex.into_inner().unwrap()))
            .unwrap_or_else(|arc| Ok(arc.lock().unwrap().clone()))
    }

    /// Resolve transitive dependencies in parallel
    pub fn resolve_transitive_parallel(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Artifact>> {
        let all_resolved = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::new(Mutex::new(HashSet::new()));
        let to_process = Arc::new(Mutex::new(dependencies.to_vec()));

        while !to_process.lock().unwrap().is_empty() {
            let current_batch: Vec<Dependency> = {
                let mut to_process_lock = to_process.lock().unwrap();
                to_process_lock.drain(..).collect()
            };

            let new_deps = Arc::new(Mutex::new(Vec::new()));

            current_batch.par_iter().for_each(|dep| {
                let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                
                // Check if already processed
                {
                    let mut seen_lock = seen.lock().unwrap();
                    if seen_lock.contains(&key) {
                        return;
                    }
                    seen_lock.insert(key.clone());
                }

                // Resolve dependency
                if let Ok(Some(artifact)) = self.resolver.resolve_dependency(dep) {
                    all_resolved.lock().unwrap().push(artifact.clone());

                    // Get transitive dependencies
                    if let Ok(Some(model)) = self.resolver.resolve_pom(&artifact) {
                        if let Some(deps) = model.dependencies {
                            for sub_dep in deps.dependencies {
                                // Skip test/provided scope
                                if sub_dep.scope.as_deref() == Some("test") 
                                    || sub_dep.scope.as_deref() == Some("provided") {
                                    continue;
                                }
                                
                                // Skip optional dependencies
                                if sub_dep.optional == Some(true) {
                                    continue;
                                }

                                new_deps.lock().unwrap().push(sub_dep);
                            }
                        }
                    }
                }
            });

            // Add new dependencies to process
            let mut to_process_lock = to_process.lock().unwrap();
            to_process_lock.extend(new_deps.lock().unwrap().drain(..));
        }

        Arc::try_unwrap(all_resolved)
            .map(|mutex| Ok(mutex.into_inner().unwrap()))
            .unwrap_or_else(|arc| Ok(arc.lock().unwrap().clone()))
    }

    /// Batch resolve with chunking for better performance
    pub fn resolve_batched(
        &self,
        dependencies: &[Dependency],
        batch_size: usize,
    ) -> Result<Vec<Artifact>> {
        let mut all_resolved = Vec::new();
        let mut seen = HashSet::new();

        for chunk in dependencies.chunks(batch_size) {
            let chunk_resolved = self.resolve_parallel(chunk)?;
            for artifact in chunk_resolved {
                let key = artifact.coordinates.full_id();
                if !seen.contains(&key) {
                    seen.insert(key);
                    all_resolved.push(artifact);
                }
            }
        }

        Ok(all_resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::LocalRepository;

    #[test]
    fn test_parallel_resolver_creation() {
        let local_repo = crate::artifact::repository::DefaultLocalRepository::default();
        let resolver = DependencyResolver::new(Box::new(local_repo));
        let parallel_resolver = ParallelDependencyResolver::new(resolver);
        assert_eq!(parallel_resolver.max_parallel, num_cpus::get());
    }

    #[test]
    fn test_with_max_parallel() {
        let local_repo = crate::artifact::repository::DefaultLocalRepository::default();
        let resolver = DependencyResolver::new(Box::new(local_repo));
        let parallel_resolver = ParallelDependencyResolver::new(resolver).with_max_parallel(4);
        assert_eq!(parallel_resolver.max_parallel, 4);
    }
}
