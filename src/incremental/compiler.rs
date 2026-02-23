//! Incremental compiler with smart recompilation

use std::path::{Path, PathBuf};
use std::collections::HashSet;
use anyhow::Result;
use super::detector::ChangeDetector;
use crate::core::cache::BuildCache;

/// Incremental compiler that only recompiles changed files
pub struct IncrementalCompiler {
    cache: BuildCache,
    change_detector: ChangeDetector,
}

impl IncrementalCompiler {
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let cache = BuildCache::load(&project_root)?;
        let change_detector = ChangeDetector::new();
        Ok(Self {
            cache,
            change_detector,
        })
    }

    /// Compile only changed sources
    pub fn compile_incremental<F>(
        &mut self,
        sources: Vec<PathBuf>,
        mut compile_fn: F,
    ) -> Result<CompilationResult>
    where
        F: FnMut(&Path) -> Result<CompileOutput>,
    {
        // Detect changes
        let changes = self.change_detector.detect_changes();
        let recompile_set = self.change_detector.get_recompilation_set(&changes);

        // Also check cache for stale sources
        let stale = self.cache.get_stale_sources(&sources);
        let mut to_compile: HashSet<_> = recompile_set.iter().chain(stale.iter()).collect();
        to_compile.retain(|p| sources.contains(p));

        let mut outputs = Vec::new();
        let mut failed = Vec::new();

        for source in &sources {
            if to_compile.contains(source) {
                match compile_fn(source) {
                    Ok(output) => {
                        // Update cache first before moving output
                        if let Ok(modified) = std::fs::metadata(source).and_then(|m| m.modified()) {
                            let checksum = BuildCache::calculate_checksum(source)?;
                            let entry = crate::core::cache::CacheEntry {
                                source_path: source.clone(),
                                source_modified: modified,
                                output_path: output.output_path.clone(),
                                dependencies: output.dependencies.clone(),
                                checksum,
                                success: true,
                            };
                            self.cache.update_entry(source.clone(), entry);
                        }
                        outputs.push(output);
                    }
                    Err(e) => {
                        failed.push((source.clone(), e));
                    }
                }
            } else {
                // Use cached output
                if let Some(entry) = self.cache.entries.get(source) {
                    outputs.push(CompileOutput {
                        source_path: source.clone(),
                        output_path: entry.output_path.clone(),
                        dependencies: entry.dependencies.clone(),
                        success: true,
                    });
                }
            }
        }

        // Save cache
        let _ = self.cache.save();

        Ok(CompilationResult {
            compiled: to_compile.len(),
            cached: sources.len() - to_compile.len(),
            outputs,
            failed,
        })
    }

    /// Force full recompilation
    pub fn compile_full<F>(&mut self, sources: Vec<PathBuf>, mut compile_fn: F) -> Result<CompilationResult>
    where
        F: FnMut(&Path) -> Result<CompileOutput>,
    {
        let mut outputs = Vec::new();
        let mut failed = Vec::new();

        for source in &sources {
            match compile_fn(source) {
                Ok(output) => {
                    // Update cache
                    if let Ok(modified) = std::fs::metadata(source).and_then(|m| m.modified()) {
                        let checksum = BuildCache::calculate_checksum(source)?;
                        let entry = crate::core::cache::CacheEntry {
                            source_path: source.clone(),
                            source_modified: modified,
                            output_path: output.output_path.clone(),
                            dependencies: output.dependencies.clone(),
                            checksum,
                            success: true,
                        };
                        self.cache.update_entry(source.clone(), entry);
                    }
                    outputs.push(output);
                }
                Err(e) => {
                    failed.push((source.clone(), e));
                }
            }
        }

        // Save cache
        let _ = self.cache.save();

        Ok(CompilationResult {
            compiled: sources.len(),
            cached: 0,
            outputs,
            failed,
        })
    }

    /// Clean stale cache entries
    pub fn clean(&mut self) {
        self.cache.clean();
    }
}

/// Result of incremental compilation
#[derive(Debug)]
pub struct CompilationResult {
    pub compiled: usize,
    pub cached: usize,
    pub outputs: Vec<CompileOutput>,
    pub failed: Vec<(PathBuf, anyhow::Error)>,
}

/// Output from compiling a source file
#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub dependencies: Vec<PathBuf>,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_compiler() {
        // Test would require actual file system
    }
}
