use serde::{Deserialize, Serialize};

/// Distribution management configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DistributionManagement {
    /// Repository for deployment.
    pub repository: Option<DeploymentRepository>,

    /// Snapshot repository for deployment.
    pub snapshot_repository: Option<DeploymentRepository>,

    /// Site distribution information.
    pub site: Option<Site>,

    /// Download URL.
    pub download_url: Option<String>,

    /// Relocation information.
    pub relocation: Option<Relocation>,

    /// Status of the artifact in the remote repository.
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRepository {
    /// Repository identifier.
    pub id: Option<String>,

    /// Repository name.
    pub name: Option<String>,

    /// Repository URL.
    pub url: String,

    /// Repository layout.
    pub layout: Option<String>,

    /// Whether the repository is unique.
    pub unique_version: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    /// Site identifier.
    pub id: Option<String>,

    /// Site name.
    pub name: Option<String>,

    /// Site URL.
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Relocation {
    /// The group ID of the artifact to relocate to.
    pub group_id: Option<String>,

    /// The artifact ID of the artifact to relocate to.
    pub artifact_id: Option<String>,

    /// The version of the artifact to relocate to.
    pub version: Option<String>,

    /// An additional message to show the user about the relocation.
    pub message: Option<String>,
}

