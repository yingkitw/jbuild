use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;
use glob::Pattern as GlobPattern;

/// Resource file filter
#[derive(Debug, Clone)]
pub struct ResourceFilter {
    includes: Vec<GlobPattern>,
    excludes: Vec<GlobPattern>,
}

impl ResourceFilter {
    pub fn new() -> Self {
        Self {
            includes: vec![GlobPattern::new("**/*").expect("Default include pattern")],
            excludes: Vec::new(),
        }
    }

    pub fn with_includes(mut self, includes: Vec<String>) -> Result<Self> {
        self.includes = includes
            .into_iter()
            .map(|p| GlobPattern::new(&p))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid include pattern")?;
        Ok(self)
    }

    pub fn with_excludes(mut self, excludes: Vec<String>) -> Result<Self> {
        self.excludes = excludes
            .into_iter()
            .map(|p| GlobPattern::new(&p))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid exclude pattern")?;
        Ok(self)
    }

    /// Check if a path matches the filter
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check excludes first
        for exclude in &self.excludes {
            if exclude.matches(&path_str) {
                return false;
            }
        }

        // Check includes
        for include in &self.includes {
            if include.matches(&path_str) {
                return true;
            }
        }

        false
    }
}

impl Default for ResourceFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource collector
pub struct ResourceCollector;

impl ResourceCollector {
    /// Collect resources from a directory with filtering
    pub fn collect_resources(
        source_dir: &Path,
        target_path: Option<&str>,
        filter: &ResourceFilter,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut resources = Vec::new();

        if !source_dir.exists() {
            return Ok(resources);
        }

        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let source_path = entry.path();
            
            if source_path.is_file() && filter.matches(source_path) {
                let relative_path = source_path.strip_prefix(source_dir)
                    .context("Failed to get relative path")?;
                
                let target = if let Some(target_prefix) = target_path {
                    PathBuf::from(target_prefix).join(relative_path)
                } else {
                    relative_path.to_path_buf()
                };

                resources.push((source_path.to_path_buf(), target));
            }
        }

        Ok(resources)
    }

    /// Collect resources from multiple directories
    pub fn collect_from_directories(
        directories: &[(PathBuf, Option<String>, ResourceFilter)],
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut all_resources = Vec::new();

        for (source_dir, target_path, filter) in directories {
            let resources = Self::collect_resources(
                source_dir,
                target_path.as_deref(),
                filter,
            )?;
            all_resources.extend(resources);
        }

        Ok(all_resources)
    }
}

