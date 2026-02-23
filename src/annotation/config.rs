//! Annotation processing configuration

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for annotation processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationConfig {
    /// Enable annotation processing
    pub enabled: bool,
    /// Generated sources directory
    pub generated_sources_dir: PathBuf,
    /// Generated classes directory
    pub generated_classes_dir: PathBuf,
    /// Processor options
    pub options: Vec<(String, String)>,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            generated_sources_dir: PathBuf::from("target/generated-sources/annotations"),
            generated_classes_dir: PathBuf::from("target/generated-sources/annotations"),
            options: Vec::new(),
        }
    }
}

impl AnnotationConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_generated_sources_dir(mut self, dir: PathBuf) -> Self {
        self.generated_sources_dir = dir;
        self
    }

    pub fn with_generated_classes_dir(mut self, dir: PathBuf) -> Self {
        self.generated_classes_dir = dir;
        self
    }

    pub fn add_option(mut self, key: String, value: String) -> Self {
        self.options.push((key, value));
        self
    }

    /// Get options as command-line arguments
    pub fn get_options_as_args(&self) -> Vec<String> {
        self.options
            .iter()
            .flat_map(|(k, v)| vec![format!("-A{}={}", k, v)])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_config() {
        let config = AnnotationConfig::new()
            .add_option("mapstruct.defaultComponentModel".to_string(), "default".to_string());

        assert_eq!(config.options.len(), 1);
        assert_eq!(config.get_options_as_args().len(), 1);
    }
}
