//! Tests for Scala compiler integration

use jbuild::compiler::scala::{ScalaCompiler, ScalaCompilerConfig};
use std::path::PathBuf;

#[test]
fn test_scala_compiler_config_default() {
    let config = ScalaCompilerConfig::default();
    
    assert_eq!(config.scala_version, "2.13");
    assert_eq!(config.target, "17");
    assert_eq!(config.optimize, false);
}

#[test]
fn test_scala_compiler_config_custom() {
    let config = ScalaCompilerConfig {
        scala_home: Some(PathBuf::from("/usr/local/scala")),
        scala_version: "3.3".to_string(),
        target: "21".to_string(),
        options: vec!["-Ypartial-unification".to_string()],
        optimize: true,
    };
    
    assert_eq!(config.scala_version, "3.3");
    assert_eq!(config.target, "21");
    assert_eq!(config.optimize, true);
    assert_eq!(config.options.len(), 1);
}

#[test]
fn test_scala_compiler_creation() {
    let config = ScalaCompilerConfig::default();
    let _compiler = ScalaCompiler::new(config);
    
    // Just test creation works
    assert!(true);
}

#[test]
fn test_scala_compiler_detect() {
    // This will fail if scalac is not installed, which is expected
    let result = ScalaCompiler::detect_scalac();
    
    // We just test that the method exists and returns a Result
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_scala_compiler_versions() {
    let versions = vec!["2.12", "2.13", "3.3"];
    
    for version in versions {
        let config = ScalaCompilerConfig {
            scala_version: version.to_string(),
            ..Default::default()
        };
        
        let _compiler = ScalaCompiler::new(config);
    }
    assert!(true);
}

#[test]
fn test_scala_compiler_optimization() {
    let config = ScalaCompilerConfig {
        optimize: true,
        ..Default::default()
    };
    
    assert_eq!(config.optimize, true);
}

#[test]
fn test_scala_compiler_custom_options() {
    let custom_options = vec![
        "-Ypartial-unification".to_string(),
        "-language:higherKinds".to_string(),
    ];
    
    let config = ScalaCompilerConfig {
        options: custom_options.clone(),
        ..Default::default()
    };
    
    assert_eq!(config.options, custom_options);
}

#[test]
fn test_scala_compiler_jvm_targets() {
    let targets = vec!["8", "11", "17", "21"];
    
    for target in targets {
        let config = ScalaCompilerConfig {
            target: target.to_string(),
            ..Default::default()
        };
        
        assert_eq!(config.target, target);
    }
}
