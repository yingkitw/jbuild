//! Enhanced build cache for incremental compilation and dependency tracking

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Build cache tracking file modifications and compilation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    /// Project root directory
    pub project_root: PathBuf,
    /// Cache entries for source files
    #[serde(skip)]
    pub entries: HashMap<PathBuf, CacheEntry>,
    /// Classpath dependencies
    pub classpath_dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Generated files tracking
    pub generated_files: HashMap<PathBuf, GeneratedFile>,
    /// Last cache update timestamp
    pub last_updated: SystemTime,
}

/// Cache entry for a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Source file path
    pub source_path: PathBuf,
    /// Last modification time when compiled
    pub source_modified: SystemTime,
    /// Output file path
    pub output_path: PathBuf,
    /// Source files this file depends on
    pub dependencies: Vec<PathBuf>,
    /// Compilation checksum
    pub checksum: String,
    /// Whether compilation succeeded
    pub success: bool,
}

/// Information about generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Generator type (e.g., "annotation-processor", "protobuf")
    pub generator: String,
    /// Source files that triggered generation
    pub sources: Vec<PathBuf>,
    /// Generated file path
    pub output_path: PathBuf,
    /// Generation checksum
    pub checksum: String,
}

impl BuildCache {
    /// Create a new build cache for a project
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            entries: HashMap::new(),
            classpath_dependencies: HashMap::new(),
            generated_files: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }

    /// Load cache from file if it exists
    pub fn load(project_root: &Path) -> Result<Self> {
        let cache_path = project_root.join(".jbuild").join("cache.json");
        if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)?;
            let mut cache: BuildCache = serde_json::from_str(&content)?;
            cache.entries = HashMap::new(); // Rebuild entries from disk
            Ok(cache)
        } else {
            Ok(Self::new(project_root.to_path_buf()))
        }
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let cache_dir = self.project_root.join(".jbuild");
        std::fs::create_dir_all(&cache_dir)?;
        let cache_path = cache_dir.join("cache.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&cache_path, content)?;
        Ok(())
    }

    /// Check if a file needs recompilation
    pub fn needs_recompilation(&self, source_path: &Path) -> bool {
        if let Some(entry) = self.entries.get(source_path) {
            // Check if source file was modified
            if let Ok(metadata) = std::fs::metadata(source_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > entry.source_modified {
                        return true;
                    }
                }
            }

            // Check if dependencies were modified
            for dep in &entry.dependencies {
                if self.needs_recompilation(dep) {
                    return true;
                }
            }

            false
        } else {
            true // No cache entry, needs compilation
        }
    }

    /// Update cache entry after successful compilation
    pub fn update_entry(&mut self, source_path: PathBuf, entry: CacheEntry) {
        self.entries.insert(source_path, entry);
        self.last_updated = SystemTime::now();
    }

    /// Invalidate cache for a file and all dependent files
    pub fn invalidate(&mut self, source_path: &Path) {
        // Find all files that depend on this one
        let source_pathbuf = source_path.to_path_buf();
        let dependents: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.dependencies.contains(&source_pathbuf))
            .map(|(path, _)| path.clone())
            .collect();

        // Remove this entry
        self.entries.remove(source_path);

        // Recursively invalidate dependents
        for dependent in dependents {
            self.invalidate(&dependent);
        }
    }

    /// Get stale sources that need recompilation
    pub fn get_stale_sources(&self, sources: &[PathBuf]) -> Vec<PathBuf> {
        sources
            .iter()
            .filter(|p| self.needs_recompilation(p))
            .cloned()
            .collect()
    }

    /// Clean cache for non-existent files
    pub fn clean(&mut self) {
        let mut to_remove = Vec::new();
        for path in self.entries.keys() {
            if !path.exists() {
                to_remove.push(path.clone());
            }
        }
        for path in to_remove {
            self.entries.remove(&path);
        }
    }

    /// Calculate cache checksum for a file
    pub fn calculate_checksum(path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Track classpath dependencies
    pub fn add_classpath_dependency(&mut self, source: PathBuf, dependency: PathBuf) {
        self.classpath_dependencies
            .entry(source)
            .or_default()
            .push(dependency);
    }

    /// Check if classpath has changed
    pub fn classpath_changed(&self, source_path: &Path) -> bool {
        if let Some(deps) = self.classpath_dependencies.get(source_path) {
            deps.iter().any(|dep| self.needs_recompilation(dep))
        } else {
            false
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.entries.len();
        let valid_entries = self.entries.values().filter(|e| e.success).count();
        let generated_count = self.generated_files.len();

        CacheStats {
            total_entries,
            valid_entries,
            hit_rate: if total_entries > 0 {
                (valid_entries as f64 / total_entries as f64 * 100.0) as u32
            } else {
                0
            },
            generated_count,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub hit_rate: u32,
    pub generated_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cache_creation() {
        let cache = BuildCache::new(PathBuf::from("/project"));
        assert_eq!(cache.project_root, PathBuf::from("/project"));
        assert_eq!(cache.entries.len(), 0);
    }

    #[test]
    fn test_needs_recompilation_no_cache() {
        let cache = BuildCache::new(PathBuf::from("/project"));
        assert!(cache.needs_recompilation(Path::new("test.java")));
    }
}
