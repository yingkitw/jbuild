//! Annotation processing support for Java compilation
//!
//! Implements JSR 269 annotation processing integration

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};
use std::collections::HashMap;

/// Annotation processor configuration
#[derive(Debug, Clone)]
pub struct AnnotationProcessorConfig {
    /// Processor classpath
    pub processor_path: Vec<PathBuf>,
    /// Processor class names
    pub processors: Vec<String>,
    /// Generated source output directory
    pub generated_source_dir: PathBuf,
    /// Generated class output directory
    pub generated_class_dir: PathBuf,
    /// Processor options
    pub options: HashMap<String, String>,
    /// Whether to process only annotations
    pub proc_only: bool,
}

impl AnnotationProcessorConfig {
    pub fn new(generated_source_dir: PathBuf, generated_class_dir: PathBuf) -> Self {
        Self {
            processor_path: Vec::new(),
            processors: Vec::new(),
            generated_source_dir,
            generated_class_dir,
            options: HashMap::new(),
            proc_only: false,
        }
    }

    pub fn add_processor(&mut self, processor_class: String) {
        self.processors.push(processor_class);
    }

    pub fn add_processor_path(&mut self, path: PathBuf) {
        self.processor_path.push(path);
    }

    pub fn add_option(&mut self, key: String, value: String) {
        self.options.insert(key, value);
    }
}

/// Annotation processor executor
pub struct AnnotationProcessor {
    config: AnnotationProcessorConfig,
}

impl AnnotationProcessor {
    pub fn new(config: AnnotationProcessorConfig) -> Self {
        Self { config }
    }

    /// Run annotation processing
    pub fn process(
        &self,
        java_home: &Path,
        source_files: &[PathBuf],
        classpath: &[PathBuf],
        source_version: &str,
        target_version: &str,
    ) -> Result<AnnotationProcessingResult> {
        // Ensure output directories exist
        std::fs::create_dir_all(&self.config.generated_source_dir)?;
        std::fs::create_dir_all(&self.config.generated_class_dir)?;

        let javac = if cfg!(windows) {
            java_home.join("bin/javac.exe")
        } else {
            java_home.join("bin/javac")
        };

        let mut cmd = Command::new(&javac);

        // Source and target versions
        cmd.arg("-source").arg(source_version);
        cmd.arg("-target").arg(target_version);

        // Classpath
        if !classpath.is_empty() {
            let cp = classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.arg("-classpath").arg(cp);
        }

        // Processor path
        if !self.config.processor_path.is_empty() {
            let proc_path = self.config.processor_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.arg("-processorpath").arg(proc_path);
        }

        // Processors
        if !self.config.processors.is_empty() {
            cmd.arg("-processor").arg(self.config.processors.join(","));
        }

        // Generated source directory
        cmd.arg("-s").arg(&self.config.generated_source_dir);

        // Output directory for generated classes
        cmd.arg("-d").arg(&self.config.generated_class_dir);

        // Processor options
        for (key, value) in &self.config.options {
            cmd.arg(format!("-A{}={}", key, value));
        }

        // Process only (don't compile)
        if self.config.proc_only {
            cmd.arg("-proc:only");
        }

        // Verbose output for debugging
        cmd.arg("-verbose");

        // Source files
        for source in source_files {
            cmd.arg(source);
        }

        // Execute
        let output = cmd.output()
            .context("Failed to execute annotation processor")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Collect generated files
        let generated_sources = self.collect_generated_files(&self.config.generated_source_dir)?;
        let generated_classes = self.collect_generated_files(&self.config.generated_class_dir)?;

        Ok(AnnotationProcessingResult {
            success: output.status.success(),
            generated_sources,
            generated_classes,
            stdout,
            stderr,
        })
    }

    fn collect_generated_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if dir.exists() {
            for entry in walkdir::WalkDir::new(dir) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        Ok(files)
    }
}

/// Result of annotation processing
#[derive(Debug, Clone)]
pub struct AnnotationProcessingResult {
    pub success: bool,
    pub generated_sources: Vec<PathBuf>,
    pub generated_classes: Vec<PathBuf>,
    pub stdout: String,
    pub stderr: String,
}

/// Common annotation processors
pub struct CommonProcessors;

impl CommonProcessors {
    /// Lombok annotation processor
    pub fn lombok() -> String {
        "lombok.launch.AnnotationProcessorHider$AnnotationProcessor".to_string()
    }

    /// MapStruct annotation processor
    pub fn mapstruct() -> String {
        "org.mapstruct.ap.MappingProcessor".to_string()
    }

    /// Dagger annotation processor
    pub fn dagger() -> String {
        "dagger.internal.codegen.ComponentProcessor".to_string()
    }

    /// AutoValue annotation processor
    pub fn autovalue() -> String {
        "com.google.auto.value.processor.AutoValueProcessor".to_string()
    }

    /// Immutables annotation processor
    pub fn immutables() -> String {
        "org.immutables.processor.ProxyProcessor".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = AnnotationProcessorConfig::new(
            PathBuf::from("target/generated-sources"),
            PathBuf::from("target/generated-classes"),
        );
        assert_eq!(config.processors.len(), 0);
        assert_eq!(config.processor_path.len(), 0);
    }

    #[test]
    fn test_add_processor() {
        let mut config = AnnotationProcessorConfig::new(
            PathBuf::from("target/generated-sources"),
            PathBuf::from("target/generated-classes"),
        );
        config.add_processor(CommonProcessors::lombok());
        assert_eq!(config.processors.len(), 1);
    }

    #[test]
    fn test_common_processors() {
        assert!(CommonProcessors::lombok().contains("lombok"));
        assert!(CommonProcessors::mapstruct().contains("mapstruct"));
        assert!(CommonProcessors::dagger().contains("dagger"));
    }
}
