//! Base check implementations

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::violation::Violation;
use std::collections::BTreeSet;
use std::sync::Mutex;

/// Base implementation for AST-based checks
pub struct AbstractCheck {
    /// Module ID
    pub module_id: String,
    /// Context
    pub context: Context,
    /// Violations collected by this check
    violations: Mutex<BTreeSet<Violation>>,
}

impl AbstractCheck {
    /// Create a new abstract check
    pub fn new(module_id: String) -> Self {
        Self {
            module_id,
            context: Context::new(),
            violations: Mutex::new(BTreeSet::new()),
        }
    }

    /// Log a violation at a specific line
    pub fn log(&self, line: usize, key: String, args: Vec<String>) {
        let violation = Violation::new(
            line,
            0,
            0,
            0,
            self.context.severity,
            self.module_id.clone(),
            key,
            args,
            "checkstyle".to_string(),
            self.module_id.clone(),
            None,
        );
        let mut violations = self.violations.lock().unwrap();
        violations.insert(violation);
    }

    /// Log a violation at a specific line and column
    pub fn log_with_column(&self, line: usize, column: usize, key: String, args: Vec<String>) {
        let violation = Violation::new(
            line,
            column,
            0,
            0,
            self.context.severity,
            self.module_id.clone(),
            key,
            args,
            "checkstyle".to_string(),
            self.module_id.clone(),
            None,
        );
        let mut violations = self.violations.lock().unwrap();
        violations.insert(violation);
    }

    /// Log a violation from an AST node
    pub fn log_ast(&self, ast: &dyn DetailAst, key: String, args: Vec<String>) {
        let violation = Violation::new(
            ast.get_line_no(),
            ast.get_column_no(),
            ast.get_column_no(),
            ast.get_type(),
            self.context.severity,
            self.module_id.clone(),
            key,
            args,
            "checkstyle".to_string(),
            self.module_id.clone(),
            None,
        );
        let mut violations = self.violations.lock().unwrap();
        violations.insert(violation);
    }
}

impl Configurable for AbstractCheck {
    fn configure(&mut self, _config: &Configuration) -> CheckstyleResult<()> {
        Ok(())
    }
}

impl Contextualizable for AbstractCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.context = context.clone();
        Ok(())
    }
}

impl Check for AbstractCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        Vec::new()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        Vec::new()
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        Vec::new()
    }

    fn get_violations(&self) -> BTreeSet<Violation> {
        self.violations.lock().unwrap().clone()
    }

    fn clear_violations(&mut self) {
        self.violations.lock().unwrap().clear();
    }
}
