use jbuild::config::{JbuildConfig, PackageSection};
use std::fs;
use tempfile::TempDir;

/// Test basic jbuild.toml parsing
#[test]
fn test_jbuild_config_basic_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "test-project"
version = "1.0.0"
description = "A test project"
java = "17"

[dependencies]
"junit:junit" = "4.13.2"
"com.google.guava:guava" = "31.1-jre"

[dev-dependencies]
"org.mockito:mockito-core" = "5.1.1"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.package.name, "test-project");
    assert_eq!(config.package.version, "1.0.0");
    assert_eq!(config.package.description.unwrap(), "A test project");
    assert_eq!(config.package.java.unwrap(), "17");

    assert_eq!(config.dependencies.len(), 2);
    assert_eq!(config.dependencies["junit:junit"], "4.13.2");
    assert_eq!(config.dependencies["com.google.guava:guava"], "31.1-jre");

    assert_eq!(config.dev_dependencies.len(), 1);
    assert_eq!(config.dev_dependencies["org.mockito:mockito-core"], "5.1.1");
}

/// Test jbuild.toml with minimal configuration
#[test]
fn test_jbuild_config_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "minimal"
version = "0.1.0"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.package.name, "minimal");
    assert_eq!(config.package.version, "0.1.0");
    assert!(config.package.description.is_none());
    assert!(config.package.java.is_none());
    assert!(config.dependencies.is_empty());
    assert!(config.dev_dependencies.is_empty());
}

/// Test jbuild.toml file operations
#[test]
fn test_jbuild_config_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "file-test"
version = "2.0.0"

[dependencies]
"test:dep" = "1.0"
"#;

    // Write to file
    fs::write(&config_path, toml_content).unwrap();

    // Load from file
    let config = JbuildConfig::from_file(&config_path).unwrap();
    assert_eq!(config.package.name, "file-test");
    assert_eq!(config.package.version, "2.0.0");
    assert_eq!(config.dependencies["test:dep"], "1.0");
}

/// Test PackageSection creation and validation
#[test]
fn test_package_section_creation_and_validation() {
    let package = PackageSection {
        name: "test-package".to_string(),
        version: "1.2.3".to_string(),
        description: Some("Test package".to_string()),
        java: Some("11".to_string()),
    };

    assert_eq!(package.name, "test-package");
    assert_eq!(package.version, "1.2.3");
    assert_eq!(package.description.unwrap(), "Test package");
    assert_eq!(package.java.unwrap(), "11");
}

/// Test JbuildConfig dependency management
#[test]
fn test_jbuild_config_dependency_management() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "dep-test"
version = "1.0.0"

[dependencies]
"group:artifact" = "1.0.0"
"another:dep" = "2.0.0"

[dev-dependencies]
"test:junit" = "4.13.2"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.dependencies.len(), 2);
    assert_eq!(config.dev_dependencies.len(), 1);
}

/// Test jbuild.toml with complex dependency specifications
#[test]
fn test_jbuild_config_complex_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "complex-deps"
version = "1.0.0"

[dependencies]
"org.springframework:spring-core" = "5.3.23"
"com.fasterxml.jackson.core:jackson-databind" = "2.13.4.2"
"org.apache.commons:commons-lang3" = "3.12.0"
"ch.qos.logback:logback-classic" = "1.2.11"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter" = "5.9.1"
"org.mockito:mockito-core" = "4.11.0"
"org.assertj:assertj-core" = "3.24.2"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.dependencies.len(), 4);
    assert_eq!(config.dev_dependencies.len(), 3);

    // Verify specific versions
    assert_eq!(config.dependencies["org.springframework:spring-core"], "5.3.23");
    assert_eq!(config.dev_dependencies["org.junit.jupiter:junit-jupiter"], "5.9.1");
}

/// Test jbuild.toml POM XML generation
#[test]
fn test_jbuild_config_pom_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "pom-test"
version = "1.0.0"
description = "Test POM generation"
java = "17"

[dependencies]
"junit:junit" = "4.13.2"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();
    let pom_xml = config.to_pom_xml();

    // Verify POM contains expected elements
    assert!(pom_xml.contains("<artifactId>pom-test</artifactId>"));
    assert!(pom_xml.contains("<version>1.0.0</version>"));
    assert!(pom_xml.contains("Test POM generation"));
    assert!(pom_xml.contains("<maven.compiler.source>17</maven.compiler.source>"));
    assert!(pom_xml.contains("<artifactId>junit</artifactId>"));
    assert!(pom_xml.contains("<version>4.13.2</version>"));
}

