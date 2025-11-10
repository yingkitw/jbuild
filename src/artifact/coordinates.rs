use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Artifact coordinates (groupId:artifactId:version[:packaging[:classifier]])
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ArtifactCoordinates {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub packaging: Option<String>,
    pub classifier: Option<String>,
}

#[derive(Debug, Error)]
pub enum CoordinateParseError {
    #[error("Invalid coordinate format: {0}")]
    InvalidFormat(String),
}

impl ArtifactCoordinates {
    /// Create new coordinates
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
            version: version.into(),
            packaging: None,
            classifier: None,
        }
    }

    /// Get the identifier (groupId:artifactId)
    pub fn id(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }

    /// Get the full identifier (groupId:artifactId:version)
    pub fn full_id(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

impl FromStr for ArtifactCoordinates {
    type Err = CoordinateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 3 {
            return Err(CoordinateParseError::InvalidFormat(s.to_string()));
        }

        Ok(Self {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: parts[2].to_string(),
            packaging: parts.get(3).map(|s| s.to_string()),
            classifier: parts.get(4).map(|s| s.to_string()),
        })
    }
}

impl fmt::Display for ArtifactCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.group_id, self.artifact_id, self.version
        )?;
        if let Some(packaging) = &self.packaging {
            write!(f, ":{}", packaging)?;
        }
        if let Some(classifier) = &self.classifier {
            write!(f, ":{}", classifier)?;
        }
        Ok(())
    }
}

