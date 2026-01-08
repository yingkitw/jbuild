//! Domain services for Build System context

use super::value_objects::{BuildFile, BuildSystemType};
use std::path::Path;

/// Build system detector service
pub struct BuildSystemDetector;

impl BuildSystemDetector {
    /// Detects the build system from the project directory
    pub fn detect(project_dir: &Path) -> Option<BuildFile> {
        // Check for jbuild.toml
        let jbuild_toml = project_dir.join("jbuild.toml");
        if jbuild_toml.exists() {
            return Some(BuildFile::new(jbuild_toml, BuildSystemType::JBuild));
        }

        // Check for pom.xml
        let pom_xml = project_dir.join("pom.xml");
        if pom_xml.exists() {
            return Some(BuildFile::new(pom_xml, BuildSystemType::Maven));
        }

        // Check for build.gradle or build.gradle.kts
        let build_gradle = project_dir.join("build.gradle");
        if build_gradle.exists() {
            return Some(BuildFile::new(build_gradle, BuildSystemType::Gradle));
        }

        let build_gradle_kts = project_dir.join("build.gradle.kts");
        if build_gradle_kts.exists() {
            return Some(BuildFile::new(build_gradle_kts, BuildSystemType::Gradle));
        }

        None
    }

    /// Checks if a directory contains a build file
    pub fn has_build_file(project_dir: &Path) -> bool {
        Self::detect(project_dir).is_some()
    }

    /// Gets the build system type for a directory
    pub fn get_build_type(project_dir: &Path) -> Option<BuildSystemType> {
        Self::detect(project_dir).map(|bf| bf.build_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_maven() {
        let temp = TempDir::new().unwrap();
        let pom = temp.path().join("pom.xml");
        fs::write(&pom, "<project></project>").unwrap();

        let result = BuildSystemDetector::detect(temp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().build_type(), BuildSystemType::Maven);
    }

    #[test]
    fn test_detect_gradle() {
        let temp = TempDir::new().unwrap();
        let build_gradle = temp.path().join("build.gradle");
        fs::write(&build_gradle, "plugins { id 'java' }").unwrap();

        let result = BuildSystemDetector::detect(temp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().build_type(), BuildSystemType::Gradle);
    }

    #[test]
    fn test_detect_jbuild() {
        let temp = TempDir::new().unwrap();
        let jbuild_toml = temp.path().join("jbuild.toml");
        fs::write(&jbuild_toml, "[package]\nname = \"test\"").unwrap();

        let result = BuildSystemDetector::detect(temp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().build_type(), BuildSystemType::JBuild);
    }

    #[test]
    fn test_no_build_file() {
        let temp = TempDir::new().unwrap();
        let result = BuildSystemDetector::detect(temp.path());
        assert!(result.is_none());
    }
}
