//! MultipleVariableDeclarations check - detects multiple variable declarations per statement

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::checks::base::AbstractCheck;
use crate::checkstyle::utils::ast_util::{are_on_same_line, get_first_node, get_last_node};

/// Check that each variable declaration is in its own statement and on its own line
///
/// Detects patterns like:
/// ```java
/// int a, b;  // Multiple variables in one statement
/// int x; int y;  // Multiple statements on one line
/// ```
pub struct MultipleVariableDeclarationsCheck {
    base: AbstractCheck,
}

impl MultipleVariableDeclarationsCheck {
    /// Create a new MultipleVariableDeclarationsCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("MultipleVariableDeclarations".to_string()),
        }
    }
}

impl Default for MultipleVariableDeclarationsCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for MultipleVariableDeclarationsCheck {
    fn configure(&mut self, config: &crate::checkstyle::api::config::Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for MultipleVariableDeclarationsCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for MultipleVariableDeclarationsCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::VARIABLE_DEF]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // Get next sibling as Arc
        if let Some(next_node_arc) = ast.get_next_sibling_arc() {
            let next_type = next_node_arc.get_type();
            let is_comma_separated = next_type == crate::checkstyle::api::ast::token_types::COMMA;

            // Skip comma or semicolon to get to the next variable
            let next_var_node =
                if is_comma_separated || next_type == crate::checkstyle::api::ast::token_types::SEMI {
                    next_node_arc.get_next_sibling_arc()
                } else {
                    Some(next_node_arc)
                };

            // Check if next node is also a variable declaration
            if let Some(next_var) = next_var_node {
                if next_var.get_type() == crate::checkstyle::api::ast::token_types::VARIABLE_DEF {
                    if is_comma_separated {
                        // Check if parent is FOR_INIT (allowed in for loops)
                        let parent_type = ast.get_parent_arc().map(|p| p.get_type()).unwrap_or(0);

                        if parent_type != crate::checkstyle::api::ast::token_types::FOR_INIT {
                            self.base.log_ast(
                                ast,
                                "multiple.variable.declarations.comma".to_string(),
                                vec![],
                            );
                        }
                    } else {
                        // Check if on same line
                        if let Some(ast_first_child) = ast.get_first_child_arc() {
                            let next_first = get_first_node(&next_var);

                            // Get last node of current variable
                            let ast_last = get_last_node(&ast_first_child);

                            if are_on_same_line(&next_first, &ast_last) {
                                self.base.log_ast(
                                    ast,
                                    "multiple.variable.declarations".to_string(),
                                    vec![],
                                );
                            }
                        } else {
                            // Fallback: check line numbers directly
                            if ast.get_line_no() == next_var.get_line_no() {
                                self.base.log_ast(
                                    ast,
                                    "multiple.variable.declarations".to_string(),
                                    vec![],
                                );
                            }
                        }
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
