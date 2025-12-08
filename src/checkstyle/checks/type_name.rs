//! TypeName check - validates type naming conventions

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::checks::base::AbstractCheck;
use crate::checkstyle::utils::ast_util::get_identifier_name;
use regex::Regex;

/// Check that type names conform to a specified pattern
///
/// Default pattern: `^[A-Z][a-zA-Z0-9]*$` (PascalCase)
/// Applies to: classes, interfaces, enums, annotations
pub struct TypeNameCheck {
    base: AbstractCheck,
    /// Pattern to match valid type names
    format: Regex,
    /// Apply to public types
    apply_to_public: bool,
    /// Apply to protected types
    apply_to_protected: bool,
    /// Apply to package-private types
    apply_to_package: bool,
    /// Apply to private types
    apply_to_private: bool,
}

impl TypeNameCheck {
    /// Create a new TypeNameCheck with default pattern
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("TypeName".to_string()),
            // Default pattern: PascalCase
            format: Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap(),
            apply_to_public: true,
            apply_to_protected: true,
            apply_to_package: true,
            apply_to_private: true,
        }
    }

    /// Set the format pattern
    pub fn set_format(&mut self, pattern: String) -> Result<(), regex::Error> {
        self.format = Regex::new(&pattern)?;
        Ok(())
    }

    /// Check if we should check this type based on access modifiers
    fn should_check(&self, ast: &dyn DetailAst) -> bool {
        // For types (classes, interfaces, etc.), we check all by default
        // Access control is more relevant for members, but we support it for types too
        if ast
            .find_first_token_arc(crate::checkstyle::api::ast::token_types::MODIFIERS)
            .is_some()
        {
            let is_public = crate::checkstyle::utils::ast_util::is_public(ast);
            let is_protected = crate::checkstyle::utils::ast_util::is_protected(ast);
            let is_private = crate::checkstyle::utils::ast_util::is_private(ast);
            let is_package = !(is_public || is_protected || is_private);

            return (self.apply_to_public && is_public)
                || (self.apply_to_protected && is_protected)
                || (self.apply_to_package && is_package)
                || (self.apply_to_private && is_private);
        }

        // Default: check if no modifiers found (package-private)
        self.apply_to_package
    }
}

impl Default for TypeNameCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for TypeNameCheck {
    fn configure(&mut self, config: &crate::checkstyle::api::config::Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)?;

        // Read format property
        if let Some(format_str) = config.get_property("format") {
            if let Err(e) = self.set_format(format_str.clone()) {
                return Err(crate::checkstyle::api::error::CheckstyleError::Configuration(format!(
                    "Invalid format pattern: {}",
                    e
                )));
            }
        }

        // Read access control properties
        if let Some(apply_str) = config.get_property("applyToPublic") {
            self.apply_to_public = apply_str.parse().unwrap_or(true);
        }
        if let Some(apply_str) = config.get_property("applyToProtected") {
            self.apply_to_protected = apply_str.parse().unwrap_or(true);
        }
        if let Some(apply_str) = config.get_property("applyToPackage") {
            self.apply_to_package = apply_str.parse().unwrap_or(true);
        }
        if let Some(apply_str) = config.get_property("applyToPrivate") {
            self.apply_to_private = apply_str.parse().unwrap_or(true);
        }

        Ok(())
    }
}

impl Contextualizable for TypeNameCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for TypeNameCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        self.get_acceptable_tokens()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        vec![
            crate::checkstyle::api::ast::token_types::CLASS_DEF,
            crate::checkstyle::api::ast::token_types::INTERFACE_DEF,
            crate::checkstyle::api::ast::token_types::ENUM_DEF,
            // ANNOTATION_DEF and RECORD_DEF not yet supported
        ]
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![] // Empty - not required
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        if !self.should_check(ast) {
            return Ok(());
        }

        // Get the type name
        if let Some(type_name) = get_identifier_name(ast) {
            if !self.format.is_match(&type_name) {
                self.base.log_ast(
                    ast,
                    "name.invalidPattern".to_string(),
                    vec![type_name, self.format.as_str().to_string()],
                );
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
