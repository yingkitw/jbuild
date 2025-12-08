//! Tests for AST utility functions

use jbuild::checkstyle::parser::ast_impl::DetailAstImpl;
use jbuild::checkstyle::utils::ast_util::{
    has_modifier, is_abstract, is_final, is_private, is_protected, is_public, is_static,
    get_identifier_name, is_equals_method,
};
use std::sync::Arc;

fn create_test_ast_with_modifier(
    text: &str,
    modifier_type: i32,
) -> Arc<DetailAstImpl> {
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::CLASS_DEF,
        text.to_string(),
        1,
        1,
    );
    
    let modifiers = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::MODIFIERS,
        "modifiers".to_string(),
        1,
        1,
    );
    
    let modifier = DetailAstImpl::new(
        modifier_type,
        text.to_string(),
        1,
        1,
    );
    
    modifiers.set_first_child(Some(Arc::new(modifier)));
    ast.set_first_child(Some(Arc::new(modifiers)));
    
    Arc::new(ast)
}

#[test]
fn test_has_modifier_public() {
    let ast = create_test_ast_with_modifier("public", jbuild::checkstyle::api::ast::token_types::LITERAL_PUBLIC);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_PUBLIC));
}

#[test]
fn test_has_modifier_private() {
    let ast = create_test_ast_with_modifier("private", jbuild::checkstyle::api::ast::token_types::LITERAL_PRIVATE);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_PRIVATE));
}

#[test]
fn test_has_modifier_protected() {
    let ast = create_test_ast_with_modifier("protected", jbuild::checkstyle::api::ast::token_types::LITERAL_PROTECTED);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_PROTECTED));
}

#[test]
fn test_has_modifier_static() {
    let ast = create_test_ast_with_modifier("static", jbuild::checkstyle::api::ast::token_types::LITERAL_STATIC);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_STATIC));
}

#[test]
fn test_has_modifier_final() {
    let ast = create_test_ast_with_modifier("final", jbuild::checkstyle::api::ast::token_types::LITERAL_FINAL);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_FINAL));
}

#[test]
fn test_has_modifier_abstract() {
    let ast = create_test_ast_with_modifier("abstract", jbuild::checkstyle::api::ast::token_types::LITERAL_ABSTRACT);
    assert!(has_modifier(ast.as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_ABSTRACT));
}

#[test]
fn test_has_modifier_no_modifiers() {
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::CLASS_DEF,
        "Test".to_string(),
        1,
        1,
    );
    assert!(!has_modifier(Arc::new(ast).as_ref(), jbuild::checkstyle::api::ast::token_types::LITERAL_PUBLIC));
}

#[test]
fn test_is_public() {
    let ast = create_test_ast_with_modifier("public", jbuild::checkstyle::api::ast::token_types::LITERAL_PUBLIC);
    assert!(is_public(ast.as_ref()));
}

#[test]
fn test_is_private() {
    let ast = create_test_ast_with_modifier("private", jbuild::checkstyle::api::ast::token_types::LITERAL_PRIVATE);
    assert!(is_private(ast.as_ref()));
}

#[test]
fn test_is_protected() {
    let ast = create_test_ast_with_modifier("protected", jbuild::checkstyle::api::ast::token_types::LITERAL_PROTECTED);
    assert!(is_protected(ast.as_ref()));
}

#[test]
fn test_is_static() {
    let ast = create_test_ast_with_modifier("static", jbuild::checkstyle::api::ast::token_types::LITERAL_STATIC);
    assert!(is_static(ast.as_ref()));
}

#[test]
fn test_is_final() {
    let ast = create_test_ast_with_modifier("final", jbuild::checkstyle::api::ast::token_types::LITERAL_FINAL);
    assert!(is_final(ast.as_ref()));
}

#[test]
fn test_is_abstract() {
    let ast = create_test_ast_with_modifier("abstract", jbuild::checkstyle::api::ast::token_types::LITERAL_ABSTRACT);
    assert!(is_abstract(ast.as_ref()));
}

#[test]
fn test_get_identifier_name() {
    let ident = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "TestClass".to_string(),
        1,
        1,
    );
    
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::CLASS_DEF,
        "class".to_string(),
        1,
        1,
    );
    ast.set_first_child(Some(Arc::new(ident)));
    
    let name = get_identifier_name(Arc::new(ast).as_ref());
    assert_eq!(name, Some("TestClass".to_string()));
}

#[test]
fn test_get_identifier_name_no_ident() {
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::CLASS_DEF,
        "class".to_string(),
        1,
        1,
    );
    
    let name = get_identifier_name(Arc::new(ast).as_ref());
    assert_eq!(name, None);
}

#[test]
fn test_is_equals_method_not_method() {
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::CLASS_DEF,
        "class".to_string(),
        1,
        1,
    );
    assert!(!is_equals_method(Arc::new(ast).as_ref()));
}

#[test]
fn test_is_equals_method_wrong_name() {
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::METHOD_DEF,
        "method".to_string(),
        1,
        1,
    );
    
    let name = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "notEquals".to_string(),
        1,
        1,
    );
    ast.set_first_child(Some(Arc::new(name)));
    
    assert!(!is_equals_method(Arc::new(ast).as_ref()));
}

