//! Gradle settings.gradle parser
//!
//! Parses settings.gradle and settings.gradle.kts files to support multi-project builds.

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Gradle settings model
#[derive(Debug, Clone)]
pub struct GradleSettings {
    /// Root project name
    pub root_project_name: String,
    /// Included subprojects
    pub subprojects: Vec<SubprojectConfig>,
    /// Plugin management configuration
    pub plugin_management: Option<PluginManagement>,
    /// Dependency resolution management
    pub dependency_resolution_management: Option<DependencyResolutionManagement>,
    /// Build cache configuration
    pub build_cache: Option<BuildCacheConfig>,
}

/// Subproject configuration
#[derive(Debug, Clone)]
pub struct SubprojectConfig {
    /// Project path (e.g., ":subproject" or ":parent:child")
    pub path: String,
    /// Project directory (relative to root)
    pub project_dir: Option<PathBuf>,
    /// Build file name (if non-default)
    pub build_file_name: Option<String>,
}

impl SubprojectConfig {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            project_dir: None,
            build_file_name: None,
        }
    }

    pub fn with_project_dir(mut self, dir: PathBuf) -> Self {
        self.project_dir = Some(dir);
        self
    }

    /// Get the project name (last segment of path)
    pub fn name(&self) -> &str {
        self.path.rsplit(':').next().unwrap_or(&self.path)
    }

    /// Get the directory for this subproject relative to root
    pub fn directory(&self, root_dir: &Path) -> PathBuf {
        if let Some(ref dir) = self.project_dir {
            root_dir.join(dir)
        } else {
            // Default: path segments become directory segments
            let segments: Vec<&str> = self.path.split(':').filter(|s| !s.is_empty()).collect();
            let mut path = root_dir.to_path_buf();
            for segment in segments {
                path = path.join(segment);
            }
            path
        }
    }
}

/// Plugin management configuration
#[derive(Debug, Clone, Default)]
pub struct PluginManagement {
    /// Plugin repositories
    pub repositories: Vec<String>,
    /// Plugin version overrides
    pub plugins: Vec<PluginSpec>,
}

/// Plugin specification
#[derive(Debug, Clone)]
pub struct PluginSpec {
    /// Plugin ID
    pub id: String,
    /// Plugin version
    pub version: Option<String>,
}

/// Dependency resolution management
#[derive(Debug, Clone, Default)]
pub struct DependencyResolutionManagement {
    /// Repository mode
    pub repositories_mode: Option<String>,
    /// Repositories
    pub repositories: Vec<String>,
}

/// Build cache configuration
#[derive(Debug, Clone, Default)]
pub struct BuildCacheConfig {
    /// Local cache enabled
    pub local_enabled: bool,
    /// Remote cache enabled
    pub remote_enabled: bool,
    /// Remote cache URL
    pub remote_url: Option<String>,
    /// Push to remote cache
    pub push: bool,
}

impl GradleSettings {
    pub fn new(root_project_name: impl Into<String>) -> Self {
        Self {
            root_project_name: root_project_name.into(),
            subprojects: Vec::new(),
            plugin_management: None,
            dependency_resolution_management: None,
            build_cache: None,
        }
    }

    /// Add a subproject
    pub fn include(&mut self, path: impl Into<String>) {
        self.subprojects.push(SubprojectConfig::new(path));
    }

    /// Check if this is a multi-project build
    pub fn is_multi_project(&self) -> bool {
        !self.subprojects.is_empty()
    }

    /// Get all project paths including root
    pub fn all_project_paths(&self) -> Vec<String> {
        let mut paths = vec![":".to_string()];
        for subproject in &self.subprojects {
            paths.push(subproject.path.clone());
        }
        paths
    }
}

/// Parse a settings.gradle or settings.gradle.kts file
pub fn parse_settings_file(settings_file: &Path, root_dir: &Path) -> Result<GradleSettings> {
    let content = std::fs::read_to_string(settings_file)
        .with_context(|| format!("Failed to read settings file: {:?}", settings_file))?;

    parse_settings(&content, root_dir)
}

/// Parse settings content
pub fn parse_settings(content: &str, root_dir: &Path) -> Result<GradleSettings> {
    let root_name = root_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let mut settings = GradleSettings::new(root_name);

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        // Parse rootProject.name
        if let Some(name) = parse_root_project_name(line) {
            settings.root_project_name = name;
            continue;
        }

        // Parse include statements
        if let Some(includes) = parse_include(line) {
            for include in includes {
                settings.include(include);
            }
            continue;
        }

        // Parse includeFlat statements
        if let Some(includes) = parse_include_flat(line) {
            for include in includes {
                let mut config = SubprojectConfig::new(format!(":{}", include));
                config.project_dir = Some(PathBuf::from(format!("../{}", include)));
                settings.subprojects.push(config);
            }
        }
    }

    Ok(settings)
}

