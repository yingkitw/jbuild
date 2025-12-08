//! Unit tests for individual checks

use jbuild::checkstyle::api::ast::DetailAst;
use jbuild::checkstyle::api::check::{Check, FileSetCheck};
use jbuild::checkstyle::api::config::{Configurable, Context, Contextualizable};
use jbuild::checkstyle::api::file::{FileContents, FileText};
use jbuild::checkstyle::checks::empty_catch_block::EmptyCatchBlockCheck;
use jbuild::checkstyle::checks::empty_statement::EmptyStatementCheck;
use jbuild::checkstyle::checks::line_length::LineLengthCheck;
use jbuild::checkstyle::checks::missing_switch_default::MissingSwitchDefaultCheck;
use jbuild::checkstyle::checks::multiple_variable_declarations::MultipleVariableDeclarationsCheck;
use jbuild::checkstyle::checks::package_name::PackageNameCheck;
use jbuild::checkstyle::checks::redundant_import::RedundantImportCheck;
use jbuild::checkstyle::checks::simplify_boolean_return::SimplifyBooleanReturnCheck;
use jbuild::checkstyle::checks::type_name::TypeNameCheck;
use jbuild::checkstyle::parser::java_parser::JavaParser;
use std::path::PathBuf;

fn create_context() -> Context {
    Context::new()
}

#[test]
fn test_empty_catch_block_check_required_tokens() {
    let check = EmptyCatchBlockCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::LITERAL_CATCH),
        "Should require LITERAL_CATCH token"
    );
}

#[test]
fn test_empty_statement_check_required_tokens() {
    let check = EmptyStatementCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::EMPTY_STAT),
        "Should require EMPTY_STAT token"
    );
}

#[test]
fn test_missing_switch_default_check_required_tokens() {
    let check = MissingSwitchDefaultCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::LITERAL_SWITCH),
        "Should require LITERAL_SWITCH token"
    );
}

#[test]
fn test_simplify_boolean_return_check_required_tokens() {
    let check = SimplifyBooleanReturnCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::LITERAL_IF),
        "Should require LITERAL_IF token"
    );
}

#[test]
fn test_multiple_variable_declarations_check_required_tokens() {
    let check = MultipleVariableDeclarationsCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::VARIABLE_DEF),
        "Should require VARIABLE_DEF token"
    );
}

#[test]
fn test_package_name_check_required_tokens() {
    let check = PackageNameCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF),
        "Should require PACKAGE_DEF token"
    );
}

#[test]
fn test_redundant_import_check_required_tokens() {
    let check = RedundantImportCheck::new();
    let tokens = check.get_required_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::IMPORT),
        "Should require IMPORT token"
    );
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::STATIC_IMPORT),
        "Should require STATIC_IMPORT token"
    );
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF),
        "Should require PACKAGE_DEF token"
    );
}

#[test]
fn test_type_name_check_acceptable_tokens() {
    let check = TypeNameCheck::new();
    let tokens = check.get_acceptable_tokens();
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::CLASS_DEF),
        "Should accept CLASS_DEF token"
    );
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::INTERFACE_DEF),
        "Should accept INTERFACE_DEF token"
    );
    assert!(
        tokens.contains(&jbuild::checkstyle::api::ast::token_types::ENUM_DEF),
        "Should accept ENUM_DEF token"
    );
}

#[test]
fn test_line_length_check_configuration() {
    let mut check = LineLengthCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("LineLength".to_string());
    config.add_property("max".to_string(), "120".to_string());

    let result = check.configure(&config);
    assert!(result.is_ok(), "Should configure LineLength check");
}

#[test]
fn test_line_length_check_process() {
    let mut check = LineLengthCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();

    let long_line = "a".repeat(100);
    let file_text = FileText::new(PathBuf::from("test.java"), long_line);

    let violations = check
        .process(&PathBuf::from("test.java"), &file_text)
        .unwrap();
    // Should find violation for line exceeding default max (80)
    assert!(violations.len() > 0, "Should find violations for long line");
}

#[test]
fn test_line_length_check_ignore_pattern() {
    let mut check = LineLengthCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("LineLength".to_string());
    config.add_property("ignorePattern".to_string(), r"^package .*".to_string());
    check.configure(&config).unwrap();

    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();

    let long_package_line = format!("package {};", "a".repeat(100));
    let file_text = FileText::new(PathBuf::from("test.java"), long_package_line);

    let violations = check
        .process(&PathBuf::from("test.java"), &file_text)
        .unwrap();
    // Should not find violations for ignored pattern
    assert_eq!(violations.len(), 0, "Should ignore lines matching pattern");
}

#[test]
fn test_package_name_check_configuration() {
    let mut check = PackageNameCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("PackageName".to_string());
    config.add_property(
        "format".to_string(),
        r"^[a-z]+(\.[a-z][a-z0-9]*)*$".to_string(),
    );

    let result = check.configure(&config);
    assert!(result.is_ok(), "Should configure PackageName check");
}

#[test]
fn test_type_name_check_configuration() {
    let mut check = TypeNameCheck::new();
    let mut config = jbuild::checkstyle::api::config::Configuration::new("TypeName".to_string());
    config.add_property("format".to_string(), r"^[A-Z][a-zA-Z0-9]*$".to_string());
    config.add_property("applyToPrivate".to_string(), "false".to_string());

    let result = check.configure(&config);
    assert!(result.is_ok(), "Should configure TypeName check");
}

#[test]
fn test_redundant_import_check_begin_tree() {
    let mut check = RedundantImportCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();

    let java_code = "public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = jbuild::checkstyle::api::file::FileContents::new(file_text);
    let ast = JavaParser::parse(&file_contents).unwrap();

    let result = check.begin_tree(ast.as_ref() as &dyn DetailAst);
    assert!(result.is_ok(), "begin_tree should succeed");

    // After begin_tree, violations should be cleared
    let violations = check.get_violations();
    assert_eq!(
        violations.len(),
        0,
        "Violations should be cleared after begin_tree"
    );
}

#[test]
fn test_check_clear_violations() {
    let mut check = EmptyCatchBlockCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();

    // Log a violation (this would normally happen during visit_token)
    // For this test, we'll just verify clear_violations works
    check.clear_violations();
    let violations = check.get_violations();
    assert_eq!(violations.len(), 0, "Violations should be cleared");
}

#[test]
fn test_check_get_default_tokens() {
    let check = EmptyCatchBlockCheck::new();
    let default_tokens = check.get_default_tokens();
    let required_tokens = check.get_required_tokens();
    assert_eq!(
        default_tokens, required_tokens,
        "Default tokens should match required tokens"
    );
}

#[test]
fn test_check_get_acceptable_tokens() {
    let check = EmptyCatchBlockCheck::new();
    let acceptable_tokens = check.get_acceptable_tokens();
    let required_tokens = check.get_required_tokens();
    assert_eq!(
        acceptable_tokens, required_tokens,
        "Acceptable tokens should match required tokens"
    );
}
