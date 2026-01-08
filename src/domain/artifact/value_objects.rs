//! Value objects for Artifact context

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Artifact coordinates (GAV) value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactCoordinates {
    group_id: String,
    artifact_id: String,
    version: String,
    classifier: Option<String>,
    extension: String,
}

impl ArtifactCoordinates {
    /// Creates new artifact coordinates with validation
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<Self> {
        let group_id = group_id.into();
        let artifact_id = artifact_id.into();
        let version = version.into();

        // Validate inputs
        if group_id.trim().is_empty() {
            return Err(anyhow!("Group ID cannot be empty"));
        }
        if artifact_id.trim().is_empty() {
            return Err(anyhow!("Artifact ID cannot be empty"));
        }
        if version.trim().is_empty() {
            return Err(anyhow!("Version cannot be empty"));
        }

        Ok(Self {
            group_id,
            artifact_id,
            version,
            classifier: None,
            extension: "jar".to_string(),
        })
    }

    /// Creates coordinates without validation (for internal use)
    pub fn new_unchecked(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
            version: version.into(),
            classifier: None,
            extension: "jar".to_string(),
        }
    }

    /// Parses coordinates from GAV string (groupId:artifactId:version)
    pub fn from_gav(gav: &str) -> Result<Self> {
        let parts: Vec<&str> = gav.split(':').collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid GAV format: {gav}"));
        }

        let mut coords = Self::new(parts[0], parts[1], parts[2])?;
        
        // Handle classifier and extension if present
        if parts.len() >= 4 {
            coords.classifier = Some(parts[3].to_string());
        }
        if parts.len() >= 5 {
            coords.extension = parts[4].to_string();
        }

        Ok(coords)
    }

    pub fn with_classifier(mut self, classifier: impl Into<String>) -> Self {
        self.classifier = Some(classifier.into());
        self
    }

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = extension.into();
        self
    }

    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn classifier(&self) -> Option<&str> {
        self.classifier.as_deref()
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// Returns the GAV string (groupId:artifactId:version)
    pub fn gav(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }

    /// Returns the full coordinates string with classifier and extension
    pub fn full_coordinates(&self) -> String {
        let mut coords = self.gav();
        if let Some(classifier) = &self.classifier {
            coords.push(':');
            coords.push_str(classifier);
        }
        coords.push(':');
        coords.push_str(&self.extension);
        coords
    }

    /// Checks if coordinates are valid
    pub fn is_valid(&self) -> bool {
        !self.group_id.trim().is_empty()
            && !self.artifact_id.trim().is_empty()
            && !self.version.trim().is_empty()
    }

    /// Returns the repository path for this artifact
    pub fn repository_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        
        // Convert group ID to path (e.g., org.example -> org/example)
        for part in self.group_id.split('.') {
            path.push(part);
        }
        
        path.push(&self.artifact_id);
        path.push(&self.version);
        
        // Build filename
        let mut filename = format!("{}-{}", self.artifact_id, self.version);
        if let Some(classifier) = &self.classifier {
            filename.push('-');
            filename.push_str(classifier);
        }
        filename.push('.');
        filename.push_str(&self.extension);
        
        path.push(filename);
        path
    }

    /// Returns the artifact filename
    pub fn filename(&self) -> String {
        let mut filename = format!("{}-{}", self.artifact_id, self.version);
        if let Some(classifier) = &self.classifier {
            filename.push('-');
            filename.push_str(classifier);
        }
        filename.push('.');
        filename.push_str(&self.extension);
        filename
    }
}

impl fmt::Display for ArtifactCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.gav())
    }
}

/// Dependency scope value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum Scope {
    #[default]
    Compile,
    Provided,
    Runtime,
    Test,
    System,
    Import,
}

impl Scope {
    pub fn as_str(&self) -> &str {
        match self {
            Scope::Compile => "compile",
            Scope::Provided => "provided",
            Scope::Runtime => "runtime",
            Scope::Test => "test",
            Scope::System => "system",
            Scope::Import => "import",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "compile" => Some(Scope::Compile),
            "provided" => Some(Scope::Provided),
            "runtime" => Some(Scope::Runtime),
            "test" => Some(Scope::Test),
            "system" => Some(Scope::System),
            "import" => Some(Scope::Import),
            _ => None,
        }
    }
}


/// Version range value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionRange {
    Exact(String),
    Range { min: String, max: String, min_inclusive: bool, max_inclusive: bool },
    Latest,
}