/// Parse rootProject.name = "name" or rootProject.name = 'name'
fn parse_root_project_name(line: &str) -> Option<String> {
    let patterns = [
        "rootProject.name",
        "rootProject.name=",
    ];

    for pattern in patterns {
        if line.starts_with(pattern) {
            let rest = line[pattern.len()..].trim();
            let rest = rest.trim_start_matches('=').trim();
            return extract_string_value(rest);
        }
    }

    None
}

/// Parse include ':project1', ':project2' or include(":project1", ":project2")
fn parse_include(line: &str) -> Option<Vec<String>> {
    if !line.starts_with("include") {
        return None;
    }

    let rest = line["include".len()..].trim();

    // Handle both include ':a', ':b' and include(':a', ':b')
    let rest = rest.trim_start_matches('(').trim_end_matches(')');

    let mut includes = Vec::new();
    for part in rest.split(',') {
        let part = part.trim();
        if let Some(value) = extract_string_value(part) {
            includes.push(value);
        }
    }

    if includes.is_empty() {
        None
    } else {
        Some(includes)
    }
}

/// Parse includeFlat 'project1', 'project2'
fn parse_include_flat(line: &str) -> Option<Vec<String>> {
    if !line.starts_with("includeFlat") {
        return None;
    }

    let rest = line["includeFlat".len()..].trim();
    let rest = rest.trim_start_matches('(').trim_end_matches(')');

    let mut includes = Vec::new();
    for part in rest.split(',') {
        let part = part.trim();
        if let Some(value) = extract_string_value(part) {
            includes.push(value);
        }
    }

    if includes.is_empty() {
        None
    } else {
        Some(includes)
    }
}

/// Extract string value from quoted string
fn extract_string_value(s: &str) -> Option<String> {
    let s = s.trim();

    // Handle double quotes
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Some(s[1..s.len() - 1].to_string());
    }

    // Handle single quotes
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return Some(s[1..s.len() - 1].to_string());
    }

    None
}

/// Find settings file in a directory
pub fn find_settings_file(dir: &Path) -> Option<PathBuf> {
    let settings_gradle = dir.join("settings.gradle");
    if settings_gradle.exists() {
        return Some(settings_gradle);
    }

    let settings_kts = dir.join("settings.gradle.kts");
    if settings_kts.exists() {
        return Some(settings_kts);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root_project_name() {
        assert_eq!(
            parse_root_project_name("rootProject.name = \"my-project\""),
            Some("my-project".to_string())
        );
        assert_eq!(
            parse_root_project_name("rootProject.name = 'my-project'"),
            Some("my-project".to_string())
        );
        assert_eq!(
            parse_root_project_name("rootProject.name=\"my-project\""),
            Some("my-project".to_string())
        );
    }

    #[test]
    fn test_parse_include() {
        assert_eq!(
            parse_include("include ':app'"),
            Some(vec![":app".to_string()])
        );
        assert_eq!(
            parse_include("include ':app', ':lib'"),
            Some(vec![":app".to_string(), ":lib".to_string()])
        );
        assert_eq!(
            parse_include("include(\":app\", \":lib\")"),
            Some(vec![":app".to_string(), ":lib".to_string()])
        );
    }

    #[test]
    fn test_parse_include_flat() {
        assert_eq!(
            parse_include_flat("includeFlat 'shared'"),
            Some(vec!["shared".to_string()])
        );
    }

    #[test]
    fn test_parse_settings() {
        let content = r#"
rootProject.name = "my-multi-project"

include ':app'
include ':lib:core', ':lib:utils'
"#;

        let settings = parse_settings(content, Path::new("/project")).unwrap();
        assert_eq!(settings.root_project_name, "my-multi-project");
        assert_eq!(settings.subprojects.len(), 3);
        assert!(settings.is_multi_project());
    }

    #[test]
    fn test_subproject_config() {
        let config = SubprojectConfig::new(":lib:core");
        assert_eq!(config.name(), "core");

        let dir = config.directory(Path::new("/project"));
        assert_eq!(dir, PathBuf::from("/project/lib/core"));
    }

    #[test]
    fn test_subproject_with_custom_dir() {
        let config = SubprojectConfig::new(":app")
            .with_project_dir(PathBuf::from("application"));

        let dir = config.directory(Path::new("/project"));
        assert_eq!(dir, PathBuf::from("/project/application"));
    }

    #[test]
    fn test_all_project_paths() {
        let mut settings = GradleSettings::new("root");
        settings.include(":app");
        settings.include(":lib");

        let paths = settings.all_project_paths();
        assert_eq!(paths, vec![":".to_string(), ":app".to_string(), ":lib".to_string()]);
    }
}
