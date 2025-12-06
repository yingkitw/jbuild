//! Gradle model structures
//! 
//! Defines the data structures for representing Gradle build scripts.

pub mod parser;

pub use parser::parse_gradle_build_script;

use std::path::PathBuf;

/// Gradle project model
#[derive(Debug, Clone)]
pub struct GradleProject {
    /// Project name
    pub name: String,
    /// Project group
    pub group: Option<String>,
    /// Project version
    pub version: Option<String>,
    /// Source compatibility
    pub source_compatibility: Option<String>,
    /// Target compatibility
    pub target_compatibility: Option<String>,
    /// Main class (for application plugin)
    pub main_class: Option<String>,
    /// Base directory
    pub base_dir: PathBuf,
    /// Build file path
    pub build_file: PathBuf,
    /// Tasks defined in this project
    pub tasks: Vec<Task>,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// Repositories
    pub repositories: Vec<Repository>,
    /// Plugins
    pub plugins: Vec<Plugin>,
    /// Subprojects (for multi-project builds)
    pub subprojects: Vec<GradleProject>,
}

/// Gradle task
#[derive(Debug, Clone)]
pub struct Task {
    /// Task name
    pub name: String,
    /// Task type (e.g., "JavaCompile", "Jar", "Test")
    pub task_type: Option<String>,
    /// Task description
    pub description: Option<String>,
    /// Task group
    pub group: Option<String>,
    /// Task dependencies (other tasks that must run first)
    pub depends_on: Vec<String>,
    /// Task actions (what the task does)
    pub actions: Vec<String>,
}

/// Gradle dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Dependency configuration (e.g., "implementation", "testImplementation")
    pub configuration: String,
    /// Dependency notation (e.g., "group:artifact:version" or "group:artifact")
    pub notation: String,
    /// Dependency group
    pub group: Option<String>,
    /// Dependency artifact
    pub artifact: Option<String>,
    /// Dependency version
    pub version: Option<String>,
    /// Dependency classifier
    pub classifier: Option<String>,
    /// Dependency extension
    pub extension: Option<String>,
}

/// Gradle repository
#[derive(Debug, Clone)]
pub struct Repository {
    /// Repository name
    pub name: String,
    /// Repository type (e.g., "maven", "ivy", "flatDir")
    pub repo_type: RepositoryType,
    /// Repository URL
    pub url: Option<String>,
}

/// Repository type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryType {
    Maven,
    Ivy,
    FlatDir,
    MavenCentral,
    JCenter,
    Google,
}

/// Gradle plugin
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Plugin ID (e.g., "java", "org.gradle.java")
    pub id: String,
    /// Plugin version (if specified)
    pub version: Option<String>,
}

impl GradleProject {
    /// Create a new Gradle project
    pub fn new(base_dir: PathBuf, build_file: PathBuf) -> Self {
        Self {
            name: base_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string(),
            group: None,
            version: None,
            source_compatibility: None,
            target_compatibility: None,
            main_class: None,
            base_dir,
            build_file,
            tasks: Vec::new(),
            dependencies: Vec::new(),
            repositories: Vec::new(),
            plugins: Vec::new(),
            subprojects: Vec::new(),
        }
    }

    /// Find a task by name
    pub fn find_task(&self, name: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.name == name)
    }

    /// Get all task names
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.iter().map(|t| t.name.clone()).collect()
    }
}
