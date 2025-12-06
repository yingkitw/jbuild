//! Build Wrapper Support
//!
//! Detects and supports Maven Wrapper (mvnw) and Gradle Wrapper (gradlew).

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

/// Wrapper type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapperType {
    /// Maven Wrapper (mvnw)
    Maven,
    /// Gradle Wrapper (gradlew)
    Gradle,
}

/// Wrapper detection and execution
pub struct BuildWrapper {
    /// Wrapper type
    pub wrapper_type: WrapperType,
    /// Path to wrapper script
    pub script_path: PathBuf,
    /// Project base directory
    pub base_dir: PathBuf,
}

impl BuildWrapper {
    /// Detect wrapper in a directory
    pub fn detect(base_dir: &Path) -> Option<Self> {
        // Check for Gradle wrapper first (more common in modern projects)
        let gradlew = if cfg!(windows) {
            base_dir.join("gradlew.bat")
        } else {
            base_dir.join("gradlew")
        };
        
        if gradlew.exists() {
            return Some(Self {
                wrapper_type: WrapperType::Gradle,
                script_path: gradlew,
                base_dir: base_dir.to_path_buf(),
            });
        }

        // Check for Maven wrapper
        let mvnw = if cfg!(windows) {
            base_dir.join("mvnw.cmd")
        } else {
            base_dir.join("mvnw")
        };

        if mvnw.exists() {
            return Some(Self {
                wrapper_type: WrapperType::Maven,
                script_path: mvnw,
                base_dir: base_dir.to_path_buf(),
            });
        }

        None
    }

    /// Check if wrapper properties file exists
    pub fn has_properties(&self) -> bool {
        match self.wrapper_type {
            WrapperType::Maven => {
                self.base_dir.join(".mvn/wrapper/maven-wrapper.properties").exists()
            }
            WrapperType::Gradle => {
                self.base_dir.join("gradle/wrapper/gradle-wrapper.properties").exists()
            }
        }
    }

    /// Get the wrapper version from properties
    pub fn get_version(&self) -> Option<String> {
        let props_path = match self.wrapper_type {
            WrapperType::Maven => self.base_dir.join(".mvn/wrapper/maven-wrapper.properties"),
            WrapperType::Gradle => self.base_dir.join("gradle/wrapper/gradle-wrapper.properties"),
        };

        if let Ok(content) = std::fs::read_to_string(&props_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("distributionUrl") || line.starts_with("wrapperUrl") {
                    // Extract version from URL
                    if let Some(version) = extract_version_from_url(line) {
                        return Some(version);
                    }
                }
            }
        }

        None
    }

    /// Execute the wrapper with given arguments
    pub fn execute(&self, args: &[&str]) -> Result<std::process::Output> {
        let mut cmd = Command::new(&self.script_path);
        cmd.current_dir(&self.base_dir);
        cmd.args(args);

        cmd.output()
            .with_context(|| format!("Failed to execute wrapper: {:?}", self.script_path))
    }

    /// Execute wrapper and stream output
    pub fn execute_streaming(&self, args: &[&str]) -> Result<std::process::ExitStatus> {
        let mut cmd = Command::new(&self.script_path);
        cmd.current_dir(&self.base_dir);
        cmd.args(args);

        cmd.status()
            .with_context(|| format!("Failed to execute wrapper: {:?}", self.script_path))
    }
}

/// Extract version from distribution URL
fn extract_version_from_url(line: &str) -> Option<String> {
    // Maven: distributionUrl=https://repo.maven.apache.org/.../apache-maven-3.9.6-bin.zip
    // Gradle: distributionUrl=https\://services.gradle.org/.../gradle-8.5-bin.zip
    
    let url = line.split('=').nth(1)?;
    let url = url.replace("\\:", ":");
    
    // Find version pattern
    if url.contains("maven") {
        // apache-maven-X.Y.Z
        if let Some(start) = url.find("apache-maven-") {
            let rest = &url[start + 13..];
            if let Some(end) = rest.find('-') {
                return Some(rest[..end].to_string());
            }
        }
    } else if url.contains("gradle") {
        // gradle-X.Y
        if let Some(start) = url.find("gradle-") {
            let rest = &url[start + 7..];
            if let Some(end) = rest.find('-') {
                return Some(rest[..end].to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_maven_version() {
        let line = "distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.6/apache-maven-3.9.6-bin.zip";
        assert_eq!(extract_version_from_url(line), Some("3.9.6".to_string()));
    }

    #[test]
    fn test_extract_gradle_version() {
        let line = r"distributionUrl=https\://services.gradle.org/distributions/gradle-8.5-bin.zip";
        assert_eq!(extract_version_from_url(line), Some("8.5".to_string()));
    }

    #[test]
    fn test_wrapper_type() {
        assert_eq!(WrapperType::Maven, WrapperType::Maven);
        assert_ne!(WrapperType::Maven, WrapperType::Gradle);
    }
}
