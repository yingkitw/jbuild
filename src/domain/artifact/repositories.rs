//! Repository traits and implementations for Artifact context

use super::value_objects::ArtifactCoordinates;
use crate::domain::shared::value_objects::Version;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Artifact metadata from repository
#[derive(Debug, Clone)]
pub struct ArtifactMetadata {
    pub coordinates: ArtifactCoordinates,
    pub dependencies: Vec<ArtifactCoordinates>,
}

/// Repository for artifact storage and retrieval
pub trait ArtifactRepository: Send + Sync {
    /// Find an artifact in the repository
    fn find(&self, coords: &ArtifactCoordinates) -> Result<Option<PathBuf>> {
        if self.exists(coords) {
            Ok(Some(self.path().join(coords.repository_path())))
        } else {
            Ok(None)
        }
    }

    /// Install an artifact to the repository
    fn install(&self, coords: &ArtifactCoordinates, file: PathBuf) -> Result<()>;

    /// Check if an artifact exists
    fn exists(&self, coords: &ArtifactCoordinates) -> bool;

    /// Get the repository path
    fn path(&self) -> &PathBuf;

    /// Get artifact metadata (for dependency resolution)
    fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata>;

    /// List available versions for an artifact
    fn list_versions(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<Version>>;

    /// Download an artifact
    fn download(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<u8>>;
}

/// Local repository implementation (e.g., ~/.m2/repository)
pub struct LocalRepository {
    base_path: PathBuf,
}

impl LocalRepository {
    /// Create a new local repository
    pub fn new(base_path: PathBuf) -> Result<Self> {
        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;
        }
        Ok(Self { base_path })
    }

    /// Get default Maven local repository (~/.m2/repository)
    pub fn default_maven() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let repo_path = home.join(".m2").join("repository");
        Self::new(repo_path)
    }

    /// Get artifact file path
    fn artifact_path(&self, coords: &ArtifactCoordinates) -> PathBuf {
        self.base_path.join(coords.repository_path())
    }

    /// Get POM file path
    fn pom_path(&self, coords: &ArtifactCoordinates) -> PathBuf {
        let pom_file = format!("{}-{}.pom", coords.artifact_id(), coords.version());
        self.base_path
            .join(coords.group_id().replace('.', "/"))
            .join(coords.artifact_id())
            .join(coords.version())
            .join(pom_file)
    }

    /// Parse POM file for metadata
    fn parse_pom(&self, pom_path: &PathBuf) -> Result<ArtifactMetadata> {
        if !pom_path.exists() {
            return Err(anyhow!("POM file not found: {pom_path:?}"));
        }

        let content = fs::read_to_string(pom_path)?;
        let dependencies = self.extract_dependencies_from_pom(&content)?;

        let coords = self.extract_coordinates_from_pom(&content)?;

        Ok(ArtifactMetadata {
            coordinates: coords,
            dependencies,
        })
    }

    fn extract_coordinates_from_pom(&self, _content: &str) -> Result<ArtifactCoordinates> {
        ArtifactCoordinates::from_gav("unknown:unknown:unknown")
    }

    fn extract_dependencies_from_pom(&self, _content: &str) -> Result<Vec<ArtifactCoordinates>> {
        Ok(Vec::new())
    }
}

impl ArtifactRepository for LocalRepository {
    fn install(&self, coords: &ArtifactCoordinates, source: PathBuf) -> Result<()> {
        let dest = self.artifact_path(coords);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source, dest)?;
        Ok(())
    }

    fn exists(&self, coords: &ArtifactCoordinates) -> bool {
        self.artifact_path(coords).exists()
    }

    fn path(&self) -> &PathBuf {
        &self.base_path
    }

    fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata> {
        let pom_path = self.pom_path(coordinates);
        self.parse_pom(&pom_path)
    }

    fn list_versions(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
        let group_path = self
            .base_path
            .join(coordinates.group_id().replace('.', "/"))
            .join(coordinates.artifact_id());

        if !group_path.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in fs::read_dir(group_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(name) = entry.file_name().to_str() {
                    versions.push(Version::new(name));
                }
        }

        Ok(versions)
    }

    fn download(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<u8>> {
        let path = self.artifact_path(coordinates);
        if !path.exists() {
            return Err(anyhow!("Artifact not found: {path:?}"));
        }
        fs::read(&path).map_err(|e| anyhow!("Failed to read artifact: {e}"))
    }
}

/// Remote repository implementation (e.g., Maven Central)
pub struct RemoteRepository {
    url: String,
    cache_dir: PathBuf,
}

