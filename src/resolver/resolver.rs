use std::collections::HashSet;

use crate::artifact::{Artifact, LocalRepository};
use crate::model::Dependency;
use crate::resolver::repository::RemoteRepository;
use crate::resolver::downloader::ArtifactDownloader;

/// Dependency resolver
pub struct DependencyResolver {
    local_repository: Box<dyn LocalRepository>,
    remote_repositories: Vec<RemoteRepository>,
    downloader: ArtifactDownloader,
}

impl DependencyResolver {
    pub fn new(local_repository: Box<dyn LocalRepository>) -> Self {
        Self {
            local_repository,
            remote_repositories: vec![RemoteRepository::new(
                "central",
                url::Url::parse("https://repo1.maven.org/maven2/").unwrap(),
            )],
            downloader: ArtifactDownloader::new(),
        }
    }

    pub fn with_remote_repositories(
        mut self,
        repositories: Vec<RemoteRepository>,
    ) -> Self {
        self.remote_repositories = repositories;
        self
    }

    /// Get remote repositories
    pub fn remote_repositories(&self) -> &[RemoteRepository] {
        &self.remote_repositories
    }

    /// Resolve a dependency to an artifact
    pub fn resolve_dependency(
        &self,
        dependency: &Dependency,
    ) -> anyhow::Result<Option<Artifact>> {
        let version = dependency.version.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Dependency {}:{} has no version", dependency.group_id, dependency.artifact_id)
        })?;

        let artifact = Artifact::new(
            &dependency.group_id,
            &dependency.artifact_id,
            version,
        );

        // Check local repository first
        if self.local_repository.artifact_exists(&artifact) {
            let mut resolved = artifact.clone();
            resolved.file = Some(self.local_repository.artifact_path(&artifact));
            return Ok(Some(resolved));
        }

        // Download from remote repositories
        let local_path = self.local_repository.artifact_path(&artifact);
        
        match self.downloader.download_from_repositories(
            &artifact,
            &self.remote_repositories,
            &local_path,
        ) {
            Ok(_) => {
                let mut resolved = artifact.clone();
                resolved.file = Some(local_path);
                Ok(Some(resolved))
            }
            Err(e) => {
                tracing::warn!("Failed to resolve dependency {}: {}", dependency.id(), e);
                Ok(None)
            }
        }
    }

    /// Resolve all dependencies for a project
    pub fn resolve_dependencies(
        &self,
        dependencies: &[Dependency],
    ) -> anyhow::Result<Vec<Artifact>> {
        let mut resolved = Vec::new();
        let mut seen = HashSet::new();

        for dependency in dependencies {
            let key = format!("{}:{}", dependency.group_id, dependency.artifact_id);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            if let Some(artifact) = self.resolve_dependency(dependency)? {
                resolved.push(artifact);
            }
        }

        Ok(resolved)
    }
}

