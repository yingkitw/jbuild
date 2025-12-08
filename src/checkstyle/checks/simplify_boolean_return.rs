//! SimplifyBooleanReturn check - detects if boolean return can be simplified

use crate::checkstyle::api::ast::DetailAst;
use crate::checkstyle::api::check::Check;
use crate::checkstyle::api::config::{Configurable, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::checks::base::AbstractCheck;

/// Check for over-complicated boolean return statements
///
/// Detects patterns like:
/// ```java
/// if (valid())
///   return false;
/// else
///   return true;
/// ```
/// which can be simplified to:
/// ```java
/// return !valid();
/// ```
pub struct SimplifyBooleanReturnCheck {
    base: AbstractCheck,
}

impl SimplifyBooleanReturnCheck {
    /// Create a new SimplifyBooleanReturnCheck
    pub fn new() -> Self {
        Self {
            base: AbstractCheck::new("SimplifyBooleanReturn".to_string()),
        }
    }

    /// Check if an AST is a return statement with a boolean literal
    /// or a compound statement that contains only such a return statement
    fn can_return_or_yield_only_boolean_literal(&self, ast: &dyn DetailAst) -> bool {
        // First check if the AST itself is a boolean return
        if let Some(ast_arc) = ast.get_first_child_arc() {
            if self.is_boolean_literal_return_or_yield_statement(&ast_arc) {
                return true;
            }
        }

        // Check if it's a block (SLIST) containing only a boolean return
        if ast.get_type() == crate::checkstyle::api::ast::token_types::SLIST {
            if let Some(first_child) = ast.get_first_child_arc() {
                return self.is_boolean_literal_return_or_yield_statement(&first_child);
            }
        }

        false
    }

    /// Check if an AST is a return or yield statement with a boolean literal
    fn is_boolean_literal_return_or_yield_statement(
        &self,
        ast: &std::sync::Arc<dyn DetailAst>,
    ) -> bool {
        let ast_type = ast.get_type();

        if ast_type != crate::checkstyle::api::ast::token_types::LITERAL_RETURN {
            // TODO: Support LITERAL_YIELD when available
            return false;
        }

        // Get the expression child (skip SEMI if present)
        if let Some(expr) = ast.get_first_child_arc() {
            if expr.get_type() == crate::checkstyle::api::ast::token_types::SEMI {
                return false; // Empty return
            }

            // Check if the expression is a boolean literal
            if let Some(value) = expr.get_first_child_arc() {
                let value_type = value.get_type();
                return value_type == crate::checkstyle::api::ast::token_types::LITERAL_TRUE
                    || value_type == crate::checkstyle::api::ast::token_types::LITERAL_FALSE;
            }
        }

        false
    }
}

impl Default for SimplifyBooleanReturnCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for SimplifyBooleanReturnCheck {
    fn configure(&mut self, config: &crate::checkstyle::api::config::Configuration) -> CheckstyleResult<()> {
        self.base.configure(config)
    }
}

impl Contextualizable for SimplifyBooleanReturnCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.base.contextualize(context)
    }
}

impl Check for SimplifyBooleanReturnCheck {
    fn get_default_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_acceptable_tokens(&self) -> Vec<i32> {
        self.get_required_tokens()
    }

    fn get_required_tokens(&self) -> Vec<i32> {
        vec![crate::checkstyle::api::ast::token_types::LITERAL_IF]
    }

    fn visit_token(&mut self, ast: &dyn DetailAst) -> CheckstyleResult<()> {
        // LITERAL_IF has the following structure:
        // '('
        // condition
        // ')'
        // thenStatement
        // [ LITERAL_ELSE (with the elseStatement as a child) ]

        // Only check if-else statements
        if let Some(else_literal) =
            ast.find_first_token_arc(crate::checkstyle::api::ast::token_types::LITERAL_ELSE)
        {
            // Get first child (should be '(')
            if let Some(lparen) = ast.get_first_child_arc() {
                // Get condition (next sibling after '(')
                if let Some(condition) = lparen.get_next_sibling_arc() {
                    // Get ')' (next sibling after condition)
                    if let Some(rparen) = condition.get_next_sibling_arc() {
                        // Get then statement (next sibling after ')')
                        if let Some(then_statement) = rparen.get_next_sibling_arc() {
                            // Get else statement
                            if let Some(else_statement) = else_literal.get_first_child_arc() {
                                // Convert Arc to &dyn DetailAst for the helper method
                                let then_ref = then_statement.as_ref() as &dyn DetailAst;
                                let else_ref = else_statement.as_ref() as &dyn DetailAst;

                                if self.can_return_or_yield_only_boolean_literal(then_ref)
                                    && self.can_return_or_yield_only_boolean_literal(else_ref)
                                {
                                    self.base.log_ast(
                                        ast,
                                        "simplify.boolReturn".to_string(),
                                        vec![],
                                    );
                                }
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
