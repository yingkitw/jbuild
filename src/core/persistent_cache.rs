//! Persistent build cache with disk storage and compression
//!
//! Provides a persistent cache layer that survives across builds

use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use sha2::{Sha256, Digest};

/// Persistent build cache with disk storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentBuildCache {
    /// Cache version for compatibility checking
    pub version: String,
    /// Project identifier
    pub project_id: String,
    /// Compilation cache entries
    pub compilation_cache: HashMap<String, CompilationEntry>,
    /// Dependency resolution cache
    pub dependency_cache: HashMap<String, DependencyEntry>,
    /// Test execution cache
    pub test_cache: HashMap<String, TestEntry>,
    /// Last cache update
    pub last_updated: SystemTime,
}

/// Compilation cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationEntry {
    pub source_hash: String,
    pub output_hash: String,
    pub dependencies: Vec<String>,
    pub compiled_at: SystemTime,
    pub compiler_version: String,
}

/// Dependency resolution cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEntry {
    pub artifact_id: String,
    pub version: String,
    pub resolved_version: String,
    pub transitive_deps: Vec<String>,
    pub resolved_at: SystemTime,
    pub checksum: String,
}

/// Test execution cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEntry {
    pub test_class: String,
    pub source_hash: String,
    pub last_result: TestResult,
    pub executed_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestResult {
    Passed,
    Failed { message: String },
    Skipped,
}

impl PersistentBuildCache {
    const CACHE_VERSION: &'static str = "1.0.0";

    pub fn new(project_id: String) -> Self {
        Self {
            version: Self::CACHE_VERSION.to_string(),
            project_id,
            compilation_cache: HashMap::new(),
            dependency_cache: HashMap::new(),
            test_cache: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }

    /// Load cache from disk
    pub fn load(cache_dir: &Path, project_id: &str) -> Result<Self> {
        let cache_file = cache_dir.join(format!("{}.cache", project_id));
        
        if cache_file.exists() {
            let content = std::fs::read_to_string(&cache_file)?;
            let cache: PersistentBuildCache = serde_json::from_str(&content)?;
            
            // Version compatibility check
            if cache.version != Self::CACHE_VERSION {
                tracing::warn!(
                    "Cache version mismatch: {} vs {}, creating new cache",
                    cache.version,
                    Self::CACHE_VERSION
                );
                return Ok(Self::new(project_id.to_string()));
            }
            
            Ok(cache)
        } else {
            Ok(Self::new(project_id.to_string()))
        }
    }

    /// Save cache to disk
    pub fn save(&self, cache_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(cache_dir)?;
        let cache_file = cache_dir.join(format!("{}.cache", self.project_id));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    /// Check if source needs recompilation
    pub fn needs_compilation(&self, source_path: &Path) -> bool {
        let source_hash = match Self::hash_file(source_path) {
            Ok(hash) => hash,
            Err(_) => return true,
        };

        if let Some(entry) = self.compilation_cache.get(&source_hash) {
            // Check if dependencies changed
            for dep in &entry.dependencies {
                if self.compilation_cache.get(dep).is_none() {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    /// Add compilation entry
    pub fn add_compilation(
        &mut self,
        source_path: &Path,
        output_path: &Path,
        dependencies: Vec<String>,
        compiler_version: String,
    ) -> Result<()> {
        let source_hash = Self::hash_file(source_path)?;
        let output_hash = Self::hash_file(output_path)?;

        self.compilation_cache.insert(
            source_hash.clone(),
            CompilationEntry {
                source_hash,
                output_hash,
                dependencies,
                compiled_at: SystemTime::now(),
                compiler_version,
            },
        );

        self.last_updated = SystemTime::now();
        Ok(())
    }

    /// Check if dependency is cached
    pub fn get_cached_dependency(&self, artifact_id: &str, version: &str) -> Option<&DependencyEntry> {
        let key = format!("{}:{}", artifact_id, version);
        self.dependency_cache.get(&key)
    }

    /// Cache dependency resolution
    pub fn cache_dependency(
        &mut self,
        artifact_id: String,
        version: String,
        resolved_version: String,
        transitive_deps: Vec<String>,
        checksum: String,
    ) {
        let key = format!("{}:{}", artifact_id, version);
        self.dependency_cache.insert(
            key,
            DependencyEntry {
                artifact_id,
                version,
                resolved_version,
                transitive_deps,
                resolved_at: SystemTime::now(),
                checksum,
            },
        );
        self.last_updated = SystemTime::now();
    }

    /// Check if test can be skipped
    pub fn can_skip_test(&self, test_class: &str, source_hash: &str) -> bool {
        if let Some(entry) = self.test_cache.get(test_class) {
            entry.source_hash == source_hash && matches!(entry.last_result, TestResult::Passed)
        } else {
            false
        }
    }

    /// Cache test result
    pub fn cache_test_result(&mut self, test_class: String, source_hash: String, result: TestResult) {
        self.test_cache.insert(
            test_class.clone(),
            TestEntry {
                test_class,
                source_hash,
                last_result: result,
                executed_at: SystemTime::now(),
            },
        );
        self.last_updated = SystemTime::now();
    }

    /// Clean stale entries
    pub fn clean_stale(&mut self, max_age_days: u64) {
        let now = SystemTime::now();
        let max_age = std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);

        self.compilation_cache.retain(|_, entry| {
            now.duration_since(entry.compiled_at)
                .map(|d| d < max_age)
                .unwrap_or(false)
        });

        self.dependency_cache.retain(|_, entry| {
            now.duration_since(entry.resolved_at)
                .map(|d| d < max_age)
                .unwrap_or(false)
        });

        self.test_cache.retain(|_, entry| {
            now.duration_since(entry.executed_at)
                .map(|d| d < max_age)
                .unwrap_or(false)
        });
    }

    /// Calculate file hash
    pub fn hash_file(path: &Path) -> Result<String> {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStatistics {
        CacheStatistics {
            compilation_entries: self.compilation_cache.len(),
            dependency_entries: self.dependency_cache.len(),
            test_entries: self.test_cache.len(),
            total_size_estimate: self.estimate_size(),
        }
    }

    fn estimate_size(&self) -> usize {
        // Rough estimate of cache size in bytes
        self.compilation_cache.len() * 256
            + self.dependency_cache.len() * 512
            + self.test_cache.len() * 128
    }
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub compilation_entries: usize,
    pub dependency_entries: usize,
    pub test_entries: usize,
    pub total_size_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = PersistentBuildCache::new("test-project".to_string());
        assert_eq!(cache.project_id, "test-project");
        assert_eq!(cache.version, PersistentBuildCache::CACHE_VERSION);
    }

    #[test]
    fn test_cache_dependency() {
        let mut cache = PersistentBuildCache::new("test".to_string());
        cache.cache_dependency(
            "org.example:lib".to_string(),
            "1.0.0".to_string(),
            "1.0.0".to_string(),
            vec![],
            "abc123".to_string(),
        );

        let entry = cache.get_cached_dependency("org.example:lib", "1.0.0");
        assert!(entry.is_some());
    }
}