/// Test jbuild.toml with Java version specifications
#[test]
fn test_jbuild_config_java_versions() {
    let test_cases = vec![
        "8", "11", "17", "21"
    ];

    for java_version in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("jbuild.toml");

        let toml_content = format!(r#"
[package]
name = "java-test"
version = "1.0.0"
java = "{java_version}"
"#);

        fs::write(&config_path, toml_content).unwrap();
        let config = JbuildConfig::from_file(&config_path).unwrap();
        let pom_xml = config.to_pom_xml();

        assert!(pom_xml.contains(&format!("<maven.compiler.source>{java_version}</maven.compiler.source>")));
        assert!(pom_xml.contains(&format!("<maven.compiler.target>{java_version}</maven.compiler.target>")));
    }
}

/// Test jbuild.toml error handling
#[test]
fn test_jbuild_config_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Test missing required fields
    let invalid_toml = r#"
[package]
description = "Missing name and version"
"#;
    let config_path = temp_dir.path().join("invalid.toml");
    fs::write(&config_path, invalid_toml).unwrap();

    let result = JbuildConfig::from_file(&config_path);
    assert!(result.is_err());

    // Test invalid TOML syntax
    let invalid_syntax = r#"
[package
name = "invalid"
version = "1.0.0"
"#;
    let config_path2 = temp_dir.path().join("invalid2.toml");
    fs::write(&config_path2, invalid_syntax).unwrap();

    let result = JbuildConfig::from_file(&config_path2);
    assert!(result.is_err());
}

/// Test jbuild.toml with empty sections
#[test]
fn test_jbuild_config_empty_sections() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "empty-test"
version = "1.0.0"

[dependencies]

[dev-dependencies]
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.package.name, "empty-test");
    assert_eq!(config.package.version, "1.0.0");
    assert!(config.dependencies.is_empty());
    assert!(config.dev_dependencies.is_empty());
}

/// Test PackageSection creation
#[test]
fn test_package_section_creation() {
    let package = PackageSection {
        name: "test-package".to_string(),
        version: "1.2.3".to_string(),
        description: Some("Test package".to_string()),
        java: Some("11".to_string()),
    };

    assert_eq!(package.name, "test-package");
    assert_eq!(package.version, "1.2.3");
    assert_eq!(package.description.unwrap(), "Test package");
    assert_eq!(package.java.unwrap(), "11");
}

/// Test PackageSection default values
#[test]
fn test_package_section_defaults() {
    let package = PackageSection {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        java: None,
    };

    assert_eq!(package.name, "test");
    assert_eq!(package.version, "1.0.0");
    assert!(package.description.is_none());
    assert!(package.java.is_none());
}

/// Test jbuild.toml parsing and field access
#[test]
fn test_jbuild_config_parsing_and_access() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "access-test"
version = "2.1.0"
description = "Test field access"
java = "17"

[dependencies]
"test:dep1" = "1.0.0"
"test:dep2" = "2.0.0"

[dev-dependencies]
"test:dev-dep" = "3.0.0"
"#;

    // Write and parse
    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    // Test field access
    assert_eq!(config.package.name, "access-test");
    assert_eq!(config.package.version, "2.1.0");
    assert_eq!(config.package.description.unwrap(), "Test field access");
    assert_eq!(config.package.java.unwrap(), "17");
    assert_eq!(config.dependencies["test:dep1"], "1.0.0");
    assert_eq!(config.dev_dependencies["test:dev-dep"], "3.0.0");
}

/// Test jbuild.toml with special characters in values
#[test]
fn test_jbuild_config_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "special-chars-test"
version = "1.0.0-beta.1"
description = "Test with special chars: @#$%^&*()"
java = "17"

[dependencies]
"com.example:special-artifact" = "1.0.0-SNAPSHOT"
"group:with:colons" = "2.0.0"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    assert_eq!(config.package.name, "special-chars-test");
    assert_eq!(config.package.version, "1.0.0-beta.1");
    assert_eq!(config.package.description.unwrap(), "Test with special chars: @#$%^&*()");
    assert_eq!(config.dependencies["com.example:special-artifact"], "1.0.0-SNAPSHOT");
    assert_eq!(config.dependencies["group:with:colons"], "2.0.0");
}

/// Test jbuild.toml dependency handling
#[test]
fn test_jbuild_config_dependency_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild.toml");

    let toml_content = r#"
[package]
name = "ordering-test"
version = "1.0.0"

[dependencies]
"zzz:last" = "1.0"
"aaa:first" = "1.0"
"mmm:middle" = "1.0"
"#;

    fs::write(&config_path, toml_content).unwrap();
    let config = JbuildConfig::from_file(&config_path).unwrap();

    // Check that all dependencies are present (BTreeMap may reorder)
    assert_eq!(config.dependencies.len(), 3);
    assert_eq!(config.dependencies["zzz:last"], "1.0");
    assert_eq!(config.dependencies["aaa:first"], "1.0");
    assert_eq!(config.dependencies["mmm:middle"], "1.0");
}