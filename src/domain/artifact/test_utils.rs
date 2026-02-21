//! Test utilities for artifact domain
//! Shared mock implementations for testing

use super::repositories::{ArtifactMetadata, ArtifactRepository};
use super::value_objects::ArtifactCoordinates;
use crate::domain::shared::value_objects::Version;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Mock repository for testing
/// Provides in-memory artifact storage with configurable behavior
#[derive(Clone)]
pub struct MockRepository {
    artifacts: HashMap<String, ArtifactMetadata>,
    repo_path: PathBuf,
}

impl MockRepository {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            repo_path: PathBuf::from("/tmp/test-repo"),
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            artifacts: HashMap::new(),
            repo_path: path,
        }
    }

    pub fn add_artifact(&mut self, coords: ArtifactCoordinates) {
        let metadata = ArtifactMetadata {
            coordinates: coords.clone(),
            dependencies: Vec::new(),
        };
        self.artifacts.insert(coords.gav(), metadata);
    }

    pub fn add_artifact_with_deps(
        &mut self,
        coords: ArtifactCoordinates,
        deps: Vec<ArtifactCoordinates>,
    ) {
        let metadata = ArtifactMetadata {
            coordinates: coords.clone(),
            dependencies: deps,
        };
        self.artifacts.insert(coords.gav(), metadata);
    }
}

impl Default for MockRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactRepository for MockRepository {
    fn install(&self, _coords: &ArtifactCoordinates, _file: PathBuf) -> Result<()> {
        Ok(())
    }

    fn exists(&self, coords: &ArtifactCoordinates) -> bool {
        self.artifacts.contains_key(&coords.gav())
    }

    fn path(&self) -> &PathBuf {
        &self.repo_path
    }

    fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata> {
        self.artifacts
            .get(&coordinates.gav())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Artifact not found"))
    }

    fn list_versions(&self, _coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
        Ok(vec![
            Version::new("1.0.0"),
            Version::new("1.1.0"),
            Version::new("2.0.0"),
        ])
    }

    fn download(&self, _coordinates: &ArtifactCoordinates) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}