impl RemoteRepository {
    /// Create a new remote repository
    pub fn new(url: String, cache_dir: PathBuf) -> Result<Self> {
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        Ok(Self { url, cache_dir })
    }

    /// Create Maven Central repository
    pub fn maven_central() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let cache = home.join(".jbuild").join("cache").join("maven-central");
        Self::new("https://repo1.maven.org/maven2".to_string(), cache)
    }

    /// Get artifact URL
    fn artifact_url(&self, coords: &ArtifactCoordinates) -> String {
        format!("{}/{}", self.url, coords.repository_path().display())
    }

    /// Get cached artifact path
    fn cache_path(&self, coords: &ArtifactCoordinates) -> PathBuf {
        self.cache_dir.join(coords.repository_path())
    }
}

impl ArtifactRepository for RemoteRepository {
    fn install(&self, coords: &ArtifactCoordinates, source: PathBuf) -> Result<()> {
        let dest = self.cache_path(coords);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source, dest)?;
        Ok(())
    }

    fn exists(&self, coords: &ArtifactCoordinates) -> bool {
        self.cache_path(coords).exists()
    }

    fn path(&self) -> &PathBuf {
        &self.cache_dir
    }

    fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata> {
        Ok(ArtifactMetadata {
            coordinates: coordinates.clone(),
            dependencies: Vec::new(),
        })
    }

    fn list_versions(&self, _coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
        Ok(Vec::new())
    }

    fn download(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<u8>> {
        let cached = self.cache_path(coordinates);
        if cached.exists() {
            return fs::read(&cached).map_err(|e| anyhow!("Failed to read cached artifact: {e}"));
        }

        Err(anyhow!("Remote download not yet implemented"))
    }
}

/// Repository chain for fallback logic
pub struct RepositoryChain {
    repositories: Vec<Box<dyn ArtifactRepository>>,
}

impl RepositoryChain {
    /// Create a new repository chain
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Add a repository to the chain
    pub fn add_repository(&mut self, repo: Box<dyn ArtifactRepository>) {
        self.repositories.push(repo);
    }

    /// Create default chain (local -> Maven Central)
    pub fn new_default() -> Result<Self> {
        let mut chain = Self::new();
        chain.add_repository(Box::new(LocalRepository::default_maven()?));
        chain.add_repository(Box::new(RemoteRepository::maven_central()?));
        Ok(chain)
    }
}

impl ArtifactRepository for RepositoryChain {
    fn install(&self, coords: &ArtifactCoordinates, file: PathBuf) -> Result<()> {
        if let Some(repo) = self.repositories.first() {
            repo.install(coords, file)
        } else {
            Err(anyhow!("No repositories in chain"))
        }
    }

    fn exists(&self, coords: &ArtifactCoordinates) -> bool {
        self.repositories.iter().any(|repo| repo.exists(coords))
    }

    fn path(&self) -> &PathBuf {
        self.repositories
            .first()
            .map(|repo| repo.path())
            .unwrap_or_else(|| {
                static DEFAULT_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
                DEFAULT_PATH.get_or_init(|| PathBuf::from("."))
            })
    }

    fn get_metadata(&self, coordinates: &ArtifactCoordinates) -> Result<ArtifactMetadata> {
        for repo in &self.repositories {
            if let Ok(metadata) = repo.get_metadata(coordinates) {
                return Ok(metadata);
            }
        }
        Err(anyhow!("Artifact metadata not found in any repository"))
    }

    fn list_versions(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<Version>> {
        for repo in &self.repositories {
            if let Ok(versions) = repo.list_versions(coordinates)
                && !versions.is_empty() {
                    return Ok(versions);
                }
        }
        Ok(Vec::new())
    }

    fn download(&self, coordinates: &ArtifactCoordinates) -> Result<Vec<u8>> {
        for repo in &self.repositories {
            if let Ok(data) = repo.download(coordinates) {
                return Ok(data);
            }
        }
        Err(anyhow!("Artifact not found in any repository"))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_local_repository_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo = LocalRepository::new(temp_dir.path().to_path_buf());
        assert!(repo.is_ok());
    }

    #[test]
    fn test_local_repository_install_and_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = LocalRepository::new(temp_dir.path().to_path_buf()).unwrap();

        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();

        let source_file = temp_dir.path().join("test.jar");
        fs::write(&source_file, b"test content").unwrap();

        assert!(!repo.exists(&coords));

        let result = repo.install(&coords, source_file);
        assert!(result.is_ok());

        assert!(repo.exists(&coords));
    }

