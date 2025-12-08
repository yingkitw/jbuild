//! TreeWalker implementation for Checkstyle-rs

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::Context;
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::event::SeverityLevel;
use crate::checkstyle::api::file::{FileContents, FileText};
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::parser::ast_impl::DetailAstImpl;
use crate::checkstyle::parser::java_parser::JavaParser;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// State of AST during traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AstState {
    /// Ordinary AST without comments
    Ordinary,
    /// AST with comments
    WithComments,
}

/// TreeWalker - walks AST and notifies checks
pub struct TreeWalker {
    /// Maps from token type to ordinary check indices
    token_to_ordinary_checks: HashMap<i32, Vec<usize>>,
    /// Maps from token type to comment check indices
    token_to_comment_checks: HashMap<i32, Vec<usize>>,
    /// Registered ordinary checks
    ordinary_checks: Vec<Arc<Mutex<dyn Check>>>,
    /// Registered comment checks
    comment_checks: Vec<Arc<Mutex<dyn Check>>>,
    /// Context for child components
    child_context: Context,
    /// Control whether to skip files with Java parsing exceptions
    skip_file_on_java_parse_exception: bool,
    /// Severity level for Java parse exceptions
    java_parse_exception_severity: SeverityLevel,
}

impl TreeWalker {
    /// Create a new TreeWalker
    pub fn new() -> Self {
        Self {
            token_to_ordinary_checks: HashMap::new(),
            token_to_comment_checks: HashMap::new(),
            ordinary_checks: Vec::new(),
            comment_checks: Vec::new(),
            child_context: Context::new(),
            skip_file_on_java_parse_exception: false,
            java_parse_exception_severity: SeverityLevel::Error,
        }
    }

    /// Add a check to the tree walker
    pub fn add_check(&mut self, check: Arc<Mutex<dyn Check>>) -> CheckstyleResult<()> {
        // Initialize the check
        {
            let mut check_guard = check.lock().unwrap();
            check_guard.init()?;

            // Get tokens the check is interested in
            let tokens = check_guard.get_default_tokens();
            let requires_comments = check_guard.is_comment_nodes_required();
            drop(check_guard);

            // Register check for each token
            if requires_comments {
                let check_idx = self.comment_checks.len();
                self.comment_checks.push(check.clone());
                for token_id in tokens {
                    self.token_to_comment_checks
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(check_idx);
                }
            } else {
                let check_idx = self.ordinary_checks.len();
                self.ordinary_checks.push(check.clone());
                for token_id in tokens {
                    self.token_to_ordinary_checks
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(check_idx);
                }
            }
        }

        Ok(())
    }

    /// Process a file and return violations
    pub fn process_file(
        &self,
        _file: &PathBuf,
        file_text: &FileText,
    ) -> CheckstyleResult<BTreeSet<Violation>> {
        let mut violations = BTreeSet::new();

        if self.ordinary_checks.is_empty() && self.comment_checks.is_empty() {
            return Ok(violations);
        }

        let file_contents = FileContents::new(file_text.clone());

        // Parse the file
        let root_ast = match JavaParser::parse(&file_contents) {
            Ok(ast) => ast,
            Err(e) => {
                if self.skip_file_on_java_parse_exception {
                    // Add parse exception violation
                    let violation = Violation::new(
                        1,
                        0,
                        0,
                        0,
                        self.java_parse_exception_severity,
                        "TreeWalker".to_string(),
                        "parse.exception".to_string(),
                        vec![e.to_string()],
                        "checkstyle".to_string(),
                        "TreeWalker".to_string(),
                        None,
                    );
                    violations.insert(violation);
                    return Ok(violations);
                } else {
                    return Err(e);
                }
            }
        };

        // Walk the AST
        if !self.ordinary_checks.is_empty() {
            let file_violations = self.walk(&root_ast, &file_contents, AstState::Ordinary)?;
            violations.extend(file_violations);
        }

        if !self.comment_checks.is_empty() {
            // TODO: Implement appendHiddenCommentNodes
            // For now, use the same AST
            let file_violations = self.walk(&root_ast, &file_contents, AstState::WithComments)?;
            violations.extend(file_violations);
        }

        Ok(violations)
    }

    /// Walk the AST and collect violations
    fn walk(
        &self,
        ast: &Arc<DetailAstImpl>,
        contents: &FileContents,
        ast_state: AstState,
    ) -> CheckstyleResult<BTreeSet<Violation>> {
        let mut violations = BTreeSet::new();

        // Notify begin
        self.notify_begin(ast, contents, ast_state)?;

        // Process iteratively
        self.process_iter(ast, ast_state)?;

        // Notify end and collect violations
        let end_violations = self.notify_end(ast, ast_state)?;
        violations.extend(end_violations);

        Ok(violations)
    }

