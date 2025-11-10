use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};
use zip::{ZipWriter, CompressionMethod};
use zip::write::FileOptions;

use crate::packaging::{Manifest, ResourceCollector, ResourceFilter};

/// JAR file builder
pub struct JarBuilder {
    manifest: Manifest,
    classes_dir: Option<PathBuf>,
    resources: Vec<(PathBuf, PathBuf)>, // (source, target_in_jar)
}

impl JarBuilder {
    pub fn new() -> Self {
        Self {
            manifest: Manifest::default(),
            classes_dir: None,
            resources: Vec::new(),
        }
    }

    /// Set the manifest
    pub fn with_manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = manifest;
        self
    }

    /// Set the classes directory (compiled classes)
    pub fn with_classes_dir(mut self, classes_dir: PathBuf) -> Self {
        self.classes_dir = Some(classes_dir);
        self
    }

    /// Add a resource file
    pub fn add_resource(mut self, source: PathBuf, target: PathBuf) -> Self {
        self.resources.push((source, target));
        self
    }

    /// Add resources from a directory
    pub fn add_resources_from_dir(
        mut self,
        source_dir: PathBuf,
        target_path: Option<String>,
        filter: ResourceFilter,
    ) -> Result<Self> {
        let resources = ResourceCollector::collect_resources(
            &source_dir,
            target_path.as_deref(),
            &filter,
        )?;
        self.resources.extend(resources);
        Ok(self)
    }

    /// Build the JAR file
    pub fn build(&self, output_path: &Path) -> Result<()> {
        tracing::info!("Creating JAR file: {:?}", output_path);

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let file = File::create(output_path)
            .with_context(|| format!("Failed to create JAR file: {:?}", output_path))?;

        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated);

        // Write manifest
        let manifest_bytes = self.manifest.to_bytes()?;
        zip.start_file("META-INF/MANIFEST.MF", options)
            .context("Failed to start manifest file")?;
        zip.write_all(&manifest_bytes)
            .context("Failed to write manifest")?;

        // Add classes from classes directory
        if let Some(ref classes_dir) = self.classes_dir {
            if classes_dir.exists() {
                self.add_directory_contents(&mut zip, classes_dir, "", options)?;
            }
        }

        // Add resources
        for (source, target) in &self.resources {
            if source.exists() {
                let target_str = target.to_string_lossy().replace('\\', "/");
                zip.start_file(&target_str, options)
                    .with_context(|| format!("Failed to start file: {}", target_str))?;
                
                let content = std::fs::read(source)
                    .with_context(|| format!("Failed to read resource: {:?}", source))?;
                zip.write_all(&content)
                    .with_context(|| format!("Failed to write resource: {}", target_str))?;
            }
        }

        zip.finish()
            .context("Failed to finish JAR file")?;

        tracing::info!("JAR file created successfully: {:?}", output_path);
        Ok(())
    }

    /// Add all contents of a directory to the JAR
    fn add_directory_contents(
        &self,
        zip: &mut ZipWriter<File>,
        dir: &Path,
        prefix: &str,
        options: FileOptions,
    ) -> Result<()> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(dir)
                    .context("Failed to get relative path")?;
                
                let zip_path = if prefix.is_empty() {
                    relative_path.to_string_lossy().replace('\\', "/")
                } else {
                    format!("{}/{}", prefix, relative_path.to_string_lossy().replace('\\', "/"))
                };

                zip.start_file(&zip_path, options)
                    .with_context(|| format!("Failed to start file in JAR: {}", zip_path))?;
                
                let content = std::fs::read(path)
                    .with_context(|| format!("Failed to read file: {:?}", path))?;
                zip.write_all(&content)
                    .with_context(|| format!("Failed to write file to JAR: {}", zip_path))?;
            }
        }

        Ok(())
    }
}

impl Default for JarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