    #[test]
    fn test_local_repository_list_versions() {
        let temp_dir = TempDir::new().unwrap();
        let repo = LocalRepository::new(temp_dir.path().to_path_buf()).unwrap();

        let coords1 = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        let coords2 = ArtifactCoordinates::from_gav("com.example:test:2.0.0").unwrap();

        let source1 = temp_dir.path().join("test1.jar");
        let source2 = temp_dir.path().join("test2.jar");
        fs::write(&source1, b"test1").unwrap();
        fs::write(&source2, b"test2").unwrap();

        repo.install(&coords1, source1).unwrap();
        repo.install(&coords2, source2).unwrap();

        let versions = repo.list_versions(&coords1).unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_local_repository_download() {
        let temp_dir = TempDir::new().unwrap();
        let repo = LocalRepository::new(temp_dir.path().to_path_buf()).unwrap();

        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();

        let source_file = temp_dir.path().join("test.jar");
        let content = b"test artifact content";
        fs::write(&source_file, content).unwrap();

        repo.install(&coords, source_file).unwrap();

        let downloaded = repo.download(&coords).unwrap();
        assert_eq!(downloaded, content);
    }

    #[test]
    fn test_local_repository_download_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let repo = LocalRepository::new(temp_dir.path().to_path_buf()).unwrap();

        let coords = ArtifactCoordinates::from_gav("com.example:nonexistent:1.0.0").unwrap();

        let result = repo.download(&coords);
        assert!(result.is_err());
    }

    #[test]
    fn test_remote_repository_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo = RemoteRepository::new(
            "https://repo1.maven.org/maven2".to_string(),
            temp_dir.path().to_path_buf(),
        );
        assert!(repo.is_ok());
    }

    #[test]
    fn test_remote_repository_cache() {
        let temp_dir = TempDir::new().unwrap();
        let repo = RemoteRepository::new(
            "https://repo1.maven.org/maven2".to_string(),
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();

        let source_file = temp_dir.path().join("test.jar");
        fs::write(&source_file, b"cached content").unwrap();

        repo.install(&coords, source_file).unwrap();

        assert!(repo.exists(&coords));

        let cached = repo.download(&coords).unwrap();
        assert_eq!(cached, b"cached content");
    }

    #[test]
    fn test_repository_chain_creation() {
        let chain = RepositoryChain::new();
        assert_eq!(chain.repositories.len(), 0);
    }

    #[test]
    fn test_repository_chain_add_repository() {
        let temp_dir = TempDir::new().unwrap();
        let mut chain = RepositoryChain::new();

        let repo = LocalRepository::new(temp_dir.path().to_path_buf()).unwrap();
        chain.add_repository(Box::new(repo));

        assert_eq!(chain.repositories.len(), 1);
    }

    #[test]
    fn test_repository_chain_fallback() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let repo1 = LocalRepository::new(temp_dir1.path().to_path_buf()).unwrap();
        let repo2 = LocalRepository::new(temp_dir2.path().to_path_buf()).unwrap();

        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();

        let source_file = temp_dir2.path().join("test.jar");
        fs::write(&source_file, b"from repo2").unwrap();
        repo2.install(&coords, source_file).unwrap();

        let mut chain = RepositoryChain::new();
        chain.add_repository(Box::new(repo1));
        chain.add_repository(Box::new(repo2));

        assert!(chain.exists(&coords));

        let data = chain.download(&coords).unwrap();
        assert_eq!(data, b"from repo2");
    }

    #[test]
    fn test_repository_chain_install_to_first() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let repo1 = LocalRepository::new(temp_dir1.path().to_path_buf()).unwrap();
        let repo2 = LocalRepository::new(temp_dir2.path().to_path_buf()).unwrap();

        let mut chain = RepositoryChain::new();
        chain.add_repository(Box::new(repo1));
        chain.add_repository(Box::new(repo2));

        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        let source_file = temp_dir1.path().join("test.jar");
        fs::write(&source_file, b"test").unwrap();

        let result = chain.install(&coords, source_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repository_chain_empty_error() {
        let chain = RepositoryChain::new();
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.jar");
        fs::write(&temp_file, b"test").unwrap();

        let result = chain.install(&coords, temp_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_artifact_metadata() {
        let coords = ArtifactCoordinates::from_gav("com.example:test:1.0.0").unwrap();
        let dep1 = ArtifactCoordinates::from_gav("com.example:dep1:1.0.0").unwrap();
        let dep2 = ArtifactCoordinates::from_gav("com.example:dep2:2.0.0").unwrap();

        let metadata = ArtifactMetadata {
            coordinates: coords.clone(),
            dependencies: vec![dep1, dep2],
        };

        assert_eq!(metadata.coordinates.group_id(), "com.example");
        assert_eq!(metadata.dependencies.len(), 2);
    }
}
