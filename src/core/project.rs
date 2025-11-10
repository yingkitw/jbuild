use std::path::PathBuf;

use crate::model::Model;

/// Maven project representation
#[derive(Debug, Clone)]
pub struct MavenProject {
    /// Project model (POM)
    pub model: Model,

    /// Project base directory
    pub basedir: PathBuf,

    /// Project build directory
    pub build_directory: PathBuf,

    /// Project output directory
    pub output_directory: PathBuf,

    /// Project test output directory
    pub test_output_directory: PathBuf,
}

impl MavenProject {
    /// Create a new Maven project
    pub fn new(model: Model, basedir: PathBuf) -> Self {
        let build_dir = basedir.join("target");
        let output_dir = build_dir.join("classes");
        let test_output_dir = build_dir.join("test-classes");

        Self {
            model,
            basedir,
            build_directory: build_dir,
            output_directory: output_dir,
            test_output_directory: test_output_dir,
        }
    }

    /// Get the project identifier
    pub fn id(&self) -> String {
        self.model.id()
    }

    /// Get the project coordinates
    pub fn coordinates(&self) -> (&str, &str, &str) {
        self.model.coordinates()
    }
}

