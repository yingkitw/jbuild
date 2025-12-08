//! Tests for FullIdent utility

use jbuild::checkstyle::parser::ast_impl::DetailAstImpl;
use jbuild::checkstyle::utils::full_ident::FullIdent;
use std::sync::Arc;

#[test]
fn test_full_ident_simple_ident() {
    let ident = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "Test".to_string(),
        1,
        1,
    ));
    
    let full_ident = FullIdent::create_full_ident(&(ident as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    assert_eq!(full_ident.get_text(), "Test");
}

#[test]
fn test_full_ident_get_detail_ast() {
    let ident = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "Test".to_string(),
        1,
        1,
    ));
    
    let full_ident = FullIdent::create_full_ident(&(ident as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    assert!(full_ident.get_detail_ast().is_some());
    if let Some(ast) = full_ident.get_detail_ast() {
        assert_eq!(ast.get_text(), "Test");
    }
}

#[test]
fn test_full_ident_get_line_no() {
    let ident = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "Test".to_string(),
        5,
        10,
    ));
    
    let full_ident = FullIdent::create_full_ident(&(ident as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    if let Some(ast) = full_ident.get_detail_ast() {
        assert_eq!(ast.get_line_no(), 5);
    }
}

#[test]
fn test_full_ident_get_column_no() {
    let ident = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "Test".to_string(),
        5,
        10,
    ));
    
    let full_ident = FullIdent::create_full_ident(&(ident as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    if let Some(ast) = full_ident.get_detail_ast() {
        assert_eq!(ast.get_column_no(), 10);
    }
}

#[test]
fn test_full_ident_create_full_ident_below() {
    let parent = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IMPORT,
        "import".to_string(),
        1,
        1,
    ));
    
    let child = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IDENT,
        "java".to_string(),
        1,
        8,
    ));
    
    parent.set_first_child(Some(child));
    
    let full_ident = FullIdent::create_full_ident_below(&(parent as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    assert_eq!(full_ident.get_text(), "java");
}

#[test]
fn test_full_ident_create_full_ident_below_no_child() {
    let parent = Arc::new(DetailAstImpl::new(
        jbuild::checkstyle::api::ast::token_types::IMPORT,
        "import".to_string(),
        1,
        1,
    ));
    
    let full_ident = FullIdent::create_full_ident_below(&(parent as Arc<dyn jbuild::checkstyle::api::ast::DetailAst>));
    assert_eq!(full_ident.get_text(), "");
}
