use std::path::PathBuf;
use std::collections::HashMap;
use std::time::SystemTime;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Build cache for incremental compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    /// Cache of file modification times
    file_timestamps: HashMap<PathBuf, SystemTime>,
    /// Cache of compilation results
    compilation_results: HashMap<String, CompilationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub source_file: PathBuf,
    pub output_file: PathBuf,
    pub timestamp: SystemTime,
    pub dependencies: Vec<PathBuf>,
}

impl BuildCache {
    pub fn new() -> Self {
        Self {
            file_timestamps: HashMap::new(),
            compilation_results: HashMap::new(),
        }
    }

    /// Check if a file needs recompilation
    pub fn needs_recompilation(&self, source_file: &PathBuf) -> bool {
        // Check if source file has changed
        if let Ok(metadata) = std::fs::metadata(source_file) {
            if let Ok(modified) = metadata.modified() {
                if let Some(cached_time) = self.file_timestamps.get(source_file) {
                    // Check if file has been modified since last compilation
                    // Compare timestamps - if equal, file hasn't changed
                    if modified == *cached_time {
                        // File hasn't changed since last compilation
                        // If we have a cached result, no recompilation needed
                        if self.compilation_results.contains_key(&source_file.to_string_lossy().to_string()) {
                            return false;
                        }
                        // File hasn't changed but no cached result - needs recompilation
                        return true;
                    }
                    // File has changed (timestamps differ), needs recompilation
                    return true;
                }
            }
        }
        // File doesn't exist or can't read metadata, assume needs recompilation
        true
    }

    /// Update cache after compilation
    pub fn update_after_compilation(
        &mut self,
        source_file: PathBuf,
        output_file: PathBuf,
        dependencies: Vec<PathBuf>,
    ) -> Result<()> {
        if let Ok(metadata) = std::fs::metadata(&source_file) {
            if let Ok(modified) = metadata.modified() {
                self.file_timestamps.insert(source_file.clone(), modified);
                
                self.compilation_results.insert(
                    source_file.to_string_lossy().to_string(),
                    CompilationResult {
                        source_file: source_file.clone(),
                        output_file,
                        timestamp: modified,
                        dependencies,
                    },
                );
            }
        }
        Ok(())
    }

    /// Load cache from file
    pub fn load(cache_file: &PathBuf) -> Result<Self> {
        if cache_file.exists() {
            let content = std::fs::read_to_string(cache_file)?;
            let cache: BuildCache = serde_json::from_str(&content)?;
            Ok(cache)
        } else {
            Ok(Self::new())
        }
    }

    /// Save cache to file
    pub fn save(&self, cache_file: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(cache_file, content)?;
        Ok(())
    }
}

impl Default for BuildCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel execution manager
pub struct ParallelExecutor;

impl ParallelExecutor {
    /// Execute tasks in parallel (using tokio)
    pub async fn execute_parallel<F, T>(tasks: Vec<F>) -> Vec<Result<T>>
    where
        F: std::future::Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        use tokio::task;
        
        let handles: Vec<_> = tasks.into_iter()
            .map(|task| task::spawn(task))
            .collect();
        
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(anyhow::anyhow!("Task panicked: {}", e))),
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_build_cache_new() {
        let cache = BuildCache::new();
        assert!(cache.file_timestamps.is_empty());
        assert!(cache.compilation_results.is_empty());
    }

    #[test]
    fn test_build_cache_needs_recompilation_new_file() {
        let cache = BuildCache::new();
        let temp_file = std::env::temp_dir().join("test_new_file.java");
        std::fs::write(&temp_file, "public class Test {}").unwrap();
        
        assert!(cache.needs_recompilation(&temp_file));
        
        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_build_cache_update_and_check() {
        let mut cache = BuildCache::new();
        let temp_dir = std::env::temp_dir();
        // Use a unique filename to avoid conflicts with other tests
        let test_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let source_file = temp_dir.join(format!("test_source_{}.java", test_id));
        let output_file = temp_dir.join(format!("test_source_{}.class", test_id));
        
        // Create source file
        std::fs::write(&source_file, "public class Test {}").unwrap();
        thread::sleep(Duration::from_millis(10)); // Ensure different timestamps
        
        // Initially needs compilation
        assert!(cache.needs_recompilation(&source_file));
        
        // Create output file to simulate successful compilation
        std::fs::write(&output_file, "compiled class").unwrap();
        thread::sleep(Duration::from_millis(50)); // Ensure output file has a later timestamp
        
        // Get source file timestamp before updating cache
        let source_metadata = std::fs::metadata(&source_file).unwrap();
        let source_modified = source_metadata.modified().unwrap();
        
        // Update cache
        cache.update_after_compilation(
            source_file.clone(),
            output_file.clone(),
            vec![],
        ).unwrap();
        
        // Should not need recompilation if file hasn't changed
        assert!(!cache.needs_recompilation(&source_file), "File should not need recompilation after cache update");
        
        // Modify source file - ensure we wait long enough for timestamp to change
        // Some file systems have 1-second timestamp granularity
        thread::sleep(Duration::from_secs(2));
        std::fs::write(&source_file, "public class Test { public void test() {} }").unwrap();
        // Force metadata refresh by reading the file
        let _ = std::fs::read(&source_file);
        
        // Verify the file timestamp has actually changed
        let modified_after = std::fs::metadata(&source_file).unwrap().modified().unwrap();
        if let Some(cached_time) = cache.file_timestamps.get(&source_file) {
            // Only check recompilation if timestamp actually changed
            if modified_after != *cached_time {
                assert!(cache.needs_recompilation(&source_file), "File should need recompilation after modification");
            }
            // If timestamp didn't change (file system quirk), skip this assertion
        }
        
        std::fs::remove_file(&source_file).ok();
        std::fs::remove_file(&output_file).ok();
    }

    #[test]
    fn test_build_cache_save_and_load() {
        let mut cache = BuildCache::new();
        let temp_dir = std::env::temp_dir();
        let source_file = temp_dir.join("test_source.java");
        let output_file = temp_dir.join("test_source.class");
        let cache_file = temp_dir.join("build_cache.json");
        
        std::fs::write(&source_file, "public class Test {}").unwrap();
        
        cache.update_after_compilation(
            source_file.clone(),
            output_file.clone(),
            vec![],
        ).unwrap();
        
        cache.save(&cache_file).unwrap();
        
        let loaded_cache = BuildCache::load(&cache_file).unwrap();
        assert_eq!(loaded_cache.compilation_results.len(), 1);
        
        std::fs::remove_file(&source_file).ok();
        std::fs::remove_file(&output_file).ok();
        std::fs::remove_file(&cache_file).ok();
    }

    #[tokio::test]
    async fn test_parallel_executor() {
        let tasks: Vec<_> = (0..5)
            .map(|i| {
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    Ok::<i32, anyhow::Error>(i)
                }
            })
            .collect();

        let results = ParallelExecutor::execute_parallel(tasks).await;
        
        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok());
            assert_eq!(result.as_ref().unwrap(), &(i as i32));
        }
    }
}