impl VersionRange {
    pub fn exact(version: impl Into<String>) -> Self {
        Self::Exact(version.into())
    }

    pub fn range(min: impl Into<String>, max: impl Into<String>) -> Self {
        Self::Range {
            min: min.into(),
            max: max.into(),
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    pub fn latest() -> Self {
        Self::Latest
    }

    pub fn matches(&self, version: &str) -> bool {
        match self {
            VersionRange::Exact(v) => v == version,
            VersionRange::Latest => true,
            VersionRange::Range { min, max, min_inclusive, max_inclusive } => {
                let min_match = if *min_inclusive {
                    version >= min.as_str()
                } else {
                    version > min.as_str()
                };
                let max_match = if *max_inclusive {
                    version <= max.as_str()
                } else {
                    version < max.as_str()
                };
                min_match && max_match
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_coordinates_new() {
        let coords = ArtifactCoordinates::new("org.example", "lib", "1.0.0").unwrap();
        assert_eq!(coords.group_id(), "org.example");
        assert_eq!(coords.artifact_id(), "lib");
        assert_eq!(coords.version(), "1.0.0");
        assert_eq!(coords.extension(), "jar");
    }

    #[test]
    fn test_artifact_coordinates_validation() {
        assert!(ArtifactCoordinates::new("", "lib", "1.0.0").is_err());
        assert!(ArtifactCoordinates::new("org.example", "", "1.0.0").is_err());
        assert!(ArtifactCoordinates::new("org.example", "lib", "").is_err());
    }

    #[test]
    fn test_artifact_coordinates_gav() {
        let coords = ArtifactCoordinates::new("org.example", "lib", "1.0.0").unwrap();
        assert_eq!(coords.gav(), "org.example:lib:1.0.0");
    }

    #[test]
    fn test_artifact_coordinates_from_gav() {
        let coords = ArtifactCoordinates::from_gav("org.example:lib:1.0.0").unwrap();
        assert_eq!(coords.group_id(), "org.example");
        assert_eq!(coords.artifact_id(), "lib");
        assert_eq!(coords.version(), "1.0.0");
    }

    #[test]
    fn test_artifact_coordinates_from_gav_with_classifier() {
        let coords = ArtifactCoordinates::from_gav("org.example:lib:1.0.0:sources:jar").unwrap();
        assert_eq!(coords.classifier(), Some("sources"));
        assert_eq!(coords.extension(), "jar");
    }

    #[test]
    fn test_artifact_coordinates_with_classifier() {
        let coords = ArtifactCoordinates::new("org.example", "lib", "1.0.0")
            .unwrap()
            .with_classifier("sources");
        assert_eq!(coords.classifier(), Some("sources"));
    }

    #[test]
    fn test_artifact_coordinates_repository_path() {
        let coords = ArtifactCoordinates::new("org.example", "lib", "1.0.0").unwrap();
        let path = coords.repository_path();
        assert_eq!(
            path.to_str().unwrap(),
            "org/example/lib/1.0.0/lib-1.0.0.jar"
        );
    }

    #[test]
    fn test_artifact_coordinates_filename() {
        let coords = ArtifactCoordinates::new("org.example", "lib", "1.0.0").unwrap();
        assert_eq!(coords.filename(), "lib-1.0.0.jar");

        let coords_with_classifier = coords.with_classifier("sources");
        assert_eq!(coords_with_classifier.filename(), "lib-1.0.0-sources.jar");
    }

    #[test]
    fn test_scope_from_str() {
        assert_eq!(Scope::from_str("compile"), Some(Scope::Compile));
        assert_eq!(Scope::from_str("test"), Some(Scope::Test));
        assert_eq!(Scope::from_str("invalid"), None);
    }

    #[test]
    fn test_version_range_exact() {
        let range = VersionRange::exact("1.0.0");
        assert!(range.matches("1.0.0"));
        assert!(!range.matches("1.0.1"));
    }

    #[test]
    fn test_version_range_range() {
        let range = VersionRange::range("1.0.0", "2.0.0");
        assert!(range.matches("1.0.0"));
        assert!(range.matches("1.5.0"));
        assert!(range.matches("2.0.0"));
        assert!(!range.matches("0.9.0"));
        assert!(!range.matches("2.1.0"));
    }

    #[test]
    fn test_version_range_latest() {
        let range = VersionRange::latest();
        assert!(range.matches("1.0.0"));
        assert!(range.matches("999.999.999"));
    }
}
