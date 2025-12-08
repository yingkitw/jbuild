//! Tests for parser functionality

use jbuild::checkstyle::api::ast::DetailAst;
use jbuild::checkstyle::api::file::{FileContents, FileText};
use jbuild::checkstyle::parser::java_parser::JavaParser;
use std::path::PathBuf;

#[test]
fn test_java_parser_simple_class() {
    let java_code = "public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let result = JavaParser::parse(&file_contents);
    assert!(result.is_ok(), "Should parse simple class");

    let ast = result.unwrap();
    assert_eq!(
        ast.get_type(),
        jbuild::checkstyle::api::ast::token_types::COMPILATION_UNIT,
        "Root should be COMPILATION_UNIT"
    );
}

#[test]
fn test_java_parser_with_package() {
    let java_code = "package com.example; public class Test {}";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let result = JavaParser::parse(&file_contents);
    assert!(result.is_ok(), "Should parse class with package");

    let ast = result.unwrap();
    // Should be able to find package definition
    let package = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF);
    assert!(package.is_some(), "Should find package definition");
}

#[test]
fn test_java_parser_with_imports() {
    let java_code = r#"
import java.util.List;
import java.util.Map;
public class Test {}
"#;
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let result = JavaParser::parse(&file_contents);
    assert!(result.is_ok(), "Should parse class with imports");
}

#[test]
fn test_java_parser_with_method() {
    let java_code = r#"
public class Test {
    public void test() {
        System.out.println("test");
    }
}
"#;
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let result = JavaParser::parse(&file_contents);
    assert!(result.is_ok(), "Should parse class with method");

    let ast = result.unwrap();
    let method = ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::METHOD_DEF);
    assert!(method.is_some(), "Should find method definition");
}

#[test]
fn test_java_parser_invalid_syntax() {
    let java_code = "public class {"; // Invalid - missing class name
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    // Parser might still succeed (tree-sitter is lenient) or might fail
    // We just verify it doesn't panic
    let _result = JavaParser::parse(&file_contents);
    // Result might be Ok or Err depending on tree-sitter behavior
    assert!(true, "Parser should handle invalid syntax gracefully");
}

#[test]
fn test_java_parser_empty_file() {
    let java_code = "";
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let _result = JavaParser::parse(&file_contents);
    // Empty file might parse or might fail - just verify no panic
    assert!(true, "Parser should handle empty file");
}

#[test]
fn test_java_parser_complex_class() {
    let java_code = r#"
package com.example;
import java.util.List;
import java.util.Map;

public class Test {
    private int field;
    
    public Test() {
        this.field = 0;
    }
    
    public void method(int param) {
        if (param > 0) {
            System.out.println("positive");
        } else {
            System.out.println("non-positive");
        }
    }
}
"#;
    let file_text = FileText::new(PathBuf::from("Test.java"), java_code.to_string());
    let file_contents = FileContents::new(file_text);

    let result = JavaParser::parse(&file_contents);
    assert!(result.is_ok(), "Should parse complex class");

    let ast = result.unwrap();
    // Verify we can find various elements
    assert!(
        ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::PACKAGE_DEF)
            .is_some(),
        "Should find package"
    );
    assert!(
        ast.find_first_token_arc(jbuild::checkstyle::api::ast::token_types::CLASS_DEF)
            .is_some(),
        "Should find class"
    );
}
