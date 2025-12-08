//! AST utility functions

use crate::checkstyle::api::ast::DetailAst;
use std::sync::Arc;

/// Check if an AST node has a specific modifier
pub fn has_modifier(ast: &dyn DetailAst, modifier_type: i32) -> bool {
    if let Some(modifiers) = ast.find_first_token_arc(crate::checkstyle::api::ast::token_types::MODIFIERS) {
        return modifiers.find_first_token_arc(modifier_type).is_some();
    }
    false
}

/// Check if a class is abstract
pub fn is_abstract(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_ABSTRACT)
}

/// Check if a class/method is static
pub fn is_static(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_STATIC)
}

/// Check if a class/method is final
pub fn is_final(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_FINAL)
}

/// Check if a class/method is public
pub fn is_public(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_PUBLIC)
}

/// Check if a class/method is private
pub fn is_private(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_PRIVATE)
}

/// Check if a class/method is protected
pub fn is_protected(ast: &dyn DetailAst) -> bool {
    has_modifier(ast, crate::checkstyle::api::ast::token_types::LITERAL_PROTECTED)
}

/// Get the name identifier from an AST node (for methods, classes, etc.)
pub fn get_identifier_name(ast: &dyn DetailAst) -> Option<String> {
    if let Some(ident) = ast.find_first_token_arc(crate::checkstyle::api::ast::token_types::IDENT) {
        return Some(ident.get_text().to_string());
    }
    None
}

/// Check if a method is an equals method
pub fn is_equals_method(ast: &dyn DetailAst) -> bool {
    if ast.get_type() != crate::checkstyle::api::ast::token_types::METHOD_DEF {
        return false;
    }

    // Check if it's static or abstract
    if is_static(ast) || is_abstract(ast) {
        return false;
    }

    // Check if name is "equals"
    if let Some(name) = get_identifier_name(ast) {
        if name == "equals" {
            // Check if it has exactly one parameter
            if let Some(params) = ast.find_first_token_arc(crate::checkstyle::api::ast::token_types::PARAMETERS)
            {
                return params.get_child_count() == 1;
            }
        }
    }

    false
}

/// Find first token of a specific type in the AST tree (recursive)
pub fn find_first_token_recursive(
    ast: &Arc<dyn DetailAst>,
    token_type: i32,
) -> Option<Arc<dyn DetailAst>> {
    // Check current node
    if ast.get_type() == token_type {
        return Some(ast.clone());
    }

    // Check children
    if let Some(child) = ast.get_first_child_arc() {
        if let Some(found) = find_first_token_recursive(&child, token_type) {
            return Some(found);
        }

        // Check siblings
        let mut sibling = child.get_next_sibling_arc();
        while let Some(s) = sibling {
            if let Some(found) = find_first_token_recursive(&s, token_type) {
                return Some(found);
            }
            sibling = s.get_next_sibling_arc();
        }
    }

    None
}

/// Finds sub-node for given node minimal (line, column) pair.
/// This is equivalent to CheckUtil.getFirstNode in Java Checkstyle.
pub fn get_first_node(ast: &Arc<dyn DetailAst>) -> Arc<dyn DetailAst> {
    let mut current_node = ast.clone();

    if let Some(child) = ast.get_first_child_arc() {
        let first_child_node = get_first_node(&child);
        if is_before_in_source(&first_child_node, &current_node) {
            current_node = first_child_node;
        }

        // Check all siblings
        let mut sibling = child.get_next_sibling_arc();
        while let Some(s) = sibling {
            let sibling_first = get_first_node(&s);
            if is_before_in_source(&sibling_first, &current_node) {
                current_node = sibling_first;
            }
            sibling = s.get_next_sibling_arc();
        }
    }

    current_node
}

/// Finds sub-node for given node maximum (line, column) pair.
/// This is equivalent to the getLastNode method in MultipleVariableDeclarationsCheck.
pub fn get_last_node(ast: &Arc<dyn DetailAst>) -> Arc<dyn DetailAst> {
    let mut current_node = ast.clone();

    if let Some(child) = ast.get_last_child_arc() {
        let last_child_node = get_last_node(&child);
        if !is_before_in_source(&last_child_node, &current_node) {
            current_node = last_child_node;
        }
    }

    current_node
}

/// Checks if two AST nodes are on the same line
pub fn are_on_same_line(ast1: &Arc<dyn DetailAst>, ast2: &Arc<dyn DetailAst>) -> bool {
    ast1.get_line_no() == ast2.get_line_no()
}

/// Checks if ast1 is located before ast2 in source
pub fn is_before_in_source(ast1: &Arc<dyn DetailAst>, ast2: &Arc<dyn DetailAst>) -> bool {
    ast1.get_line_no() < ast2.get_line_no()
        || (are_on_same_line(ast1, ast2) && ast1.get_column_no() < ast2.get_column_no())
}
