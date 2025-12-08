//! Tests for AbstractCheck base functionality

use jbuild::checkstyle::api::ast::DetailAst;
use jbuild::checkstyle::api::check::Check;
use jbuild::checkstyle::api::config::{Configurable, Context, Contextualizable};
use jbuild::checkstyle::checks::base::AbstractCheck;
use jbuild::checkstyle::parser::ast_impl::DetailAstImpl;
use std::sync::Arc;

#[test]
fn test_abstract_check_new() {
    let check = AbstractCheck::new("TestCheck".to_string());
    assert_eq!(check.module_id, "TestCheck");
}

#[test]
fn test_abstract_check_log_ast() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let context = Context::new();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "test".to_string(),
        10,
        5,
    );
    
    check.log_ast(
        Arc::new(ast).as_ref(),
        "test.key".to_string(),
        vec!["arg1".to_string()],
    );
    
    let violations = check.get_violations();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations.iter().next().unwrap().line_no, 10);
    assert_eq!(violations.iter().next().unwrap().column_no, 5);
}

#[test]
fn test_abstract_check_log_ast_multiple() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let context = Context::new();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast1 = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "test1".to_string(),
        10,
        5,
    );
    
    let ast2 = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "test2".to_string(),
        20,
        10,
    );
    
    check.log_ast(Arc::new(ast1).as_ref(), "key1".to_string(), vec![]);
    check.log_ast(Arc::new(ast2).as_ref(), "key2".to_string(), vec![]);
    
    let violations = check.get_violations();
    assert_eq!(violations.len(), 2);
}

#[test]
fn test_abstract_check_clear_violations() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let context = Context::new();
    check.contextualize(&context).unwrap();
    check.init().unwrap();
    
    let ast = DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "test".to_string(),
        10,
        5,
    );
    
    check.log_ast(Arc::new(ast).as_ref(), "test.key".to_string(), vec![]);
    assert_eq!(check.get_violations().len(), 1);
    
    check.clear_violations();
    assert_eq!(check.get_violations().len(), 0);
}

#[test]
fn test_abstract_check_get_module_id() {
    let check = AbstractCheck::new("MyCheck".to_string());
    assert_eq!(check.module_id, "MyCheck");
}

#[test]
fn test_abstract_check_contextualize() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let context = Context::new();
    let result = check.contextualize(&context);
    assert!(result.is_ok());
}

#[test]
fn test_abstract_check_configure() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let config = jbuild::checkstyle::api::config::Configuration::new("TestCheck".to_string());
    let result = check.configure(&config);
    assert!(result.is_ok());
}

#[test]
fn test_abstract_check_init() {
    let mut check = AbstractCheck::new("TestCheck".to_string());
    let result = check.init();
    assert!(result.is_ok());
}

