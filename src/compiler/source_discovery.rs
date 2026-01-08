use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Source file discovery for Java projects
pub struct SourceDiscovery;

impl SourceDiscovery {
    /// Discover all Java source files in a directory
    pub fn discover_java_sources(source_root: &Path) -> Result<Vec<PathBuf>> {
        let mut sources = Vec::new();

        if !source_root.exists() {
            return Ok(sources);
        }

        for entry in WalkDir::new(source_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "java") {
                sources.push(path.to_path_buf());
            }
        }

        Ok(sources)
    }

    /// Discover all Java source files in multiple source roots
    pub fn discover_from_roots(source_roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut all_sources = Vec::new();

        for root in source_roots {
            let sources = Self::discover_java_sources(root)
                .with_context(|| format!("Failed to discover sources in {root:?}"))?;
            all_sources.extend(sources);
        }

        Ok(all_sources)
    }

    /// Get source roots from a project's build configuration
    pub fn get_source_roots(build: &crate::model::build::Build) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        
        if let Some(source_dir) = &build.source_directory {
            roots.push(PathBuf::from(source_dir));
        } else {
            roots.push(PathBuf::from("src/main/java"));
        }

        roots
    }

    /// Get test source roots from a project's build configuration
    pub fn get_test_source_roots(build: &crate::model::build::Build) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        
        if let Some(test_source_dir) = &build.test_source_directory {
            roots.push(PathBuf::from(test_source_dir));
        } else {
            roots.push(PathBuf::from("src/test/java"));
        }

        roots
    }
}

