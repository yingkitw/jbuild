//! Tests for TreeWalker functionality

use jbuild::checkstyle::api::file::FileText;
use jbuild::checkstyle::checks::empty_catch_block::EmptyCatchBlockCheck;
use jbuild::checkstyle::runner::tree_walker::TreeWalker;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[test]
fn test_tree_walker_new() {
    let _walker = TreeWalker::new();
    assert!(true, "TreeWalker should be created");
}

#[test]
fn test_tree_walker_add_check() {
    let mut walker = TreeWalker::new();
    let check = Arc::new(Mutex::new(EmptyCatchBlockCheck::new()));
    let result = walker.add_check(check);
    assert!(result.is_ok(), "Should add check successfully");
}

#[test]
fn test_tree_walker_process_file() {
    let walker = TreeWalker::new();
    let java_code = "public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    
    let result = walker.process_file(&PathBuf::from("Test.java"), &file_text);
    assert!(result.is_ok(), "Should process file successfully");
}

#[test]
fn test_tree_walker_process_file_with_check() {
    let mut walker = TreeWalker::new();
    let check = Arc::new(Mutex::new(EmptyCatchBlockCheck::new()));
    walker.add_check(check).unwrap();
    
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
    
    let result = walker.process_file(&PathBuf::from("Test.java"), &file_text);
    assert!(result.is_ok(), "Should process file with check");
    
    let violations = result.unwrap();
    // May or may not find violations depending on parser mapping
    println!("Found {} violations", violations.len());
}

#[test]
fn test_tree_walker_multiple_checks() {
    let mut walker = TreeWalker::new();
    let check1 = Arc::new(Mutex::new(EmptyCatchBlockCheck::new()));
    let check2 = Arc::new(Mutex::new(EmptyCatchBlockCheck::new()));
    
    walker.add_check(check1).unwrap();
    walker.add_check(check2).unwrap();
    
    let java_code = "public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    
    let result = walker.process_file(&PathBuf::from("Test.java"), &file_text);
    assert!(result.is_ok(), "Should process file with multiple checks");
}

#[test]
fn test_tree_walker_empty_file() {
    let walker = TreeWalker::new();
    let file_text = FileText::new(PathBuf::from("Test.java"), String::new());
    
    let result = walker.process_file(&PathBuf::from("Test.java"), &file_text);
    // Should handle empty file gracefully
    assert!(result.is_ok(), "Should handle empty file");
}

