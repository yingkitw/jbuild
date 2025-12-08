//! EmptyCatchBlock check - detects empty catch blocks

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::checks::base::AbstractCheck;
use std::collections::BTreeSet;

/// Check that detects empty catch blocks
pub struct EmptyCatchBlockCheck {
    /// Base check implementation
    base: AbstractCheck,
}

impl EmptyCatchBlockCheck {
    /// Create a new EmptyCatchBlockCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("EmptyCatchBlock".to_string()),
        }
    }
}

impl Default for EmptyCatchBlockCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for EmptyCatchBlockCheck {
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for EmptyCatchBlockCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for EmptyCatchBlockCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_CATCH]
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_CATCH]
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_CATCH]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // Check if catch block is empty
        // A catch block is empty if it has no statements in its body

        // Look for SLIST (statement list) child using Arc-based method
        if let Some(slist_arc) = ast.find_first_token_arc(crate::checkstyle::api::ast::token_types::SLIST) {
            let slist_child_count = slist_arc.get_child_count();
            if slist_child_count == 0 {
                // Empty catch block found - log violation
                self.base
                    .log_ast(ast, "empty.catch.block".to_string(), vec![]);
            }
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
