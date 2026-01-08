//! Gradle Source Sets
//!
//! Implements Gradle's SourceSet model for organizing source files.

use std::path::PathBuf;

/// A source set represents a logical group of source files and resources
#[derive(Debug, Clone)]
pub struct SourceSet {
    /// Source set name (e.g., "main", "test")
    pub name: String,
    /// Java source directories
    pub java_src_dirs: Vec<PathBuf>,
    /// Resource directories
    pub resources_dirs: Vec<PathBuf>,
    /// Output directory for compiled classes
    pub output_classes_dir: PathBuf,
    /// Output directory for processed resources
    pub output_resources_dir: PathBuf,
    /// Compile classpath configuration name
    pub compile_classpath_config: String,
    /// Runtime classpath configuration name
    pub runtime_classpath_config: String,
}

impl SourceSet {
    /// Create a new source set with default directories
    pub fn new(name: impl Into<String>, base_dir: &PathBuf) -> Self {
        let name = name.into();
        let src_dir = base_dir.join("src").join(&name);

        Self {
            java_src_dirs: vec![src_dir.join("java")],
            resources_dirs: vec![src_dir.join("resources")],
            output_classes_dir: base_dir.join("build/classes/java").join(&name),
            output_resources_dir: base_dir.join("build/resources").join(&name),
            compile_classpath_config: if name == "main" {
                "compileClasspath".to_string()
            } else {
                format!("{name}CompileClasspath")
            },
            runtime_classpath_config: if name == "main" {
                "runtimeClasspath".to_string()
            } else {
                format!("{name}RuntimeClasspath")
            },
            name,
        }
    }

    /// Create the main source set
    pub fn main(base_dir: &PathBuf) -> Self {
        Self::new("main", base_dir)
    }

    /// Create the test source set
    pub fn test(base_dir: &PathBuf) -> Self {
        Self::new("test", base_dir)
    }

    /// Add a Java source directory
    pub fn add_java_src_dir(&mut self, dir: PathBuf) {
        self.java_src_dirs.push(dir);
    }

    /// Add a resources directory
    pub fn add_resources_dir(&mut self, dir: PathBuf) {
        self.resources_dirs.push(dir);
    }

    /// Get all Java source files
    pub fn get_java_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for dir in &self.java_src_dirs {
            if dir.exists() {
                let entries: Vec<PathBuf> = walkdir::WalkDir::new(dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|ext| ext == "java").unwrap_or(false))
                    .map(|e| e.path().to_path_buf())
                    .collect();
                files.extend(entries);
            }
        }
        files
    }

    /// Get all resource files
    pub fn get_resource_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for dir in &self.resources_dirs {
            if dir.exists() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            files.push(path);
                        }
                    }
                }
            }
        }
        files
    }

    /// Check if any source directories exist
    pub fn has_sources(&self) -> bool {
        self.java_src_dirs.iter().any(|d| d.exists())
    }

    /// Check if any resource directories exist
    pub fn has_resources(&self) -> bool {
        self.resources_dirs.iter().any(|d| d.exists())
    }
}

/// Container for all source sets in a project
#[derive(Debug, Default)]
pub struct SourceSetContainer {
    source_sets: Vec<SourceSet>,
}

impl SourceSetContainer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with standard Java source sets (main and test)
    pub fn with_java_defaults(base_dir: &PathBuf) -> Self {
        let mut container = Self::new();
        container.add(SourceSet::main(base_dir));
        container.add(SourceSet::test(base_dir));
        container
    }

    /// Add a source set
    pub fn add(&mut self, source_set: SourceSet) {
        self.source_sets.push(source_set);
    }

    /// Get a source set by name
    pub fn get(&self, name: &str) -> Option<&SourceSet> {
        self.source_sets.iter().find(|s| s.name == name)
    }

    /// Get the main source set
    pub fn main(&self) -> Option<&SourceSet> {
        self.get("main")
    }

    /// Get the test source set
    pub fn test(&self) -> Option<&SourceSet> {
        self.get("test")
    }

    /// Get all source set names
    pub fn names(&self) -> Vec<String> {
        self.source_sets.iter().map(|s| s.name.clone()).collect()
    }

    /// Iterate over all source sets
    pub fn iter(&self) -> impl Iterator<Item = &SourceSet> {
        self.source_sets.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_set_creation() {
        let base_dir = PathBuf::from("/project");
        let source_set = SourceSet::new("main", &base_dir);

        assert_eq!(source_set.name, "main");
        assert_eq!(source_set.java_src_dirs, vec![PathBuf::from("/project/src/main/java")]);
        assert_eq!(source_set.resources_dirs, vec![PathBuf::from("/project/src/main/resources")]);
        assert_eq!(source_set.output_classes_dir, PathBuf::from("/project/build/classes/java/main"));
    }

    #[test]
    fn test_test_source_set() {
        let base_dir = PathBuf::from("/project");
        let source_set = SourceSet::test(&base_dir);

        assert_eq!(source_set.name, "test");
        assert_eq!(source_set.compile_classpath_config, "testCompileClasspath");
        assert_eq!(source_set.runtime_classpath_config, "testRuntimeClasspath");
    }

    #[test]
    fn test_source_set_container() {
        let base_dir = PathBuf::from("/project");
        let container = SourceSetContainer::with_java_defaults(&base_dir);

        assert!(container.get("main").is_some());
        assert!(container.get("test").is_some());
        assert_eq!(container.names(), vec!["main".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_add_custom_source_dir() {
        let base_dir = PathBuf::from("/project");
        let mut source_set = SourceSet::main(&base_dir);

        source_set.add_java_src_dir(PathBuf::from("/project/src/generated/java"));

        assert_eq!(source_set.java_src_dirs.len(), 2);
    }
}
