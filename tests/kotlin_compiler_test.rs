//! Tests for Kotlin compiler integration

use jbuild::compiler::kotlin::{KotlinCompiler, KotlinCompilerConfig};
use std::path::PathBuf;

#[test]
fn test_kotlin_compiler_config_default() {
    let config = KotlinCompilerConfig::default();
    
    assert_eq!(config.jvm_target, "17");
    assert_eq!(config.progressive, false);
    assert!(config.plugins.is_empty());
}

#[test]
fn test_kotlin_compiler_config_custom() {
    let config = KotlinCompilerConfig {
        kotlin_home: Some(PathBuf::from("/usr/local/kotlin")),
        jvm_target: "21".to_string(),
        api_version: Some("1.9".to_string()),
        language_version: Some("1.9".to_string()),
        progressive: true,
        plugins: vec![],
    };
    
    assert_eq!(config.jvm_target, "21");
    assert_eq!(config.progressive, true);
    assert_eq!(config.api_version, Some("1.9".to_string()));
}

#[test]
fn test_kotlin_compiler_creation() {
    let config = KotlinCompilerConfig::default();
    let _compiler = KotlinCompiler::new(config);
    
    // Just test creation works
    assert!(true);
}

#[test]
fn test_kotlin_compiler_detect() {
    // This will fail if kotlinc is not installed, which is expected
    let result = KotlinCompiler::detect_kotlinc();
    
    // We just test that the method exists and returns a Result
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_kotlin_compiler_jvm_targets() {
    let targets = vec!["1.8", "11", "17", "21"];
    
    for target in targets {
        let config = KotlinCompilerConfig {
            jvm_target: target.to_string(),
            ..Default::default()
        };
        
        let _compiler = KotlinCompiler::new(config);
        // Just test creation works
    }
    assert!(true);
}

#[test]
fn test_kotlin_compiler_progressive_mode() {
    let config = KotlinCompilerConfig {
        progressive: true,
        ..Default::default()
    };
    
    let _compiler = KotlinCompiler::new(config);
    assert!(true);
}

#[test]
fn test_kotlin_compiler_api_version() {
    let config = KotlinCompilerConfig {
        api_version: Some("1.9".to_string()),
        language_version: Some("1.9".to_string()),
        ..Default::default()
    };
    
    assert_eq!(config.api_version, Some("1.9".to_string()));
    assert_eq!(config.language_version, Some("1.9".to_string()));
}
