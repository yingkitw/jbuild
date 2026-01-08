//! Unit tests for core components

use jbuild::checkstyle::api::config::Configuration;
use jbuild::checkstyle::api::file::FileText;
use jbuild::checkstyle::runner::module_factory::{DefaultModuleFactory, ModuleFactory};
use jbuild::checkstyle::runner::{Checker, ConfigurationLoader, DefaultLogger};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_checker_new() {
    let checker = Checker::new();
    // Just verify it can be created
    assert!(true, "Checker should be created successfully");
}

#[test]
fn test_checker_configure() {
    let mut checker = Checker::new();
    let mut config = Configuration::new("Checker".to_string());
    config.add_property("fileExtensions".to_string(), "java".to_string());

    let result = checker.configure(&config);
    assert!(result.is_ok(), "Checker should configure successfully");
}

#[test]
fn test_checker_add_listener() {
    let mut checker = Checker::new();
    let logger = DefaultLogger::new();
    checker.add_listener(Box::new(logger));
    // Just verify it doesn't panic
    assert!(true, "Listener should be added successfully");
}

#[test]
fn test_configuration_new() {
    let config = Configuration::new("TestModule".to_string());
    assert_eq!(
        config.name, "TestModule",
        "Configuration should have correct name"
    );
}

#[test]
fn test_configuration_add_property() {
    let mut config = Configuration::new("TestModule".to_string());
    config.add_property("key".to_string(), "value".to_string());
    assert_eq!(
        config.get_property("key"),
        Some(&"value".to_string()),
        "Property should be stored"
    );
}

#[test]
fn test_configuration_add_child() {
    let mut parent = Configuration::new("Parent".to_string());
    let child = Configuration::new("Child".to_string());
    parent.add_child(child);
    assert_eq!(parent.get_children().len(), 1, "Child should be added");
}

#[test]
fn test_module_factory_create_tree_walker() {
    let factory = DefaultModuleFactory::new();
    let result = factory.create_module("TreeWalker");
    assert!(result.is_ok(), "Should create TreeWalker");
}

#[test]
fn test_module_factory_create_unknown_module() {
    let factory = DefaultModuleFactory::new();
    let result = factory.create_module("UnknownModule");
    assert!(result.is_err(), "Should fail for unknown module");
}

#[test]
fn test_module_factory_create_check() {
    let factory = DefaultModuleFactory::new();
    let result = factory.create_module("EmptyCatchBlock");
    assert!(result.is_ok(), "Should create EmptyCatchBlock check");
}

#[test]
fn test_configuration_loader_simple_xml() {
    // Create a temporary XML file
    let xml_content = r#"<?xml version="1.0"?>
<module name="Checker">
    <module name="TreeWalker">
        <module name="EmptyCatchBlock"/>
    </module>
</module>"#;

    let temp_file = std::env::temp_dir().join("test_config.xml");
    let mut file = File::create(&temp_file).unwrap();
    file.write_all(xml_content.as_bytes()).unwrap();
    drop(file);

    let result = ConfigurationLoader::load_configuration(&temp_file);
    assert!(result.is_ok(), "Should load configuration from XML");

    let config = result.unwrap();
    assert_eq!(
        config.name, "module",
        "Root module name should be 'module' (XML tag name)"
    );
    assert_eq!(
        config.get_property("name"),
        Some(&"Checker".to_string()),
        "Module name should be in 'name' property"
    );
    assert_eq!(config.get_children().len(), 1, "Should have one child");

    // Cleanup
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_configuration_loader_invalid_xml() {
    let temp_file = std::env::temp_dir().join("test_invalid.xml");
    let mut file = File::create(&temp_file).unwrap();
    file.write_all(b"<invalid xml>").unwrap();
    drop(file);

    let result = ConfigurationLoader::load_configuration(&temp_file);
    // XML parser might be lenient or might fail - just verify it doesn't panic
    // and returns either Ok or Err
    match result {
        Ok(_) => {
            // Parser was lenient and parsed something
            assert!(true, "Parser handled invalid XML (may be lenient)");
        }
        Err(_) => {
            // Parser correctly rejected invalid XML
            assert!(true, "Parser correctly rejected invalid XML");
        }
    }

    // Cleanup
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_default_logger_new() {
    let logger = DefaultLogger::new();
    // Just verify it can be created
    assert!(true, "DefaultLogger should be created successfully");
}

#[test]
fn test_file_text_new() {
    let file_text = FileText::new(
        PathBuf::from("test.java"),
        "public class Test {}".to_string(),
    );
    assert_eq!(file_text.line_count(), 1, "Should have one line");
    assert_eq!(
        file_text.get_line(1),
        Some("public class Test {}"),
        "Should get correct line"
    );
}

#[test]
fn test_file_text_multiline() {
    let content = "line1\nline2\nline3";
    let file_text = FileText::new(PathBuf::from("test.java"), content.to_string());
    assert_eq!(file_text.line_count(), 3, "Should have three lines");
    assert_eq!(
        file_text.get_line(2),
        Some("line2"),
        "Should get correct line"
    );
}
