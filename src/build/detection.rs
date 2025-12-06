//! Build system detection
//! 
//! Detects which build system (Maven, Gradle) should be used for a project.

use std::path::{Path, PathBuf};

/// Supported build systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    /// Apache Maven (pom.xml)
    Maven,
    /// Gradle (build.gradle, build.gradle.kts)
    Gradle,
}

impl BuildSystem {
    /// Detect the build system for a given directory
    pub fn detect(base_dir: &Path) -> Option<Self> {
        // Check for Maven
        if base_dir.join("pom.xml").exists() {
            return Some(BuildSystem::Maven);
        }

        // Check for Gradle
        if base_dir.join("build.gradle").exists() 
            || base_dir.join("build.gradle.kts").exists() {
            return Some(BuildSystem::Gradle);
        }

        // Check parent directories (for multi-module projects)
        if let Some(parent) = base_dir.parent() {
            return Self::detect(parent);
        }

        None
    }

    /// Get the primary build file name for this build system
    pub fn build_file_name(&self) -> &'static str {
        match self {
            BuildSystem::Maven => "pom.xml",
            BuildSystem::Gradle => "build.gradle",
        }
    }

    /// Get the build file path in a directory
    pub fn build_file_path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.build_file_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    

    #[test]
    fn test_detect_maven() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_maven");
        std::fs::create_dir_all(&temp_dir).ok();
        let pom_path = temp_dir.join("pom.xml");
        fs::write(&pom_path, "<project></project>").unwrap();

        assert_eq!(
            BuildSystem::detect(&temp_dir),
            Some(BuildSystem::Maven)
        );
        
        fs::remove_file(&pom_path).ok();
    }

    #[test]
    fn test_detect_gradle() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_gradle");
        std::fs::create_dir_all(&temp_dir).ok();
        let gradle_path = temp_dir.join("build.gradle");
        fs::write(&gradle_path, "plugins { }").unwrap();

        assert_eq!(
            BuildSystem::detect(&temp_dir),
            Some(BuildSystem::Gradle)
        );
        
        fs::remove_file(&gradle_path).ok();
    }

    #[test]
    fn test_detect_none() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_none");
        std::fs::create_dir_all(&temp_dir).ok();
        assert_eq!(BuildSystem::detect(&temp_dir), None);
    }
}

