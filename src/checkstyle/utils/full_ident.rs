//! FullIdent utility for extracting fully qualified names from AST

use crate::checkstyle::api::ast::DetailAst;
use std::sync::Arc;

/// Represents a fully qualified identifier (e.g., "java.util.List")
pub struct FullIdent {
    /// The full text of the identifier
    pub text: String,
    /// The detail AST node
    pub detail_ast: Option<Arc<dyn DetailAst>>,
}

impl FullIdent {
    /// Create a FullIdent from an AST node
    ///
    /// This traverses the AST starting from the given node, collecting
    /// IDENT and DOT tokens to build the full identifier string.
    pub fn create_full_ident(ast: &Arc<dyn DetailAst>) -> Self {
        let mut text_parts = Vec::new();
        let mut current: Option<Arc<dyn DetailAst>> = Some(ast.clone());
        let mut first_ast: Option<Arc<dyn DetailAst>> = None;

        while let Some(node) = current {
            let node_type = node.get_type();

            if node_type == crate::checkstyle::api::ast::token_types::IDENT {
                if first_ast.is_none() {
                    first_ast = Some(node.clone());
                }
                text_parts.push(node.get_text().to_string());
                // Move to next sibling
                current = node.get_next_sibling_arc();
            } else if node_type == crate::checkstyle::api::ast::token_types::DOT {
                text_parts.push(".".to_string());
                // Move to next sibling
                current = node.get_next_sibling_arc();
            } else {
                // For other node types, try to find IDENT in children
                if let Some(child) = node.get_first_child_arc() {
                    current = Some(child);
                } else {
                    break;
                }
            }
        }

        let text = text_parts.join("");

        Self {
            text,
            detail_ast: first_ast,
        }
    }

    /// Get the text of the full identifier
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Get the detail AST node (the first IDENT)
    pub fn get_detail_ast(&self) -> Option<&Arc<dyn DetailAst>> {
        self.detail_ast.as_ref()
    }

    /// Create a FullIdent starting from the first child of the specified node
    /// This is equivalent to FullIdent.createFullIdentBelow in Java Checkstyle
    pub fn create_full_ident_below(ast: &Arc<dyn DetailAst>) -> Self {
        if let Some(first_child) = ast.get_first_child_arc() {
            Self::create_full_ident(&first_child)
        } else {
            Self {
                text: String::new(),
                detail_ast: None,
            }
        }
    }
}
