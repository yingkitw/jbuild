use std::path::PathBuf;

use crate::artifact::artifact::Artifact;

/// Local repository interface
pub trait LocalRepository: Send + Sync {
    /// Get the base directory of the repository
    fn base_directory(&self) -> &PathBuf;

    /// Get the path for an artifact in the repository
    fn artifact_path(&self, artifact: &Artifact) -> PathBuf {
        let mut path = self.base_directory().clone();
        path.push(artifact.repository_path());
        path
    }

    /// Check if an artifact exists in the repository
    fn artifact_exists(&self, artifact: &Artifact) -> bool {
        self.artifact_path(artifact).exists()
    }
}

/// Default local repository implementation
pub struct DefaultLocalRepository {
    base_directory: PathBuf,
}

impl DefaultLocalRepository {
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }
}

impl Default for DefaultLocalRepository {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".m2");
        path.push("repository");
        Self::new(path)
    }
}

impl LocalRepository for DefaultLocalRepository {
    fn base_directory(&self) -> &PathBuf {
        &self.base_directory
    }
}
