//! Tests for TreeWalkerFileSetCheck

use jbuild::checkstyle::api::check::FileSetCheck;
use jbuild::checkstyle::api::config::{Configurable, Context, Contextualizable};
use jbuild::checkstyle::api::file::FileText;
use jbuild::checkstyle::runner::tree_walker_file_set_check::TreeWalkerFileSetCheck;
use std::path::PathBuf;

#[test]
fn test_tree_walker_file_set_check_new() {
    let check = TreeWalkerFileSetCheck::new();
    assert!(true, "TreeWalkerFileSetCheck should be created");
}

#[test]
fn test_tree_walker_file_set_check_init() {
    let mut check = TreeWalkerFileSetCheck::new();
    let result = check.init();
    assert!(result.is_ok(), "init should succeed");
}

#[test]
fn test_tree_walker_file_set_check_contextualize() {
    let mut check = TreeWalkerFileSetCheck::new();
    let context = Context::new();
    let result = check.contextualize(&context);
    assert!(result.is_ok(), "contextualize should succeed");
}

#[test]
fn test_tree_walker_file_set_check_configure() {
    let mut check = TreeWalkerFileSetCheck::new();
    let config = jbuild::checkstyle::api::config::Configuration::new("TreeWalker".to_string());
    let result = check.configure(&config);
    assert!(result.is_ok(), "configure should succeed");
}

#[test]
fn test_tree_walker_file_set_check_process() {
    let mut check = TreeWalkerFileSetCheck::new();
    let context = Context::new();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    check.begin_processing("UTF-8").unwrap();
    
    let java_code = "public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    
    let result = check.process(&PathBuf::from("Test.java"), &file_text);
    assert!(result.is_ok(), "process should succeed");
}

#[test]
fn test_tree_walker_file_set_check_with_child_check() {
    let mut check = TreeWalkerFileSetCheck::new();
    let context = Context::new();
    check.contextualize(&context).unwrap();
    
    // Add a child check configuration
    let mut config = jbuild::checkstyle::api::config::Configuration::new("TreeWalker".to_string());
    let child_config = jbuild::checkstyle::api::config::Configuration::new("EmptyCatchBlock".to_string());
    config.add_child(child_config);
    
    check.configure(&config).unwrap();
    check.init().unwrap();
    check.begin_processing("UTF-8").unwrap();
    
    let java_code = r#"
public class Test {
    public void test() {
        try {
            doSomething();
        } catch (Exception e) {
        }
    }
}
"#;
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    
    let result = check.process(&PathBuf::from("Test.java"), &file_text);
    assert!(result.is_ok(), "process with child check should succeed");
}

#[test]
fn test_tree_walker_file_set_check_finish_processing() {
    let mut check = TreeWalkerFileSetCheck::new();
    check.init().unwrap();
    check.begin_processing("UTF-8").unwrap();
    
    let result = check.finish_processing();
    assert!(result.is_ok(), "finish_processing should succeed");
}

#[test]
fn test_tree_walker_file_set_check_destroy() {
    let mut check = TreeWalkerFileSetCheck::new();
    check.init().unwrap();
    
    let result = check.destroy();
    assert!(result.is_ok(), "destroy should succeed");
}

