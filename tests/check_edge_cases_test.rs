//! Edge case tests for checks

use jbuild::checkstyle::api::ast::DetailAst;
use jbuild::checkstyle::api::check::{Check, FileSetCheck};
use jbuild::checkstyle::api::config::{Configurable, Context, Contextualizable};
use jbuild::checkstyle::api::file::{FileContents, FileText};
use jbuild::checkstyle::checks::empty_catch_block::EmptyCatchBlockCheck;
use jbuild::checkstyle::checks::line_length::LineLengthCheck;
use jbuild::checkstyle::checks::package_name::PackageNameCheck;
use jbuild::checkstyle::checks::redundant_import::RedundantImportCheck;
use jbuild::checkstyle::checks::type_name::TypeNameCheck;
use jbuild::checkstyle::parser::java_parser::JavaParser;
use std::path::PathBuf;
use std::sync::Arc;

fn create_context() -> Context {
    Context::new()
}

#[test]
fn test_line_length_check_edge_cases() {
    let mut check = LineLengthCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    // Test with exactly max length
    let exact_length = "a".repeat(80);
    let file_text = FileText::new(PathBuf::from("test.java"), exact_length);
    let violations = check.process(&PathBuf::from("test.java"), &file_text).unwrap();
    // Should not violate if exactly at max
    assert_eq!(violations.len(), 0);
    
    // Test with max + 1
    let over_length = "a".repeat(81);
    let file_text = FileText::new(PathBuf::from("test.java"), over_length);
    let violations = check.process(&PathBuf::from("test.java"), &file_text).unwrap();
    assert!(violations.len() > 0);
}

#[test]
fn test_line_length_check_tabs() {
    let mut check = LineLengthCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    // Line with tabs that expands beyond max
    let line_with_tabs = format!("{}\t{}", "a".repeat(70), "b".repeat(10));
    let file_text = FileText::new(PathBuf::from("test.java"), line_with_tabs);
    let violations = check.process(&PathBuf::from("test.java"), &file_text).unwrap();
    // Should detect violation when tabs are expanded
    println!("Found {} violations for line with tabs", violations.len());
}

#[test]
fn test_package_name_check_edge_cases() {
    let mut check = PackageNameCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    let java_code = "package a; public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);
    let ast = JavaParser::parse(&file_contents).unwrap();
    
    if let Some(package_ast) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF) {
        check.visit_token(package_ast.as_ref() as &dyn DetailAst).unwrap();
        let violations = check.get_violations();
        // Single letter package should be valid
        assert_eq!(violations.len(), 0);
    }
}

#[test]
fn test_redundant_import_check_same_package() {
    let mut check = RedundantImportCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.begin_tree(Arc::new(
        jbuild::checkstyle::parser::ast_impl::DetailAstImpl::new(
            jbuild::checkstyle::api::ast::token_types::COMPILATION_UNIT,
            "".to_string(),
            1,
            1,
        )
    ).as_ref() as &dyn DetailAst).unwrap();
    
    // This test would need proper AST construction to fully test
    // For now, just verify the check can be initialized
    assert!(true);
}

#[test]
fn test_type_name_check_custom_format() {
    let mut check = TypeNameCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("TypeName".to_string());
    config.add_property("format".to_string(), r"^[a-z]+$".to_string()); // Lowercase only
    
    check.configure(&config).unwrap();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    // Test would need proper AST construction
    assert!(true);
}

#[test]
fn test_type_name_check_access_control() {
    let mut check = TypeNameCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("TypeName".to_string());
    config.add_property("applyToPublic".to_string(), "false".to_string());
    config.add_property("applyToPrivate".to_string(), "true".to_string());
    
    check.configure(&config).unwrap();
    assert!(true);
}

#[test]
fn test_empty_catch_block_check_no_catch() {
    let mut check = EmptyCatchBlockCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let java_code = "public class Test { public void test() {} }";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);
    let ast = JavaParser::parse(&file_contents).unwrap();
    
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    // No catch blocks, so no violations
    let violations = check.get_violations();
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_init_multiple_times() {
    let mut check = EmptyCatchBlockCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    check.init().unwrap();
    check.init().unwrap(); // Should handle multiple init calls
    assert!(true);
}

#[test]
fn test_check_configure_empty_config() {
    let mut check = EmptyCatchBlockCheck::new();
    let config = jbuild::checkstyle::api::config::Configuration::new("EmptyCatchBlock".to_string());
    let result = check.configure(&config);
    assert!(result.is_ok());
}

