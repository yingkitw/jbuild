use std::path::PathBuf;
use url::Url;

use crate::artifact::Artifact;

/// Remote repository configuration
#[derive(Debug, Clone)]
pub struct RemoteRepository {
    pub id: String,
    pub url: Url,
    pub releases_enabled: bool,
    pub snapshots_enabled: bool,
}

impl RemoteRepository {
    pub fn new(id: impl Into<String>, url: Url) -> Self {
        Self {
            id: id.into(),
            url,
            releases_enabled: true,
            snapshots_enabled: false,
        }
    }

    /// Get the URL for an artifact in this repository
    pub fn artifact_url(&self, artifact: &Artifact) -> Url {
        let group_path = artifact.coordinates.group_id.replace('.', "/");
        let mut path = format!("{}/{}/", group_path, artifact.coordinates.artifact_id);
        path.push_str(&artifact.coordinates.version);
        path.push('/');
        path.push_str(&artifact.file_name());

        self.url.join(&path).unwrap_or_else(|_| self.url.clone())
    }

    /// Get the URL for maven-metadata.xml for a groupId/artifactId
    pub fn metadata_url(&self, group_id: &str, artifact_id: &str) -> Url {
        let group_path = group_id.replace('.', "/");
        let path = format!("{}/{}/maven-metadata.xml", group_path, artifact_id);
        self.url.join(&path).unwrap_or_else(|_| self.url.clone())
    }

    /// Get the URL for maven-metadata.xml for a specific version (for snapshots)
    pub fn version_metadata_url(&self, group_id: &str, artifact_id: &str, version: &str) -> Url {
        let group_path = group_id.replace('.', "/");
        let path = format!("{}/{}/{}/maven-metadata.xml", group_path, artifact_id, version);
        self.url.join(&path).unwrap_or_else(|_| self.url.clone())
    }
}

/// Repository manager
pub trait RepositoryManager {
    /// Resolve an artifact from repositories
    fn resolve(&self, artifact: &Artifact) -> anyhow::Result<Option<PathBuf>>;

    /// Download an artifact to the local repository
    fn download(&self, artifact: &Artifact) -> anyhow::Result<PathBuf>;
}

