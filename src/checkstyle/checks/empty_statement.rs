//! EmptyStatement check - detects empty statements (standalone semicolons)

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::checks::base::AbstractCheck;
use std::collections::BTreeSet;

/// Check that detects empty statements (standalone semicolons)
pub struct EmptyStatementCheck {
    /// Base check implementation
    base: AbstractCheck,
}

impl EmptyStatementCheck {
    /// Create a new EmptyStatementCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("EmptyStatement".to_string()),
        }
    }
}

impl Default for EmptyStatementCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for EmptyStatementCheck {
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for EmptyStatementCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for EmptyStatementCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::EMPTY_STAT]
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::EMPTY_STAT]
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::EMPTY_STAT]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // Empty statement detected - log violation
        self.base
            .log_ast(ast, "empty.statement".to_string(), vec![]);
        Ok(())
    }

    fn get_violations(&self) -> BTreeSet<Violation> {
        self.base.get_violations()
    }

    fn clear_violations(&mut self) {
        self.base.clear_violations();
    }
}
