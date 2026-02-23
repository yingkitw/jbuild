//! Tests for annotation processor integration

use jbuild::compiler::annotation_processor::AnnotationProcessorConfig;
use std::path::PathBuf;

#[test]
fn test_annotation_processor_config_creation() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let config = AnnotationProcessorConfig::new(gen_src.clone(), gen_class.clone());
    
    assert_eq!(config.generated_source_dir, gen_src);
    assert_eq!(config.generated_class_dir, gen_class);
    assert!(config.processors.is_empty());
    assert!(config.processor_path.is_empty());
}

#[test]
fn test_annotation_processor_config_add_processor() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let mut config = AnnotationProcessorConfig::new(gen_src, gen_class);
    config.add_processor("org.mapstruct.ap.MappingProcessor".to_string());
    
    assert_eq!(config.processors.len(), 1);
    assert_eq!(config.processors[0], "org.mapstruct.ap.MappingProcessor");
}

#[test]
fn test_annotation_processor_config_add_option() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let mut config = AnnotationProcessorConfig::new(gen_src, gen_class);
    config.add_option("mapstruct.defaultComponentModel".to_string(), "spring".to_string());
    
    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options.get("mapstruct.defaultComponentModel"), Some(&"spring".to_string()));
}

#[test]
fn test_annotation_processor_config_proc_only() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let mut config = AnnotationProcessorConfig::new(gen_src, gen_class);
    config.proc_only = true;
    
    assert_eq!(config.proc_only, true);
}

#[test]
fn test_annotation_processor_config_multiple_processors() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let mut config = AnnotationProcessorConfig::new(gen_src, gen_class);
    
    config.add_processor("lombok.launch.AnnotationProcessorHider$AnnotationProcessor".to_string());
    config.add_processor("org.mapstruct.ap.MappingProcessor".to_string());
    
    assert_eq!(config.processors.len(), 2);
}

#[test]
fn test_annotation_processor_config_multiple_options() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let mut config = AnnotationProcessorConfig::new(gen_src, gen_class);
    
    config.add_option("mapstruct.defaultComponentModel".to_string(), "spring".to_string());
    config.add_option("mapstruct.unmappedTargetPolicy".to_string(), "IGNORE".to_string());
    
    assert_eq!(config.options.len(), 2);
}

#[test]
fn test_annotation_processor_config_processor_path() {
    let gen_src = PathBuf::from("target/generated-sources");
    let gen_class = PathBuf::from("target/generated-classes");
    
    let config = AnnotationProcessorConfig::new(gen_src, gen_class);
    
    // Processor path starts empty
    assert!(config.processor_path.is_empty());
}
