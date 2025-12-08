//! AST types for Checkstyle-rs

use std::sync::Arc;

/// Interface for AST nodes
pub trait DetailAst: Send + Sync {
    /// Get the number of child nodes
    fn get_child_count(&self) -> usize;

    /// Get the number of direct child tokens of the specified type
    fn get_child_count_by_type(&self, token_type: i32) -> usize;

    /// Get the parent node as Arc (for Arc-based implementations)
    fn get_parent_arc(&self) -> Option<Arc<dyn DetailAst>>;

    /// Get the text of this AST node
    fn get_text(&self) -> &str;

    /// Get the type of this AST node
    fn get_type(&self) -> i32;

    /// Get the line number (1-based)
    fn get_line_no(&self) -> usize;

    /// Get the column number (1-based)
    fn get_column_no(&self) -> usize;

    /// Get the last child node as Arc
    fn get_last_child_arc(&self) -> Option<Arc<dyn DetailAst>>;

    /// Get the previous sibling as Arc
    fn get_previous_sibling_arc(&self) -> Option<Arc<dyn DetailAst>>;

    /// Find the first child token of the specified type as Arc
    fn find_first_token_arc(&self, token_type: i32) -> Option<Arc<dyn DetailAst>>;

    /// Get the next sibling as Arc
    fn get_next_sibling_arc(&self) -> Option<Arc<dyn DetailAst>>;

    /// Get the first child as Arc
    fn get_first_child_arc(&self) -> Option<Arc<dyn DetailAst>>;

    // Legacy methods for compatibility (may return None for Arc-based implementations)
    /// Get the parent node (legacy - may return None for Arc-based implementations)
    fn get_parent(&self) -> Option<&dyn DetailAst> {
        None
    }

    /// Get the last child node (legacy - may return None for Arc-based implementations)
    fn get_last_child(&self) -> Option<&dyn DetailAst> {
        None
    }

    /// Get the previous sibling (legacy - may return None for Arc-based implementations)
    fn get_previous_sibling(&self) -> Option<&dyn DetailAst> {
        None
    }

    /// Find the first child token of the specified type (legacy - may return None for Arc-based implementations)
    fn find_first_token(&self, token_type: i32) -> Option<&dyn DetailAst> {
        let _ = token_type;
        None
    }

    /// Get the next sibling (legacy - may return None for Arc-based implementations)
    fn get_next_sibling(&self) -> Option<&dyn DetailAst> {
        None
    }

    /// Get the first child (legacy - may return None for Arc-based implementations)
    fn get_first_child(&self) -> Option<&dyn DetailAst> {
        None
    }
}

/// Token types for Java AST
///
/// These values will be aligned with the actual Java lexer token types
/// when the parser is implemented. For now, we define common ones.
pub mod token_types {
    // EOF
    pub const EOF: i32 = -1;

    // Identifiers and literals
    pub const IDENT: i32 = 1;
    pub const LITERAL_VOID: i32 = 2;
    pub const LITERAL_BOOLEAN: i32 = 3;
    pub const LITERAL_BYTE: i32 = 4;
    pub const LITERAL_CHAR: i32 = 5;
    pub const LITERAL_SHORT: i32 = 6;
    pub const LITERAL_INT: i32 = 7;
    pub const LITERAL_LONG: i32 = 8;
    pub const LITERAL_FLOAT: i32 = 9;
    pub const LITERAL_DOUBLE: i32 = 10;
    pub const LITERAL_STRING: i32 = 11;
    pub const LITERAL_NULL: i32 = 12;
    pub const LITERAL_TRUE: i32 = 13;
    pub const LITERAL_FALSE: i32 = 14;
    pub const LITERAL_NEW: i32 = 15;

    // Numeric literals
    pub const NUM_INT: i32 = 16;
    pub const NUM_LONG: i32 = 17;
    pub const NUM_FLOAT: i32 = 18;
    pub const NUM_DOUBLE: i32 = 19;
    pub const CHAR_LITERAL: i32 = 20;
    pub const STRING_LITERAL: i32 = 21;
    pub const LITERAL_THIS: i32 = 22;
    pub const LITERAL_SUPER: i32 = 23;

    // Control flow keywords
    pub const LITERAL_IF: i32 = 24;
    pub const LITERAL_ELSE: i32 = 25;
    pub const LITERAL_WHILE: i32 = 26;
    pub const LITERAL_FOR: i32 = 27;
    pub const LITERAL_DO: i32 = 28;
    pub const LITERAL_SWITCH: i32 = 29;
    pub const LITERAL_CASE: i32 = 30;
    pub const LITERAL_DEFAULT: i32 = 31;
    pub const LITERAL_BREAK: i32 = 32;
    pub const LITERAL_CONTINUE: i32 = 33;
    pub const LITERAL_RETURN: i32 = 34;
    pub const LITERAL_TRY: i32 = 35;
    pub const LITERAL_CATCH: i32 = 36;
    pub const LITERAL_FINALLY: i32 = 37;
    pub const LITERAL_THROW: i32 = 38;
    pub const LITERAL_THROWS: i32 = 39;
    pub const LITERAL_INSTANCEOF: i32 = 40;
    pub const LITERAL_SYNCHRONIZED: i32 = 41;
    pub const LITERAL_ASSERT: i32 = 42;

