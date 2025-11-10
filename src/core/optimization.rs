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
                    if &modified == cached_time {
                        // Check if output file exists and is newer
                        if let Some(result) = self.compilation_results.get(&source_file.to_string_lossy().to_string()) {
                            if result.output_file.exists() {
                                if let Ok(output_meta) = std::fs::metadata(&result.output_file) {
                                    if let Ok(output_time) = output_meta.modified() {
                                        return output_time < modified;
                                    }
                                }
                            }
                        }
                        return false;
                    }
                }
            }
        }
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

