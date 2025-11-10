use serde::{Deserialize, Serialize};

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// Repository identifier.
    pub id: Option<String>,

    /// Human readable name of the repository.
    pub name: Option<String>,

    /// The URL of the repository.
    pub url: String,

    /// The layout of the repository (default or legacy).
    pub layout: Option<String>,

    /// Repository releases configuration.
    pub releases: Option<RepositoryPolicy>,

    /// Repository snapshots configuration.
    pub snapshots: Option<RepositoryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPolicy {
    /// Whether to enable this repository for this type.
    pub enabled: Option<bool>,

    /// The frequency of updates.
    pub update_policy: Option<String>,

    /// The checksum policy.
    pub checksum_policy: Option<String>,
}

impl Repository {
    /// Check if this is a snapshot repository
    pub fn is_snapshot_enabled(&self) -> bool {
        self.snapshots
            .as_ref()
            .and_then(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Check if this is a release repository
    pub fn is_release_enabled(&self) -> bool {
        self.releases
            .as_ref()
            .and_then(|r| r.enabled)
            .unwrap_or(true)
    }
}