    // Type and class keywords
    pub const LITERAL_CLASS: i32 = 43;
    pub const LITERAL_INTERFACE: i32 = 44;
    pub const LITERAL_ENUM: i32 = 45;
    pub const LITERAL_EXTENDS: i32 = 46;
    pub const LITERAL_IMPLEMENTS: i32 = 47;
    pub const LITERAL_IMPORT: i32 = 48;
    pub const LITERAL_PACKAGE: i32 = 49;

    // Modifiers
    pub const LITERAL_PUBLIC: i32 = 50;
    pub const LITERAL_PROTECTED: i32 = 51;
    pub const LITERAL_PRIVATE: i32 = 52;
    pub const LITERAL_STATIC: i32 = 53;
    pub const LITERAL_FINAL: i32 = 54;
    pub const LITERAL_ABSTRACT: i32 = 55;
    pub const LITERAL_NATIVE: i32 = 56;
    pub const LITERAL_TRANSIENT: i32 = 57;
    pub const LITERAL_VOLATILE: i32 = 58;
    pub const LITERAL_STRICTFP: i32 = 59;

    // AST node types
    pub const COMPILATION_UNIT: i32 = 100;
    pub const PACKAGE_DEF: i32 = 101;
    pub const IMPORT: i32 = 102;
    pub const STATIC_IMPORT: i32 = 1021; // Approximate value
    pub const CLASS_DEF: i32 = 103;
    pub const INTERFACE_DEF: i32 = 104;
    pub const ENUM_DEF: i32 = 105;
    pub const METHOD_DEF: i32 = 106;
    pub const VARIABLE_DEF: i32 = 107;
    pub const CTOR_DEF: i32 = 108;
    pub const INSTANCE_INIT: i32 = 109;
    pub const STATIC_INIT: i32 = 110;
    pub const TYPE: i32 = 111;
    pub const MODIFIERS: i32 = 112;
    pub const OBJBLOCK: i32 = 113;
    pub const SLIST: i32 = 114;
    pub const EXPR: i32 = 115;
    pub const EMPTY_STAT: i32 = 1448;
    pub const PARAMETERS: i32 = 722;
    pub const FOR_INIT: i32 = 723;

    // Operators
    pub const ASSIGN: i32 = 135;
    pub const PLUS_ASSIGN: i32 = 136;
    pub const MINUS_ASSIGN: i32 = 137;
    pub const STAR_ASSIGN: i32 = 138;
    pub const DIV_ASSIGN: i32 = 139;
    pub const MOD_ASSIGN: i32 = 140;
    pub const BSR_ASSIGN: i32 = 141;
    pub const SR_ASSIGN: i32 = 142;
    pub const SL_ASSIGN: i32 = 143;
    pub const BAND_ASSIGN: i32 = 144;
    pub const BXOR_ASSIGN: i32 = 145;
    pub const BOR_ASSIGN: i32 = 146;
    pub const QUESTION: i32 = 147;
    pub const COLON: i32 = 148;
    pub const COMMA: i32 = 149;
    pub const DOT: i32 = 150;
    pub const ELLIPSIS: i32 = 151;
    pub const SEMI: i32 = 152;
    pub const STAR: i32 = 153;

    // Logical and comparison operators
    pub const LOR: i32 = 154; // ||
    pub const LAND: i32 = 155; // &&
    pub const BOR: i32 = 156; // |
    pub const BXOR: i32 = 157; // ^
    pub const BAND: i32 = 158; // &
    pub const NOT_EQUAL: i32 = 159; // !=
    pub const EQUAL: i32 = 160; // ==
    pub const LT: i32 = 161; // <
    pub const GT: i32 = 162; // >
    pub const LE: i32 = 163; // <=
    pub const GE: i32 = 164; // >=
    pub const LNOT: i32 = 165; // !
    pub const BNOT: i32 = 166; // ~
    pub const INC: i32 = 167; // ++
    pub const DEC: i32 = 168; // --

    // Brackets and parentheses
    pub const LCURLY: i32 = 2241;
    pub const RCURLY: i32 = 2270;
    pub const LPAREN: i32 = 2353;
    pub const RPAREN: i32 = 2385;
    pub const LBRACK: i32 = 1629;
    pub const RBRACK: i32 = 1630;

    // Comments
    pub const SINGLE_LINE_COMMENT: i32 = 200;
    pub const BLOCK_COMMENT_BEGIN: i32 = 201;
    pub const BLOCK_COMMENT_END: i32 = 202;
    pub const JAVADOC_COMMENT: i32 = 203;
}
