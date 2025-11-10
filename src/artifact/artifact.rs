use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::artifact::coordinates::ArtifactCoordinates;

/// Maven artifact representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Artifact {
    /// Artifact coordinates
    pub coordinates: ArtifactCoordinates,

    /// Artifact file path (if resolved)
    pub file: Option<PathBuf>,

    /// Artifact base version (without snapshot qualifier)
    pub base_version: String,

    /// Whether this is a snapshot artifact
    pub is_snapshot: bool,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        let version = version.into();
        let is_snapshot = version.ends_with("-SNAPSHOT");
        let base_version = if is_snapshot {
            version.clone()
        } else {
            version.clone()
        };

        Self {
            coordinates: ArtifactCoordinates {
                group_id: group_id.into(),
                artifact_id: artifact_id.into(),
                version: version.into(),
                packaging: None,
                classifier: None,
            },
            file: None,
            base_version,
            is_snapshot,
        }
    }

    /// Get the artifact file name
    pub fn file_name(&self) -> String {
        let classifier = self
            .coordinates
            .classifier
            .as_ref()
            .map(|c| format!("-{}", c))
            .unwrap_or_default();
        let extension = self
            .coordinates
            .packaging
            .as_deref()
            .unwrap_or("jar");
        format!(
            "{}-{}{}.{}",
            self.coordinates.artifact_id, self.coordinates.version, classifier, extension
        )
    }

    /// Get the artifact path in a Maven repository
    pub fn repository_path(&self) -> PathBuf {
        let group_path = self.coordinates.group_id.replace('.', "/");
        let mut path = PathBuf::from(group_path);
        path.push(&self.coordinates.artifact_id);
        path.push(&self.coordinates.version);
        path.push(self.file_name());
        path
    }
}

impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.coordinates)
    }
}

