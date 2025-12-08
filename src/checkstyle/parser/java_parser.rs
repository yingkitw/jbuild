//! Java parser for Checkstyle-rs

use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use crate::checkstyle::api::file::FileContents;
use crate::checkstyle::parser::ast_impl::DetailAstImpl;
use std::sync::{Arc, Mutex};
use tree_sitter::{Parser, Tree};

/// Java parser options
#[derive(Debug, Clone, Copy)]
pub enum ParserOptions {
    /// Include comments in AST
    WithComments,
    /// Exclude comments from AST
    WithoutComments,
}

/// Java parser
pub struct JavaParser;

impl JavaParser {
    /// Parse a Java source file into an AST
    ///
    /// This implementation uses tree-sitter-java to parse Java source code.
    pub fn parse(contents: &FileContents) -> CheckstyleResult<Arc<DetailAstImpl>> {
        let text = contents.get_text().get_full_text();

        // Initialize tree-sitter parser
        let mut parser = Parser::new();
        let language = tree_sitter_java::language();
        parser
            .set_language(language)
            .map_err(|e| CheckstyleError::Parse(format!("Failed to set Java language: {}", e)))?;

        // Parse the source code
        let tree = parser.parse(text, None).ok_or_else(|| {
            CheckstyleError::Parse("Failed to parse Java source code".to_string())
        })?;

        // Convert tree-sitter tree to our AST
        Self::convert_tree_sitter_tree(&tree, text)
    }

    /// Parse with options
    pub fn parse_with_options(
        contents: &FileContents,
        _options: ParserOptions,
    ) -> CheckstyleResult<Arc<DetailAstImpl>> {
        Self::parse(contents)
    }

    /// Convert tree-sitter tree to DetailAstImpl
    fn convert_tree_sitter_tree(tree: &Tree, source: &str) -> CheckstyleResult<Arc<DetailAstImpl>> {
        let root_node = tree.root_node();

        // Build AST from tree-sitter nodes
        Self::build_ast_node(&root_node, source, None)
    }

    /// Build an AST node from a tree-sitter node
    fn build_ast_node(
        node: &tree_sitter::Node,
        source: &str,
        parent: Option<Arc<DetailAstImpl>>,
    ) -> CheckstyleResult<Arc<DetailAstImpl>> {
        let node_text = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();

        // Map tree-sitter node type to our token type
        let token_type = Self::map_node_type_to_token_type(node.kind());

        let ast_node = Arc::new(DetailAstImpl {
            token_type,
            text: node_text,
            line_no: node.start_position().row + 1, // tree-sitter is 0-based
            column_no: node.start_position().column + 1,
            parent: parent.clone(),
            first_child: Arc::new(Mutex::new(None)),
            next_sibling: Arc::new(Mutex::new(None)),
        });

        // Build children
        let child_count = node.child_count();
        let mut children = Vec::new();

        for i in 0..child_count {
            let child_node = node
                .child(i)
                .ok_or_else(|| CheckstyleError::Parse("Failed to get child node".to_string()))?;

            let child_ast = Self::build_ast_node(&child_node, source, Some(ast_node.clone()))?;
            children.push(child_ast);
        }

        // Link children together
        if !children.is_empty() {
            // Set first child
            let first = children[0].clone();
            ast_node.set_first_child(Some(first.clone()));

            // Link siblings
            for i in 0..children.len() - 1 {
                let current = &children[i];
                let next = children[i + 1].clone();
                current.set_next_sibling(Some(next));
            }
        }

        Ok(ast_node)
    }

