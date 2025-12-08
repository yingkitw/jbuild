//! RedundantImport check - detects redundant import statements

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::checks::base::AbstractCheck;
use crate::checkstyle::utils::full_ident::FullIdent;
use std::collections::HashSet;

/// Check for redundant import statements
///
/// An import is considered redundant if:
/// - It's a duplicate of another import
/// - It's from java.lang package (e.g., java.lang.String)
/// - It's from the same package as the current file
pub struct RedundantImportCheck {
    base: AbstractCheck,
    /// Set of imports seen so far
    imports: HashSet<String>,
    /// Set of static imports seen so far
    static_imports: HashSet<String>,
    /// Name of the package in the current file
    package_name: Option<String>,
}

impl RedundantImportCheck {
    /// Create a new RedundantImportCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("RedundantImport".to_string()),
            imports: HashSet::new(),
            static_imports: HashSet::new(),
            package_name: None,
        }
    }

    /// Check if an import is from a specified package
    fn is_from_package(import_name: &str, pkg: &str) -> bool {
        // Find the last dot to get the package part
        if let Some(index) = import_name.rfind('.') {
            let front = &import_name[..index];
            pkg == front
        } else {
            false
        }
    }
}

impl Default for RedundantImportCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for RedundantImportCheck {
    fn configure(&mut self, config: &crate::checkstyle::api::config::Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for RedundantImportCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for RedundantImportCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![
            crate::checkstyle::api::ast::token_types::IMPORT,
            crate::checkstyle::api::ast::token_types::STATIC_IMPORT,
            crate::checkstyle::api::ast::token_types::PACKAGE_DEF,
        ]
    }

    fn begin_tree(&mut self, _ast: &dyn DetailAst) -> CheckstyleResult<()> {
        self.package_name = None;
        self.imports.clear();
        self.static_imports.clear();
        Ok(())
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        let token_type = ast.get_type();

        if token_type == crate::checkstyle::api::ast::token_types::PACKAGE_DEF {
            // Extract package name
            if let Some(name_node) = ast.get_first_child_arc() {
                // Skip MODIFIERS if present, find the package name part
                let mut current = Some(name_node);
                while let Some(node) = current {
                    let node_type = node.get_type();
                    if node_type == crate::checkstyle::api::ast::token_types::DOT
                        || node_type == crate::checkstyle::api::ast::token_types::IDENT
                    {
                        let full_ident = FullIdent::create_full_ident(&node);
                        self.package_name = Some(full_ident.get_text().to_string());
                        break;
                    } else if node_type == crate::checkstyle::api::ast::token_types::MODIFIERS {
                        current = node.get_next_sibling_arc();
                    } else {
                        current = node.get_first_child_arc();
                    }
                }
            }
        } else if token_type == crate::checkstyle::api::ast::token_types::IMPORT {
            // Regular import
            if let Some(import_node) = ast.get_first_child_arc() {
                let full_ident = FullIdent::create_full_ident_below(&import_node);
                let import_text = full_ident.get_text();

                // Check if from java.lang
                if Self::is_from_package(import_text, "java.lang") {
                    self.base.log_ast(
                        ast,
                        "import.lang".to_string(),
                        vec![import_text.to_string()],
                    );
                }
                // Check if from same package
                else if let Some(ref pkg) = self.package_name {
                    if Self::is_from_package(import_text, pkg) {
                        self.base.log_ast(
                            ast,
                            "import.same".to_string(),
                            vec![import_text.to_string()],
                        );
                    }
                }

                // Check for duplicate
                if self.imports.contains(import_text) {
                    self.base.log_ast(
                        ast,
                        "import.duplicate".to_string(),
                        vec![import_text.to_string()],
                    );
                } else {
                    self.imports.insert(import_text.to_string());
                }
            }
        } else if token_type == crate::checkstyle::api::ast::token_types::STATIC_IMPORT {
            // Static import
            if let Some(import_node) = ast.get_first_child_arc() {
                // For static imports, get the identifier part
                let full_ident = FullIdent::create_full_ident(&import_node);
                let import_text = full_ident.get_text();

                // Check for duplicate static import
                if self.static_imports.contains(import_text) {
                    self.base.log_ast(
                        ast,
                        "import.duplicate".to_string(),
                        vec![import_text.to_string()],
                    );
                } else {
                    self.static_imports.insert(import_text.to_string());
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
