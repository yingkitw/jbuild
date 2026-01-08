//! Composite Build Support
//!
//! Implements Gradle's composite build feature for including external builds.

use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context};

use crate::gradle::settings::{GradleSettings, parse_settings_file, find_settings_file};

/// An included build in a composite
#[derive(Debug, Clone)]
pub struct IncludedBuild {
    /// Name of the included build
    pub name: String,
    /// Root directory of the included build
    pub root_dir: PathBuf,
    /// Settings for the included build
    pub settings: Option<GradleSettings>,
    /// Projects in the included build
    pub projects: Vec<String>,
    /// Dependency substitutions (original -> replacement)
    pub substitutions: HashMap<String, String>,
}

impl IncludedBuild {
    pub fn new(name: impl Into<String>, root_dir: PathBuf) -> Self {
        Self {
            name: name.into(),
            root_dir,
            settings: None,
            projects: Vec::new(),
            substitutions: HashMap::new(),
        }
    }

    /// Load the included build's settings
    pub fn load(&mut self) -> Result<()> {
        if let Some(settings_file) = find_settings_file(&self.root_dir) {
            let settings = parse_settings_file(&settings_file, &self.root_dir)
                .with_context(|| format!("Failed to load included build: {}", self.name))?;
            
            // Collect project paths
            self.projects = settings.all_project_paths();
            self.settings = Some(settings);
        }
        Ok(())
    }

    /// Add a dependency substitution
    pub fn substitute(&mut self, module: impl Into<String>, with_project: impl Into<String>) {
        self.substitutions.insert(module.into(), with_project.into());
    }

    /// Check if a module should be substituted
    pub fn get_substitution(&self, module: &str) -> Option<&String> {
        self.substitutions.get(module)
    }
}

/// Composite build configuration
#[derive(Debug, Default)]
pub struct CompositeBuild {
    /// Root build directory
    pub root_dir: PathBuf,
    /// Included builds
    pub included_builds: Vec<IncludedBuild>,
    /// Global dependency substitutions
    pub global_substitutions: HashMap<String, String>,
}

impl CompositeBuild {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            included_builds: Vec::new(),
            global_substitutions: HashMap::new(),
        }
    }

    /// Include a build from a directory
    pub fn include_build(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let full_path = if path.is_absolute() {
            path
        } else {
            self.root_dir.join(path)
        };

        let name = full_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("included")
            .to_string();

        let mut included = IncludedBuild::new(&name, full_path);
        included.load()?;

        self.included_builds.push(included);
        Ok(())
    }

    /// Get an included build by name
    pub fn get_included_build(&self, name: &str) -> Option<&IncludedBuild> {
        self.included_builds.iter().find(|b| b.name == name)
    }

    /// Resolve a dependency, checking for substitutions
    pub fn resolve_dependency(&self, group: &str, name: &str) -> Option<String> {
        let module = format!("{group}:{name}");

        // Check global substitutions first
        if let Some(substitution) = self.global_substitutions.get(&module) {
            return Some(substitution.clone());
        }

        // Check included build substitutions
        for build in &self.included_builds {
            if let Some(substitution) = build.get_substitution(&module) {
                return Some(substitution.clone());
            }
        }

        None
    }

    /// Get all projects from all included builds
    pub fn all_included_projects(&self) -> Vec<(String, String)> {
        let mut projects = Vec::new();
        for build in &self.included_builds {
            for project in &build.projects {
                projects.push((build.name.clone(), project.clone()));
            }
        }
        projects
    }

    /// Add a global dependency substitution
    pub fn substitute_dependency(&mut self, module: impl Into<String>, with_project: impl Into<String>) {
        self.global_substitutions.insert(module.into(), with_project.into());
    }
}

/// Parse includeBuild statements from settings.gradle content
pub fn parse_include_builds(content: &str, root_dir: &PathBuf) -> Vec<PathBuf> {
    let mut builds = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Match includeBuild 'path' or includeBuild("path")
        if line.starts_with("includeBuild") {
            let rest = line["includeBuild".len()..].trim();
            let path = rest
                .trim_start_matches('(')
                .trim_end_matches(')')
                .trim_matches('\'')
                .trim_matches('"')
                .trim();

            if !path.is_empty() {
                let build_path = if PathBuf::from(path).is_absolute() {
                    PathBuf::from(path)
                } else {
                    root_dir.join(path)
                };
                builds.push(build_path);
            }
        }
    }

    builds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_included_build_creation() {
        let build = IncludedBuild::new("my-lib", PathBuf::from("/path/to/lib"));
        assert_eq!(build.name, "my-lib");
        assert_eq!(build.root_dir, PathBuf::from("/path/to/lib"));
    }

    #[test]
    fn test_dependency_substitution() {
        let mut build = IncludedBuild::new("my-lib", PathBuf::from("/path/to/lib"));
        build.substitute("com.example:my-lib", ":my-lib");

        assert_eq!(
            build.get_substitution("com.example:my-lib"),
            Some(&":my-lib".to_string())
        );
    }

    #[test]
    fn test_composite_build() {
        let mut composite = CompositeBuild::new(PathBuf::from("/project"));
        composite.substitute_dependency("com.example:shared", ":shared");

        let resolved = composite.resolve_dependency("com.example", "shared");
        assert_eq!(resolved, Some(":shared".to_string()));
    }

    #[test]
    fn test_parse_include_builds() {
        let content = r#"
rootProject.name = 'my-project'

includeBuild '../shared-lib'
includeBuild("../other-lib")
includeBuild '/absolute/path/lib'
"#;

        let root = PathBuf::from("/project");
        let builds = parse_include_builds(content, &root);

        assert_eq!(builds.len(), 3);
        assert_eq!(builds[0], PathBuf::from("/project/../shared-lib"));
        assert_eq!(builds[1], PathBuf::from("/project/../other-lib"));
        assert_eq!(builds[2], PathBuf::from("/absolute/path/lib"));
    }

    #[test]
    fn test_all_included_projects() {
        let mut composite = CompositeBuild::new(PathBuf::from("/project"));
        
        let mut build1 = IncludedBuild::new("lib1", PathBuf::from("/lib1"));
        build1.projects = vec![":".to_string(), ":core".to_string()];
        composite.included_builds.push(build1);

        let mut build2 = IncludedBuild::new("lib2", PathBuf::from("/lib2"));
        build2.projects = vec![":".to_string()];
        composite.included_builds.push(build2);

        let all_projects = composite.all_included_projects();
        assert_eq!(all_projects.len(), 3);
    }
}
