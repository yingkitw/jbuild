//! Change detection for incremental compilation

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Detects changes in source files and dependencies
pub struct ChangeDetector {
    /// Watched source files
    sources: HashMap<PathBuf, SourceFile>,
    /// Classpath entries to monitor
    classpath: Vec<PathBuf>,
}

/// Information about a source file
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub last_modified: std::time::SystemTime,
    pub checksum: String,
    pub dependencies: Vec<PathBuf>,
}

/// Detected changes
#[derive(Debug, Clone)]
pub enum Change {
    /// File was added
    Added(PathBuf),
    /// File was modified
    Modified(PathBuf),
    /// File was deleted
    Deleted(PathBuf),
    /// File dependencies changed
    DependencyChanged(PathBuf, Vec<PathBuf>),
}

impl ChangeDetector {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            classpath: Vec::new(),
        }
    }

    /// Add a source file to watch
    pub fn watch_source(&mut self, path: PathBuf) -> Result<()> {
        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let checksum = crate::core::cache::BuildCache::calculate_checksum(&path)?;

        self.sources.insert(
            path.clone(),
            SourceFile {
                path,
                last_modified: modified,
                checksum,
                dependencies: Vec::new(),
            },
        );
        Ok(())
    }

    /// Scan for changes since last snapshot
    pub fn detect_changes(&self) -> Vec<Change> {
        let mut changes = Vec::new();

        for (path, source) in &self.sources {
            // Check if file still exists
            if !path.exists() {
                changes.push(Change::Deleted(path.clone()));
                continue;
            }

            // Check modification time
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > source.last_modified {
                        // Verify with checksum
                        if let Ok(checksum) = crate::core::cache::BuildCache::calculate_checksum(path) {
                            if checksum != source.checksum {
                                changes.push(Change::Modified(path.clone()));
                            }
                        }
                    }
                }
            }

            // Check dependencies
            for dep in &source.dependencies {
                if !dep.exists() {
                    changes.push(Change::DependencyChanged(
                        path.clone(),
                        vec![dep.clone()],
                    ));
                } else if let Ok(metadata) = std::fs::metadata(dep) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > source.last_modified {
                            changes.push(Change::DependencyChanged(
                                path.clone(),
                                vec![dep.clone()],
                            ));
                        }
                    }
                }
            }
        }

        changes
    }

    /// Update source file information
    pub fn update_source(&mut self, path: &Path, dependencies: Vec<PathBuf>) -> Result<()> {
        if let Some(source) = self.sources.get_mut(path) {
            source.dependencies = dependencies;
            if let Ok(metadata) = std::fs::metadata(path) {
                source.last_modified = metadata.modified()?;
            }
            source.checksum = crate::core::cache::BuildCache::calculate_checksum(path)?;
        }
        Ok(())
    }

    /// Get affected files based on changes
    pub fn get_affected_files(&self, changes: &[Change]) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();

        for change in changes {
            match change {
                Change::Added(path) | Change::Modified(path) | Change::Deleted(path) => {
                    affected.insert(path.clone());
                    // Find files that depend on this one
                    for (source_path, source) in &self.sources {
                        if source.dependencies.contains(path) {
                            affected.insert(source_path.clone());
                        }
                    }
                }
                Change::DependencyChanged(path, _) => {
                    affected.insert(path.clone());
                }
            }
        }

        affected
    }

    /// Get minimal recompilation set
    pub fn get_recompilation_set(&self, changes: &[Change]) -> Vec<PathBuf> {
        let affected = self.get_affected_files(changes);
        affected.into_iter().collect()
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_detector_creation() {
        let detector = ChangeDetector::new();
        assert_eq!(detector.sources.len(), 0);
    }
}
