use serde::{Deserialize, Serialize};
use anyhow::Result;
use quick_xml::de::from_str;
use crate::model::normalize_xml_namespaces;

/// Repository metadata from maven-metadata.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub group_id: String,
    pub artifact_id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub versioning: Option<Versioning>,
}

/// Versioning information from metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Versioning {
    #[serde(default)]
    pub latest: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub versions: Versions,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub snapshot: Option<Snapshot>,
    #[serde(default)]
    pub snapshot_versions: Option<SnapshotVersions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Versions {
    #[serde(default, rename = "version")]
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub build_number: Option<i32>,
    #[serde(default)]
    pub local_copy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotVersions {
    #[serde(default, rename = "snapshotVersion")]
    pub snapshot_versions: Vec<SnapshotVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotVersion {
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

impl RepositoryMetadata {
    pub fn new(group_id: String, artifact_id: String) -> Self {
        Self {
            group_id,
            artifact_id,
            version: None,
            versioning: None,
        }
    }

    /// Parse maven-metadata.xml content
    pub fn parse(metadata_xml: &str) -> Result<Self> {
        // Normalize XML namespaces
        let normalized = normalize_xml_namespaces(metadata_xml)
            .map_err(|e| anyhow::anyhow!("Failed to normalize metadata XML: {}", e))?;
        
        // Parse the metadata
        let metadata: RepositoryMetadata = from_str(&normalized)
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata XML: {}", e))?;
        
        Ok(metadata)
    }

    /// Get all available versions
    pub fn versions(&self) -> Vec<String> {
        self.versioning
            .as_ref()
            .map(|v| v.versions.versions.clone())
            .unwrap_or_default()
    }

    /// Get the latest version
    pub fn latest(&self) -> Option<&String> {
        self.versioning.as_ref()?.latest.as_ref()
    }

    /// Get the release version
    pub fn release(&self) -> Option<&String> {
        self.versioning.as_ref()?.release.as_ref()
    }

    /// Get the last updated timestamp
    pub fn last_updated(&self) -> Option<&String> {
        self.versioning.as_ref()?.last_updated.as_ref()
    }
}

