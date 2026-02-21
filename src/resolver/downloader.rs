use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::time::Duration;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use sha1::{Sha1, Digest};
use md5;

use crate::artifact::Artifact;
use crate::resolver::repository::RemoteRepository;
use crate::resolver::metadata::RepositoryMetadata;

/// Download configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub verify_checksums: bool,
    pub show_progress: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            verify_checksums: true,
            show_progress: true,
        }
    }
}

/// Artifact downloader - downloads artifacts from remote repositories
pub struct ArtifactDownloader {
    client: Client,
    config: DownloadConfig,
}

impl ArtifactDownloader {
    pub fn new() -> Self {
        Self::with_config(DownloadConfig::default())
    }

    pub fn with_config(config: DownloadConfig) -> Self {
        Self {
            client: Client::builder()
                .user_agent("jbuild/0.1.0")
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            config,
        }
    }

    /// Fetch repository metadata
    pub fn fetch_metadata(
        &self,
        group_id: &str,
        artifact_id: &str,
        repository: &RemoteRepository,
    ) -> Result<RepositoryMetadata> {
        let metadata_url = repository.metadata_url(group_id, artifact_id);
        
        tracing::debug!("Fetching metadata from {}", metadata_url);
        
        let response = self.client
            .get(metadata_url.as_str())
            .send()
            .with_context(|| format!("Failed to fetch metadata from {metadata_url}"))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch metadata: HTTP {}",
                response.status()
            ));
        }

        let metadata_xml = response.text()
            .with_context(|| "Failed to read metadata response")?;
        
        RepositoryMetadata::parse(&metadata_xml)
            .context("Failed to parse metadata")
    }

    /// Download an artifact from a remote repository with retry logic
    pub fn download(
        &self,
        artifact: &Artifact,
        repository: &RemoteRepository,
        target_path: &PathBuf,
    ) -> Result<()> {
        let artifact_url = repository.artifact_url(artifact);
        
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tracing::info!(
                    "Retrying download of {} (attempt {}/{})",
                    artifact,
                    attempt + 1,
                    self.config.max_retries + 1
                );
                std::thread::sleep(self.config.retry_delay * attempt);
            }

            match self.download_internal(artifact, &artifact_url, target_path) {
                Ok(()) => {
                    if self.config.verify_checksums
                        && let Err(e) = self.verify_checksums(artifact, repository, target_path) {
                            tracing::warn!("Checksum verification failed: {}", e);
                            // Continue anyway, but log the warning
                        }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Download failed after all retries")))
    }

    fn download_internal(
        &self,
        artifact: &Artifact,
        artifact_url: &url::Url,
        target_path: &PathBuf,
    ) -> Result<()> {
        tracing::info!("Downloading {} from {}", artifact, artifact_url);

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {parent:?}"))?;
        }

        // Download the artifact
        let mut response = self.client
            .get(artifact_url.as_str())
            .send()
            .with_context(|| format!("Failed to download artifact from {artifact_url}"))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download artifact: HTTP {}",
                response.status()
            ));
        }

        // Get content length for progress reporting
        let content_length = response.content_length();
        
        // Write to file with progress reporting
        let mut file = fs::File::create(target_path)
            .with_context(|| format!("Failed to create file: {target_path:?}"))?;
        
        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = response.read(&mut buffer)
                .with_context(|| "Failed to read response")?;
            
            if bytes_read == 0 {
                break;
            }
            
            file.write_all(&buffer[..bytes_read])
                .with_context(|| format!("Failed to write to {target_path:?}"))?;
            
            downloaded += bytes_read as u64;
            
            if self.config.show_progress {
                if let Some(total) = content_length {
                    let percent = (downloaded * 100) / total;
                    if downloaded.is_multiple_of(1024 * 1024) || downloaded == total {
                        tracing::info!(
                            "Downloaded {} / {} bytes ({}%)",
                            downloaded,
                            total,
                            percent
                        );
                    }
                } else if downloaded.is_multiple_of(1024 * 1024) {
                    tracing::info!("Downloaded {} bytes", downloaded);
                }
            }
        }

        tracing::info!("Successfully downloaded {} to {:?}", artifact, target_path);
        Ok(())
    }

    /// Verify checksums (SHA-1 and MD5) for a downloaded artifact
    fn verify_checksums(
        &self,
        artifact: &Artifact,
        repository: &RemoteRepository,
        artifact_path: &PathBuf,
    ) -> Result<()> {
        // Read the artifact file
        let artifact_data = fs::read(artifact_path)
            .with_context(|| format!("Failed to read artifact: {artifact_path:?}"))?;

        // Calculate local checksums
        let mut sha1_hasher = Sha1::new();
        sha1_hasher.update(&artifact_data);
        let local_sha1 = hex::encode(sha1_hasher.finalize());

        let local_md5 = hex::encode(md5::compute(&artifact_data).0);

        // Download and verify SHA-1
        let sha1_url = format!("{}.sha1", repository.artifact_url(artifact));
        if let Ok(remote_sha1) = self.download_checksum(&sha1_url) {
            let remote_sha1 = remote_sha1.trim();
            if local_sha1 != remote_sha1 {
                return Err(anyhow::anyhow!(
                    "SHA-1 checksum mismatch: local={local_sha1}, remote={remote_sha1}"
                ));
            }
            tracing::debug!("SHA-1 checksum verified");
        }

        // Download and verify MD5
        let md5_url = format!("{}.md5", repository.artifact_url(artifact));
        if let Ok(remote_md5) = self.download_checksum(&md5_url) {
            let remote_md5 = remote_md5.trim();
            if local_md5 != remote_md5 {
                return Err(anyhow::anyhow!(
                    "MD5 checksum mismatch: local={local_md5}, remote={remote_md5}"
                ));
            }
            tracing::debug!("MD5 checksum verified");
        }

        Ok(())
    }

    fn download_checksum(&self, checksum_url: &str) -> Result<String> {
        let response = self.client
            .get(checksum_url)
            .send()
            .with_context(|| format!("Failed to download checksum from {checksum_url}"))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download checksum: HTTP {}",
                response.status()
            ));
        }

        response.text()
            .with_context(|| "Failed to read checksum response")
    }

    /// Download artifact POM
    pub fn download_pom(
        &self,
        artifact: &Artifact,
        repository: &RemoteRepository,
        target_path: &PathBuf,
    ) -> Result<()> {
        // Create a POM artifact
        let mut pom_artifact = artifact.clone();
        pom_artifact.coordinates.packaging = Some("pom".to_string());

        self.download(&pom_artifact, repository, target_path)
    }

    /// Try downloading from multiple repositories
    pub fn download_from_repositories(
        &self,
        artifact: &Artifact,
        repositories: &[RemoteRepository],
        target_path: &PathBuf,
    ) -> Result<()> {
        for repository in repositories {
            match self.download(artifact, repository, target_path) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!("Failed to download from {}: {}", repository.id, e);
                    continue;
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to download {artifact} from all repositories"
        ))
    }
}

impl Default for ArtifactDownloader {
    fn default() -> Self {
        Self::new()
    }
}

