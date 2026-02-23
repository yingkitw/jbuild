//! Dependency tracking for incremental builds

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Tracks dependencies between files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTracker {
    /// Map from file to its dependencies
    pub forward_deps: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Map from file to files that depend on it
    pub reverse_deps: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            forward_deps: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.forward_deps
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.reverse_deps
            .entry(to)
            .or_default()
            .insert(from);
    }

    /// Get all files that transitively depend on a file
    pub fn get_dependents(&self, file: &PathBuf) -> HashSet<PathBuf> {
        let mut dependents = HashSet::new();
        let mut queue = Vec::new();

        if let Some(direct) = self.reverse_deps.get(file) {
            queue.extend(direct.clone());
        }

        while let Some(current) = queue.pop() {
            if dependents.contains(&current) {
                continue;
            }
            dependents.insert(current.clone());

            if let Some(direct) = self.reverse_deps.get(&current) {
                queue.extend(direct.clone());
            }
        }

        dependents
    }

    /// Get all transitive dependencies of a file
    pub fn get_dependencies(&self, file: &PathBuf) -> HashSet<PathBuf> {
        let mut dependencies = HashSet::new();
        let mut queue = Vec::new();

        if let Some(direct) = self.forward_deps.get(file) {
            queue.extend(direct.clone());
        }

        while let Some(current) = queue.pop() {
            if dependencies.contains(&current) {
                continue;
            }
            dependencies.insert(current.clone());

            if let Some(direct) = self.forward_deps.get(&current) {
                queue.extend(direct.clone());
            }
        }

        dependencies
    }

    /// Remove a file and update dependencies
    pub fn remove_file(&mut self, file: &PathBuf) {
        if let Some(deps) = self.forward_deps.remove(file) {
            for dep in deps {
                if let Some(reverse) = self.reverse_deps.get_mut(&dep) {
                    reverse.remove(file);
                }
            }
        }

        if let Some(reverse) = self.reverse_deps.remove(file) {
            for rev in reverse {
                if let Some(forward) = self.forward_deps.get_mut(&rev) {
                    forward.remove(file);
                }
            }
        }
    }

    /// Clear all dependencies
    pub fn clear(&mut self) {
        self.forward_deps.clear();
        self.reverse_deps.clear();
    }
}

impl Default for DependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_tracker() {
        let mut tracker = DependencyTracker::new();

        let a = PathBuf::from("A.java");
        let b = PathBuf::from("B.java");
        let c = PathBuf::from("C.java");

        tracker.add_dependency(a.clone(), b.clone());
        tracker.add_dependency(b.clone(), c.clone());

        let deps = tracker.get_dependencies(&a);
        assert!(deps.contains(&b));
        assert!(deps.contains(&c));
    }
}
