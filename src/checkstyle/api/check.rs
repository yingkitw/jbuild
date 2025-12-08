//! Check traits for Checkstyle-rs

use crate::checkstyle::api::config::{Configurable, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::file::FileText;
use crate::checkstyle::api::violation::Violation;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Message dispatcher for sending audit events
pub trait MessageDispatcher: Send + Sync {
    /// Fire an audit event
    fn fire_audit_event(&self, event: &crate::checkstyle::api::event::AuditEvent) -> CheckstyleResult<()>;
}

/// Interface for checking a set of files
pub trait FileSetCheck: Configurable + Contextualizable + Send + Sync {
    /// Set the message dispatcher
    fn set_message_dispatcher(&mut self, dispatcher: Box<dyn MessageDispatcher>);

    /// Initialize the check
    fn init(&mut self) -> CheckstyleResult<()>;

    /// Clean up the check
    fn destroy(&mut self) -> CheckstyleResult<()>;

    /// Called when about to process a set of files
    fn begin_processing(&mut self, charset: &str) -> CheckstyleResult<()>;

    /// Process a file and return violations
    fn process(
        &mut self,
        file: &PathBuf,
        file_text: &FileText,
    ) -> CheckstyleResult<BTreeSet<Violation>>;

    /// Called when all files have been processed
    fn finish_processing(&mut self) -> CheckstyleResult<()>;
}

/// Base trait for AST-based checks
pub trait Check: Configurable + Contextualizable + Send + Sync {
    /// Get default tokens this check is interested in
    fn get_default_tokens(&self) -> Vec<i32>;

    /// Get acceptable tokens this check can handle
    fn get_acceptable_tokens(&self) -> Vec<i32>;

    /// Get required tokens this check must receive
    fn get_required_tokens(&self) -> Vec<i32>;

    /// Whether comment nodes are required
    fn is_comment_nodes_required(&self) -> bool {
        false
    }

    /// Initialize the check
    fn init(&mut self) -> CheckstyleResult<()> {
        Ok(())
    }

    /// Visit a token in the AST
    fn visit_token(&mut self, ast: &dyn crate::checkstyle::api::ast::DetailAst) -> CheckstyleResult<()> {
        let _ = ast;
        Ok(())
    }

    /// Called when leaving a token
    fn leave_token(&mut self, ast: &dyn crate::checkstyle::api::ast::DetailAst) -> CheckstyleResult<()> {
        let _ = ast;
        Ok(())
    }

    /// Begin tree traversal
    fn begin_tree(&mut self, _ast: &dyn crate::checkstyle::api::ast::DetailAst) -> CheckstyleResult<()> {
        Ok(())
    }

    /// Finish tree traversal
    fn finish_tree(&mut self, _ast: &dyn crate::checkstyle::api::ast::DetailAst) -> CheckstyleResult<()> {
        Ok(())
    }

    /// Get all violations collected by this check
    fn get_violations(&self) -> BTreeSet<Violation>;

    /// Clear all violations collected by this check
    fn clear_violations(&mut self);
}
