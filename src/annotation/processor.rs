//! Annotation processor implementation

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Java annotation processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationProcessor {
    /// Processor name (e.g., "org.mapstruct.ap.MappingProcessor")
    pub name: String,
    /// Supported annotation types
    pub supported_annotations: Vec<String>,
    /// Supported options
    pub supported_options: Vec<String>,
    /// Processor class path
    pub class_path: Vec<PathBuf>,
}

impl AnnotationProcessor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            supported_annotations: Vec::new(),
            supported_options: Vec::new(),
            class_path: Vec::new(),
        }
    }

    pub fn with_annotations(mut self, annotations: Vec<String>) -> Self {
        self.supported_annotations = annotations;
        self
    }

    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.supported_options = options;
        self
    }

    pub fn with_class_path(mut self, class_path: Vec<PathBuf>) -> Self {
        self.class_path = class_path;
        self
    }
}

/// Annotation processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationProcessorConfig {
    /// Processors to run
    pub processors: Vec<AnnotationProcessor>,
    /// Processor options
    pub options: Vec<(String, String)>,
    /// Generated sources directory
    pub generated_sources_directory: PathBuf,
    /// Generated classes directory
    pub generated_classes_directory: PathBuf,
}

impl Default for AnnotationProcessorConfig {
    fn default() -> Self {
        Self {
            processors: Vec::new(),
            options: Vec::new(),
            generated_sources_directory: PathBuf::from("target/generated-sources/annotations"),
            generated_classes_directory: PathBuf::from("target/generated-sources/annotations"),
        }
    }
}

impl AnnotationProcessorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_processor(&mut self, processor: AnnotationProcessor) {
        self.processors.push(processor);
    }

    pub fn add_option(&mut self, key: String, value: String) {
        self.options.push((key, value));
    }

    /// Get all class paths for processors
    pub fn get_processor_class_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for processor in &self.processors {
            paths.extend(processor.class_path.clone());
        }
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Common annotation processors
pub mod common {
    use super::*;

    /// MapStruct processor
    pub fn mapstruct_processor() -> AnnotationProcessor {
        AnnotationProcessor::new("org.mapstruct.ap.MappingProcessor")
            .with_annotations(vec![
                "org.mapstruct.Mapping".to_string(),
                "org.mapstruct.Mappings".to_string(),
                "org.mapstruct.Mapper".to_string(),
            ])
    }

    /// AutoValue processor
    pub fn autovalue_processor() -> AnnotationProcessor {
        AnnotationProcessor::new("com.google.auto.value.processor.AutoValueProcessor")
            .with_annotations(vec![
                "com.google.auto.value.AutoValue".to_string(),
                "com.google.auto.value.AutoValue.Builder".to_string(),
            ])
    }

    /// Lombok processor
    pub fn lombok_processor() -> AnnotationProcessor {
        AnnotationProcessor::new("lombok.launch.AnnotationProcessorHider$AnnotationProcessor")
            .with_annotations(vec![
                "lombok.Data".to_string(),
                "lombok.Value".to_string(),
                "lombok.Builder".to_string(),
                "lombok.AllArgsConstructor".to_string(),
                "lombok.NoArgsConstructor".to_string(),
            ])
    }

    /// Dagger processor
    pub fn dagger_processor() -> AnnotationProcessor {
        AnnotationProcessor::new("dagger.internal.codegen.ComponentProcessor")
            .with_annotations(vec![
                "dagger.Component".to_string(),
                "dagger.Module".to_string(),
                "dagger.Provides".to_string(),
                "dagger.Binds".to_string(),
            ])
    }

    /// Get all common processors
    pub fn common_processors() -> Vec<AnnotationProcessor> {
        vec![
            mapstruct_processor(),
            autovalue_processor(),
            lombok_processor(),
            dagger_processor(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_processor_creation() {
        let processor = AnnotationProcessor::new("test.Processor")
            .with_annotations(vec!["com.example.Test".to_string()]);

        assert_eq!(processor.name, "test.Processor");
        assert_eq!(processor.supported_annotations.len(), 1);
    }

    #[test]
    fn test_common_processors() {
        let processors = common::common_processors();
        assert!(!processors.is_empty());
    }
}