    /// Notify checks that we are about to begin walking a tree
    fn notify_begin(
        &self,
        root_ast: &Arc<DetailAstImpl>,
        _contents: &FileContents,
        ast_state: AstState,
    ) -> CheckstyleResult<()> {
        let checks = if ast_state == AstState::WithComments {
            &self.comment_checks
        } else {
            &self.ordinary_checks
        };

        for check in checks.iter() {
            let mut check_guard = check.lock().unwrap();
            // Clear violations before starting a new tree
            check_guard.clear_violations();
            check_guard.begin_tree(root_ast.as_ref() as &dyn crate::checkstyle::api::ast::DetailAst)?;
        }

        Ok(())
    }

    /// Notify checks that we have finished walking a tree
    fn notify_end(
        &self,
        root_ast: &Arc<DetailAstImpl>,
        ast_state: AstState,
    ) -> CheckstyleResult<BTreeSet<Violation>> {
        let mut violations = BTreeSet::new();
        let checks = if ast_state == AstState::WithComments {
            &self.comment_checks
        } else {
            &self.ordinary_checks
        };

        for check in checks.iter() {
            let mut check_guard = check.lock().unwrap();
            check_guard.finish_tree(root_ast.as_ref() as &dyn crate::checkstyle::api::ast::DetailAst)?;
            // Collect violations from check
            let check_violations = check_guard.get_violations();
            violations.extend(check_violations);
            // Clear violations for next file
            check_guard.clear_violations();
        }

        Ok(violations)
    }

    /// Process AST nodes iteratively
    fn process_iter(&self, root: &Arc<DetailAstImpl>, ast_state: AstState) -> CheckstyleResult<()> {
        // Iterative traversal - using the algorithm from Java TreeWalker
        let mut cur_node: Option<Arc<DetailAstImpl>> = Some(root.clone());

        while let Some(node) = &cur_node {
            // Notify visit
            self.notify_visit(node, ast_state)?;

            // Get first child
            let mut to_visit = node.get_first_child_arc();

            // If no child, go to next sibling or parent
            while to_visit.is_none() {
                if let Some(current) = &cur_node {
                    self.notify_leave(current, ast_state)?;
                    to_visit = current.get_next_sibling_arc();
                    cur_node = current.parent.clone();
                } else {
                    break;
                }
            }

            cur_node = to_visit;
        }

        Ok(())
    }

    /// Notify checks that visiting a node
    fn notify_visit(&self, ast: &Arc<DetailAstImpl>, ast_state: AstState) -> CheckstyleResult<()> {
        let token_id = ast.get_type();
        let check_indices = self.get_list_of_check_indices(token_id, ast_state);
        let checks = if ast_state == AstState::WithComments {
            &self.comment_checks
        } else {
            &self.ordinary_checks
        };

        for &idx in &check_indices {
            if let Some(check) = checks.get(idx) {
                let mut check_guard = check.lock().unwrap();
                check_guard.visit_token(ast.as_ref() as &dyn crate::checkstyle::api::ast::DetailAst)?;
            }
        }

        Ok(())
    }

    /// Notify checks that leaving a node
    fn notify_leave(&self, ast: &Arc<DetailAstImpl>, ast_state: AstState) -> CheckstyleResult<()> {
        let token_id = ast.get_type();
        let check_indices = self.get_list_of_check_indices(token_id, ast_state);
        let checks = if ast_state == AstState::WithComments {
            &self.comment_checks
        } else {
            &self.ordinary_checks
        };

        for &idx in &check_indices {
            if let Some(check) = checks.get(idx) {
                let mut check_guard = check.lock().unwrap();
                check_guard.leave_token(ast.as_ref() as &dyn crate::checkstyle::api::ast::DetailAst)?;
            }
        }

        Ok(())
    }

    /// Get list of check indices for a token type
    fn get_list_of_check_indices(&self, token_id: i32, ast_state: AstState) -> Vec<usize> {
        if ast_state == AstState::WithComments {
            self.token_to_comment_checks
                .get(&token_id)
                .cloned()
                .unwrap_or_default()
        } else {
            self.token_to_ordinary_checks
                .get(&token_id)
                .cloned()
                .unwrap_or_default()
        }
    }

    /// Set the child context
    pub fn set_child_context(&mut self, context: Context) {
        self.child_context = context;
    }
}

impl Default for TreeWalker {
    fn default() -> Self {
        Self::new()
    }
}
