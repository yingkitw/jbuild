//! Tests for error handling and edge cases

use jbuild::checkstyle::api::config::Configuration;
use jbuild::checkstyle::api::error::CheckstyleError;
use jbuild::checkstyle::runner::{ConfigurationLoader, PropertiesLoader};
use std::fs::File;
use std::io::Write;

#[test]
fn test_configuration_loader_nonexistent_file() {
    let result = ConfigurationLoader::load_configuration("nonexistent_file.xml");
    assert!(result.is_err(), "Should fail for nonexistent file");

    if let Err(CheckstyleError::Configuration(msg)) = result {
        assert!(
            msg.contains("Could not open"),
            "Error message should mention file opening"
        );
    } else {
        panic!("Should return Configuration error");
    }
}

#[test]
fn test_configuration_loader_empty_file() {
    let temp_file = std::env::temp_dir().join("test_empty.xml");
    File::create(&temp_file).unwrap();

    let result = ConfigurationLoader::load_configuration(&temp_file);
    assert!(result.is_err(), "Should fail for empty file");

    if let Err(CheckstyleError::Configuration(msg)) = result {
        assert!(
            msg.contains("Empty") || msg.contains("empty"),
            "Error should mention empty file"
        );
    }

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_properties_loader_nonexistent_file() {
    let result = PropertiesLoader::load_properties("nonexistent.properties");
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_properties_loader_simple() {
    let temp_file = std::env::temp_dir().join("test.properties");
    let mut file = File::create(&temp_file).unwrap();
    writeln!(file, "key1=value1").unwrap();
    writeln!(file, "key2=value2").unwrap();
    drop(file);

    let result = PropertiesLoader::load_properties(&temp_file);
    assert!(result.is_ok(), "Should load properties successfully");

    let props = result.unwrap();
    assert_eq!(props.get("key1"), Some(&"value1".to_string()));
    assert_eq!(props.get("key2"), Some(&"value2".to_string()));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_properties_loader_with_comments() {
    let temp_file = std::env::temp_dir().join("test_comments.properties");
    let mut file = File::create(&temp_file).unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "key=value").unwrap();
    writeln!(file, "! Another comment").unwrap();
    drop(file);

    let result = PropertiesLoader::load_properties(&temp_file);
    assert!(result.is_ok(), "Should load properties with comments");

    let props = result.unwrap();
    assert_eq!(props.get("key"), Some(&"value".to_string()));
    assert_eq!(
        props.len(),
        1,
        "Should only have one property (comments ignored)"
    );

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_properties_loader_empty_lines() {
    let temp_file = std::env::temp_dir().join("test_empty_lines.properties");
    let mut file = File::create(&temp_file).unwrap();
    writeln!(file, "key1=value1").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "key2=value2").unwrap();
    drop(file);

    let result = PropertiesLoader::load_properties(&temp_file);
    assert!(result.is_ok(), "Should handle empty lines");

    let props = result.unwrap();
    assert_eq!(props.len(), 2, "Should have two properties");

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_configuration_get_property_nonexistent() {
    let config = Configuration::new("Test".to_string());
    assert_eq!(
        config.get_property("nonexistent"),
        None,
        "Should return None for nonexistent property"
    );
}

#[test]
fn test_configuration_multiple_properties() {
    let mut config = Configuration::new("Test".to_string());
    config.add_property("key1".to_string(), "value1".to_string());
    config.add_property("key2".to_string(), "value2".to_string());
    config.add_property("key3".to_string(), "value3".to_string());

    assert_eq!(config.get_property("key1"), Some(&"value1".to_string()));
    assert_eq!(config.get_property("key2"), Some(&"value2".to_string()));
    assert_eq!(config.get_property("key3"), Some(&"value3".to_string()));
}

#[test]
fn test_configuration_nested_children() {
    let mut root = Configuration::new("Root".to_string());
    let mut child1 = Configuration::new("Child1".to_string());
    let child2 = Configuration::new("Child2".to_string());

    child1.add_child(child2);
    root.add_child(child1);

    assert_eq!(root.get_children().len(), 1, "Root should have one child");
    assert_eq!(
        root.get_children()[0].get_children().len(),
        1,
        "Child1 should have one child"
    );
}
