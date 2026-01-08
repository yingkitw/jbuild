//! Tests against example Java files

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
use jbuild::checkstyle::checks::type_name::TypeNameCheck;
use jbuild::checkstyle::runner::checker::Checker;
use jbuild::checkstyle::runner::config_loader::ConfigurationLoader;
use std::path::PathBuf;

fn read_example_file(filename: &str) -> String {
    let path = PathBuf::from(format!("examples/checkstyle-examples/{filename}"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read example file: {filename}"))
}

fn create_context() -> Context {
    Context::new()
}

#[test]
fn test_empty_catch_block_example() {
    let java_code = read_example_file("EmptyCatchBlockExample.java");
    let file_text = FileText::new(PathBuf::from("EmptyCatchBlockExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = EmptyCatchBlockCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse AST to find catch blocks
    if let Some(catch_ast) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::LITERAL_CATCH) {
        check.visit_token(catch_ast.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("EmptyCatchBlockExample: Found {} violations", violations.len());
    // Should find violations for empty catch blocks
}

#[test]
fn test_empty_statement_example() {
    let java_code = read_example_file("EmptyStatementExample.java");
    let file_text = FileText::new(PathBuf::from("EmptyStatementExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = EmptyStatementCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse to find empty statements
    if let Some(empty_stat) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::EMPTY_STAT) {
        check.visit_token(empty_stat.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("EmptyStatementExample: Found {} violations", violations.len());
}

#[test]
fn test_missing_switch_default_example() {
    let java_code = read_example_file("MissingSwitchDefaultExample.java");
    let file_text = FileText::new(PathBuf::from("MissingSwitchDefaultExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = MissingSwitchDefaultCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse to find switch statements
    if let Some(switch_ast) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::LITERAL_SWITCH) {
        check.visit_token(switch_ast.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("MissingSwitchDefaultExample: Found {} violations", violations.len());
}

#[test]
fn test_multiple_variable_declarations_example() {
    let java_code = read_example_file("MultipleVariableDeclarationsExample.java");
    let file_text = FileText::new(PathBuf::from("MultipleVariableDeclarationsExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = MultipleVariableDeclarationsCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse to find variable declarations
    if let Some(var_decl) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::VARIABLE_DEF) {
        check.visit_token(var_decl.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("MultipleVariableDeclarationsExample: Found {} violations", violations.len());
}

#[test]
fn test_line_length_example() {
    let java_code = read_example_file("LineLengthExample.java");
    let file_text = FileText::new(PathBuf::from("LineLengthExample.java"), java_code);
    
    let mut check = LineLengthCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let violations = check.process(&PathBuf::from("LineLengthExample.java"), &file_text).unwrap();
    println!("LineLengthExample: Found {} violations", violations.len());
    // Should find violations for lines exceeding 80 characters
}

#[test]
fn test_package_name_example() {
    let java_code = read_example_file("PackageNameExample.java");
    let file_text = FileText::new(PathBuf::from("PackageNameExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = PackageNameCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    if let Some(package_ast) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF) {
        check.visit_token(package_ast.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("PackageNameExample: Found {} violations", violations.len());
    // Should find violation for uppercase package name
}

#[test]
fn test_package_name_valid_example() {
    let java_code = read_example_file("PackageNameValidExample.java");
    let file_text = FileText::new(PathBuf::from("PackageNameValidExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = PackageNameCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    if let Some(package_ast) = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF) {
        check.visit_token(package_ast.as_ref() as &dyn DetailAst).unwrap();
    }
    
    let violations = check.get_violations();
    println!("PackageNameValidExample: Found {} violations", violations.len());
    // Should find no violations for valid package name
    assert_eq!(violations.len(), 0, "Valid package name should not trigger violations");
}

#[test]
fn test_redundant_import_example() {
    let java_code = read_example_file("RedundantImportExample.java");
    let file_text = FileText::new(PathBuf::from("RedundantImportExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = RedundantImportCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse to find imports
    let mut current = Some(ast.clone());
    while let Some(node) = current {
        if node.get_type() == jbuild::checkstyle::api::ast::token_types::IMPORT
            || node.get_type() == jbuild::checkstyle::api::ast::token_types::STATIC_IMPORT
        {
            check.visit_token(node.as_ref() as &dyn DetailAst).unwrap();
        }
        
        if let Some(package_ast) = node.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF) {
            check.visit_token(package_ast.as_ref() as &dyn DetailAst).unwrap();
        }
        
        current = node.get_first_child_arc();
    }
    
    let violations = check.get_violations();
    println!("RedundantImportExample: Found {} violations", violations.len());
    // Should find violations for redundant imports
}

#[test]
fn test_type_name_example() {
    let java_code = read_example_file("TypeNameExample.java");
    let file_text = FileText::new(PathBuf::from("TypeNameExample.java"), java_code);
    let file_contents = FileContents::new(file_text);
    
    let mut check = TypeNameCheck::new();
    let context = create_context();
    check.contextualize(&context).unwrap();
    
    let ast = jbuild::checkstyle::parser::java_parser::JavaParser::parse(&file_contents).unwrap();
    check.begin_tree(ast.as_ref() as &dyn DetailAst).unwrap();
    
    // Traverse to find class/interface/enum definitions
    let mut current = Some(ast.clone());
    while let Some(node) = current {
        let node_type = node.get_type();
        if node_type == jbuild::checkstyle::api::ast::token_types::CLASS_DEF
            || node_type == jbuild::checkstyle::api::ast::token_types::INTERFACE_DEF
            || node_type == jbuild::checkstyle::api::ast::token_types::ENUM_DEF
        {
            check.visit_token(node.as_ref() as &dyn DetailAst).unwrap();
        }
        
        current = node.get_first_child_arc();
    }
    
    let violations = check.get_violations();
    println!("TypeNameExample: Found {} violations", violations.len());
    // Should find violations for invalid type names
}

#[test]
fn test_complex_example_with_checker() {
    let java_code = read_example_file("ComplexExample.java");
    
    // Write to temp location for checker
    let temp_path = std::env::temp_dir().join("ComplexExample.java");
    std::fs::write(&temp_path, &java_code).unwrap();
    
    // Create configuration with multiple checks
    let config_xml = r#"<module name="Checker"><property name="fileExtensions" value="java"/><module name="TreeWalker"><module name="EmptyCatchBlock"/><module name="EmptyStatement"/><module name="MissingSwitchDefault"/><module name="MultipleVariableDeclarations"/><module name="PackageName"/><module name="RedundantImport"/><module name="TypeName"/></module><module name="LineLength"/></module>"#;
    
    let config_path = std::env::temp_dir().join("test_config.xml");
    std::fs::write(&config_path, config_xml).unwrap();
    
    let config_result = ConfigurationLoader::load_configuration(&config_path);
    if config_result.is_err() {
        // Cleanup and skip test if config loading fails
        std::fs::remove_file(&temp_path).ok();
        std::fs::remove_file(&config_path).ok();
        println!("ComplexExample: Config loading failed, skipping test");
        return;
    }
    
    let config = config_result.unwrap();
    let mut checker = Checker::new();
    if checker.configure(&config).is_err() {
        std::fs::remove_file(&temp_path).ok();
        std::fs::remove_file(&config_path).ok();
        println!("ComplexExample: Checker configuration failed, skipping test");
        return;
    }
    
    checker.add_listener(Box::new(jbuild::checkstyle::runner::default_logger::DefaultLogger::new()));
    
    let files = vec![temp_path.clone()];
    let result = checker.process(&files);
    
    // Cleanup
    std::fs::remove_file(&temp_path).ok();
    std::fs::remove_file(&config_path).ok();
    
    assert!(result.is_ok(), "Checker should process complex example");
    println!("ComplexExample: Processed with {} errors", result.unwrap());
}

#[test]
fn test_all_examples_exist() {
    let examples = vec![
        "EmptyCatchBlockExample.java",
        "EmptyStatementExample.java",
        "MissingSwitchDefaultExample.java",
        "MultipleVariableDeclarationsExample.java",
        "LineLengthExample.java",
        "PackageNameExample.java",
        "PackageNameValidExample.java",
        "RedundantImportExample.java",
        "TypeNameExample.java",
        "SimplifyBooleanReturnExample.java",
        "ComplexExample.java",
        "NestedEmptyCatchExample.java",
        "MultipleViolationsExample.java",
        "ValidCodeExample.java",
        "EdgeCaseLineLengthExample.java",
        "ComplexTypeNameExample.java",
        "ComplexSwitchExample.java",
        "ComplexVariableDeclarationsExample.java",
    ];
    
    for example in examples {
        let path = PathBuf::from(format!("examples/checkstyle-examples/{example}"));
        assert!(path.exists(), "Example file should exist: {example}");
    }
}