    /// Map tree-sitter node type to our token type
    fn map_node_type_to_token_type(node_type: &str) -> i32 {
        // Map tree-sitter node types to Checkstyle token types
        // This mapping is based on tree-sitter-java grammar
        match node_type {
            // Compilation unit
            "program" | "compilation_unit" => crate::checkstyle::api::ast::token_types::COMPILATION_UNIT,

            // Package and imports
            "package_declaration" => crate::checkstyle::api::ast::token_types::PACKAGE_DEF,
            "import_declaration" => crate::checkstyle::api::ast::token_types::IMPORT,

            // Type declarations
            "class_declaration" => crate::checkstyle::api::ast::token_types::CLASS_DEF,
            "interface_declaration" => crate::checkstyle::api::ast::token_types::INTERFACE_DEF,
            "enum_declaration" => crate::checkstyle::api::ast::token_types::ENUM_DEF,
            "class_body" | "interface_body" | "enum_body" => crate::checkstyle::api::ast::token_types::OBJBLOCK,

            // Members
            "method_declaration" => crate::checkstyle::api::ast::token_types::METHOD_DEF,
            "constructor_declaration" => crate::checkstyle::api::ast::token_types::CTOR_DEF,
            "field_declaration" | "variable_declarator" => {
                crate::checkstyle::api::ast::token_types::VARIABLE_DEF
            }
            "block" => crate::checkstyle::api::ast::token_types::SLIST,

            // Statements
            "if_statement" => crate::checkstyle::api::ast::token_types::LITERAL_IF,
            "while_statement" => crate::checkstyle::api::ast::token_types::LITERAL_WHILE,
            "for_statement" => crate::checkstyle::api::ast::token_types::LITERAL_FOR,
            "do_statement" => crate::checkstyle::api::ast::token_types::LITERAL_DO,
            "switch_statement" => crate::checkstyle::api::ast::token_types::LITERAL_SWITCH,
            "switch_expression" => crate::checkstyle::api::ast::token_types::LITERAL_SWITCH,
            "return_statement" => crate::checkstyle::api::ast::token_types::LITERAL_RETURN,
            "break_statement" => crate::checkstyle::api::ast::token_types::LITERAL_BREAK,
            "continue_statement" => crate::checkstyle::api::ast::token_types::LITERAL_CONTINUE,
            "throw_statement" => crate::checkstyle::api::ast::token_types::LITERAL_THROW,
            "try_statement" => crate::checkstyle::api::ast::token_types::LITERAL_TRY,
            "catch_clause" => crate::checkstyle::api::ast::token_types::LITERAL_CATCH,
            "finally_clause" => crate::checkstyle::api::ast::token_types::LITERAL_FINALLY,
            "assert_statement" => crate::checkstyle::api::ast::token_types::LITERAL_ASSERT,
            "expression_statement" => crate::checkstyle::api::ast::token_types::EXPR,
            "empty_statement" => 1448, // EMPTY_STAT

            // Expressions
            "expression" => crate::checkstyle::api::ast::token_types::EXPR,
            "assignment_expression" => crate::checkstyle::api::ast::token_types::ASSIGN,
            "binary_expression" => {
                // Try to determine operator type from node text or children
                // For now, return EXPR - could be enhanced to check operator
                crate::checkstyle::api::ast::token_types::EXPR
            }
            "unary_expression" => crate::checkstyle::api::ast::token_types::EXPR,
            "method_invocation" => 959, // METHOD_CALL
            "object_creation_expression" => crate::checkstyle::api::ast::token_types::LITERAL_NEW,
            "array_creation_expression" => crate::checkstyle::api::ast::token_types::LITERAL_NEW,
            "ternary_expression" => crate::checkstyle::api::ast::token_types::QUESTION,
            "instanceof_expression" => crate::checkstyle::api::ast::token_types::LITERAL_INSTANCEOF,

            // Literals and identifiers
            "identifier" => crate::checkstyle::api::ast::token_types::IDENT,
            "string_literal" => crate::checkstyle::api::ast::token_types::STRING_LITERAL,
            "integer_literal" => crate::checkstyle::api::ast::token_types::NUM_INT,
            "boolean_literal" => crate::checkstyle::api::ast::token_types::LITERAL_BOOLEAN,
            "null_literal" => crate::checkstyle::api::ast::token_types::LITERAL_NULL,
            "character_literal" => crate::checkstyle::api::ast::token_types::CHAR_LITERAL,
            "floating_point_literal" => {
                // Try to determine if it's float or double based on suffix
                // For now, default to NUM_DOUBLE - could be enhanced
                crate::checkstyle::api::ast::token_types::NUM_DOUBLE
            }

            // Type keywords
            "void_type" => crate::checkstyle::api::ast::token_types::LITERAL_VOID,
            "boolean_type" => crate::checkstyle::api::ast::token_types::LITERAL_BOOLEAN,
            "integral_type" => crate::checkstyle::api::ast::token_types::LITERAL_INT,
            "floating_point_type" => crate::checkstyle::api::ast::token_types::LITERAL_DOUBLE,
            "type_identifier" => crate::checkstyle::api::ast::token_types::TYPE,

            // Modifiers
            "public" => crate::checkstyle::api::ast::token_types::LITERAL_PUBLIC,
            "protected" => crate::checkstyle::api::ast::token_types::LITERAL_PROTECTED,
            "private" => crate::checkstyle::api::ast::token_types::LITERAL_PRIVATE,
            "static" => crate::checkstyle::api::ast::token_types::LITERAL_STATIC,
            "final" => crate::checkstyle::api::ast::token_types::LITERAL_FINAL,
            "abstract" => crate::checkstyle::api::ast::token_types::LITERAL_ABSTRACT,
            "native" => crate::checkstyle::api::ast::token_types::LITERAL_NATIVE,
            "transient" => crate::checkstyle::api::ast::token_types::LITERAL_TRANSIENT,
            "volatile" => crate::checkstyle::api::ast::token_types::LITERAL_VOLATILE,
            "synchronized" => crate::checkstyle::api::ast::token_types::LITERAL_SYNCHRONIZED,
            "strictfp" => crate::checkstyle::api::ast::token_types::LITERAL_STRICTFP,

            // Keywords
            "class" => crate::checkstyle::api::ast::token_types::LITERAL_CLASS,
            "interface" => crate::checkstyle::api::ast::token_types::LITERAL_INTERFACE,
            "enum" => crate::checkstyle::api::ast::token_types::LITERAL_ENUM,
            "extends" => crate::checkstyle::api::ast::token_types::LITERAL_EXTENDS,
            "implements" => crate::checkstyle::api::ast::token_types::LITERAL_IMPLEMENTS,
            "import" => crate::checkstyle::api::ast::token_types::LITERAL_IMPORT,
            "package" => crate::checkstyle::api::ast::token_types::LITERAL_PACKAGE,
            "this" => crate::checkstyle::api::ast::token_types::LITERAL_THIS,
            "super" => crate::checkstyle::api::ast::token_types::LITERAL_SUPER,
            "new" => crate::checkstyle::api::ast::token_types::LITERAL_NEW,
            "if" => crate::checkstyle::api::ast::token_types::LITERAL_IF,
            "else" => crate::checkstyle::api::ast::token_types::LITERAL_ELSE,
            "while" => crate::checkstyle::api::ast::token_types::LITERAL_WHILE,
            "for" => crate::checkstyle::api::ast::token_types::LITERAL_FOR,
            "do" => crate::checkstyle::api::ast::token_types::LITERAL_DO,
            "switch" => crate::checkstyle::api::ast::token_types::LITERAL_SWITCH,
            "case" => crate::checkstyle::api::ast::token_types::LITERAL_CASE,
            "default" => crate::checkstyle::api::ast::token_types::LITERAL_DEFAULT,
            "break" => crate::checkstyle::api::ast::token_types::LITERAL_BREAK,
            "continue" => crate::checkstyle::api::ast::token_types::LITERAL_CONTINUE,
            "return" => crate::checkstyle::api::ast::token_types::LITERAL_RETURN,
            "try" => crate::checkstyle::api::ast::token_types::LITERAL_TRY,
            "catch" => crate::checkstyle::api::ast::token_types::LITERAL_CATCH,
            "finally" => crate::checkstyle::api::ast::token_types::LITERAL_FINALLY,
            "throw" => crate::checkstyle::api::ast::token_types::LITERAL_THROW,
            "throws" => crate::checkstyle::api::ast::token_types::LITERAL_THROWS,
            "assert" => crate::checkstyle::api::ast::token_types::LITERAL_ASSERT,
            "instanceof" => crate::checkstyle::api::ast::token_types::LITERAL_INSTANCEOF,

            // Operators and punctuation
            "=" => crate::checkstyle::api::ast::token_types::ASSIGN,
            "+=" => crate::checkstyle::api::ast::token_types::PLUS_ASSIGN,
            "-=" => crate::checkstyle::api::ast::token_types::MINUS_ASSIGN,
            "*=" => crate::checkstyle::api::ast::token_types::STAR_ASSIGN,
            "/=" => crate::checkstyle::api::ast::token_types::DIV_ASSIGN,
            "%=" => crate::checkstyle::api::ast::token_types::MOD_ASSIGN,
            "||" => crate::checkstyle::api::ast::token_types::LOR,
            "&&" => crate::checkstyle::api::ast::token_types::LAND,
            "|" => crate::checkstyle::api::ast::token_types::BOR,
            "^" => crate::checkstyle::api::ast::token_types::BXOR,
            "&" => crate::checkstyle::api::ast::token_types::BAND,
            "!=" => crate::checkstyle::api::ast::token_types::NOT_EQUAL,
            "==" => crate::checkstyle::api::ast::token_types::EQUAL,
            "<" => crate::checkstyle::api::ast::token_types::LT,
            ">" => crate::checkstyle::api::ast::token_types::GT,
            "<=" => crate::checkstyle::api::ast::token_types::LE,
            ">=" => crate::checkstyle::api::ast::token_types::GE,
            "!" => crate::checkstyle::api::ast::token_types::LNOT,
            "~" => crate::checkstyle::api::ast::token_types::BNOT,
            "++" => crate::checkstyle::api::ast::token_types::INC,
            "--" => crate::checkstyle::api::ast::token_types::DEC,
            "?" => crate::checkstyle::api::ast::token_types::QUESTION,
            ":" => crate::checkstyle::api::ast::token_types::COLON,
            "," => crate::checkstyle::api::ast::token_types::COMMA,
            "." => crate::checkstyle::api::ast::token_types::DOT,
            ";" => crate::checkstyle::api::ast::token_types::SEMI,
            "*" => crate::checkstyle::api::ast::token_types::STAR,
            "{" => 2241, // LCURLY
            "}" => 2270, // RCURLY
            "(" => 2353, // LPAREN
            ")" => 2385, // RPAREN
            "[" => 1629, // LBRACK
            "]" => crate::checkstyle::api::ast::token_types::RBRACK,

            // Comments
            "line_comment" => crate::checkstyle::api::ast::token_types::SINGLE_LINE_COMMENT,
            "block_comment" => crate::checkstyle::api::ast::token_types::BLOCK_COMMENT_BEGIN,

            // Default for unknown types
            _ => 0,
        }
    }
}
