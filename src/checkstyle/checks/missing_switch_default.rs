//! MissingSwitchDefault check - detects switch statements without default case

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::checks::base::AbstractCheck;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Check that detects switch statements without a default case
pub struct MissingSwitchDefaultCheck {
    /// Base check implementation
    base: AbstractCheck,
}

impl MissingSwitchDefaultCheck {
    /// Create a new MissingSwitchDefaultCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("MissingSwitchDefault".to_string()),
        }
    }
}

impl Default for MissingSwitchDefaultCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for MissingSwitchDefaultCheck {
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for MissingSwitchDefaultCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for MissingSwitchDefaultCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_SWITCH]
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_SWITCH]
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_SWITCH]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // Check if switch statement has a default case
        // Look for LITERAL_DEFAULT in the switch statement
        if !contains_default_label(ast) {
            // No default case found - log violation
            self.base
                .log_ast(ast, "missing.switch.default".to_string(), vec![]);
        }

        Ok(())
    }

    fn get_violations(&self) -> BTreeSet<Violation> {
        self.base.get_violations()
    }

    fn clear_violations(&mut self) {
        self.base.clear_violations();
    }
}

/// Check if the switch statement contains a default label
fn contains_default_label(ast: &dyn DetailAst) -> bool {
    // Look for LITERAL_DEFAULT in the switch statement
    if ast
        .find_first_token_arc(crate::checkstyle::api::ast::token_types::LITERAL_DEFAULT)
        .is_some()
    {
        return true;
    }

    // Also check children recursively
    if let Some(first_child) = ast.get_first_child_arc() {
        return check_children_for_default(&first_child);
    }

    false
}

/// Recursively check children for default label
fn check_children_for_default(ast: &Arc<dyn DetailAst>) -> bool {
    // Check current node
    if ast.get_type() == crate::checkstyle::api::ast::token_types::LITERAL_DEFAULT {
        return true;
    }

    // Check children
    if let Some(child) = ast.get_first_child_arc() {
        if check_children_for_default(&child) {
            return true;
        }
    }

    // Check siblings
    if let Some(sibling) = ast.get_next_sibling_arc() {
        if check_children_for_default(&sibling) {
            return true;
        }
    }

    false
}
