//! PackageName check - validates package naming conventions

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::checks::base::AbstractCheck;
use crate::checkstyle::utils::full_ident::FullIdent;
use regex::Regex;
use std::sync::Arc;

/// Check that package names conform to a specified pattern
///
/// Default pattern: `^[a-z]+(\.[a-zA-Z_]\w*)*$`
/// This matches Java Language Specification requirements.
pub struct PackageNameCheck {
    base: AbstractCheck,
    /// Pattern to match valid package names
    format: Regex,
}

impl PackageNameCheck {
    /// Create a new PackageNameCheck with default pattern
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("PackageName".to_string()),
            // Default pattern matching Java Language Specification
            format: Regex::new(r"^[a-z]+(\.[a-zA-Z_]\w*)*$").unwrap(),
        }
    }

    /// Set the format pattern
    pub fn set_format(&mut self, pattern: String) -> Result<(), regex::Error> {
        self.format = Regex::new(&pattern)?;
        Ok(())
    }
}

impl Default for PackageNameCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for PackageNameCheck {
    fn configure(&mut self, config: &crate::checkstyle::api::config::Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)?;

        // Read format property
        if let Some(format_str) = config.get_property("format") {
            if let Err(e) = self.set_format(format_str.clone()) {
                return Err(crate::checkstyle::api::error::CheckstyleError::Configuration(format!(
                    "Invalid format pattern: {e}"
                )));
            }
        }

        Ok(())
    }
}

impl Contextualizable for PackageNameCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for PackageNameCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::PACKAGE_DEF]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // Get the package name AST
        // PACKAGE_DEF structure: PACKAGE_DEF -> (MODIFIERS?) -> DOT -> IDENT -> ...
        // We need to find the name part (before SEMI)
        if let Some(name_ast_arc) = ast.get_first_child_arc() {
            // Skip MODIFIERS if present, find DOT or IDENT
            let mut current = Some(name_ast_arc);
            let mut name_node: Option<Arc<dyn DetailAst>> = None;

            while let Some(node) = current {
                let node_type = node.get_type();
                if node_type == crate::checkstyle::api::ast::token_types::DOT
                    || node_type == crate::checkstyle::api::ast::token_types::IDENT
                {
                    name_node = Some(node.clone());
                    break;
                } else if node_type == crate::checkstyle::api::ast::token_types::MODIFIERS {
                    // Skip modifiers, get next sibling
                    current = node.get_next_sibling_arc();
                } else {
                    // Try first child
                    current = node.get_first_child_arc();
                }
            }

            if let Some(name_node_arc) = name_node {
                let full_ident = FullIdent::create_full_ident(&name_node_arc);
                let package_name = full_ident.get_text();

                if !self.format.is_match(package_name) {
                    // Log violation using the detail AST if available
                    if let Some(detail_ast) = full_ident.get_detail_ast() {
                        self.base.log_ast(
                            detail_ast.as_ref() as &dyn DetailAst,
                            "name.invalidPattern".to_string(),
                            vec![package_name.to_string(), self.format.as_str().to_string()],
                        );
                    } else {
                        // Fallback to original AST
                        self.base.log_ast(
                            ast,
                            "name.invalidPattern".to_string(),
                            vec![package_name.to_string(), self.format.as_str().to_string()],
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn get_violations(&self) -> std::collections::BTreeSet<crate::checkstyle::api::violation::Violation> {
        self.base.get_violations()
    }

    fn clear_violations(&mut self) {
        self.base.clear_violations();
    }
}
