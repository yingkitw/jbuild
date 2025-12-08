//! AST implementation for Checkstyle-rs

use crate::checkstyle::api::ast::DetailAst;
use std::sync::{Arc, Mutex};

/// AST node implementation
#[derive(Debug)]
pub struct DetailAstImpl {
    /// Token type
    pub token_type: i32,
    /// Text content
    pub text: String,
    /// Line number (1-based)
    pub line_no: usize,
    /// Column number (1-based)
    pub column_no: usize,
    /// Parent node
    pub parent: Option<Arc<DetailAstImpl>>,
    /// First child (using Mutex for interior mutability during construction)
    pub first_child: Arc<Mutex<Option<Arc<DetailAstImpl>>>>,
    /// Next sibling (using Mutex for interior mutability during construction)
    pub next_sibling: Arc<Mutex<Option<Arc<DetailAstImpl>>>>,
}

impl DetailAstImpl {
    /// Create a new AST node
    pub fn new(token_type: i32, text: String, line_no: usize, column_no: usize) -> Self {
        Self {
            token_type,
            text,
            line_no,
            column_no,
            parent: None,
            first_child: Arc::new(Mutex::new(None)),
            next_sibling: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the first child
    pub fn set_first_child(&self, child: Option<Arc<DetailAstImpl>>) {
        *self.first_child.lock().unwrap() = child;
    }

    /// Set the next sibling
    pub fn set_next_sibling(&self, sibling: Option<Arc<DetailAstImpl>>) {
        *self.next_sibling.lock().unwrap() = sibling;
    }

    /// Get the first child as Arc (for internal use)
    pub fn get_first_child_arc(&self) -> Option<Arc<DetailAstImpl>> {
        self.first_child.lock().unwrap().clone()
    }

    /// Get the next sibling as Arc (for internal use)
    pub fn get_next_sibling_arc(&self) -> Option<Arc<DetailAstImpl>> {
        self.next_sibling.lock().unwrap().clone()
    }
}

impl DetailAst for DetailAstImpl {
    fn get_child_count(&self) -> usize {
        let mut count = 0;
        let first_child_guard = self.first_child.lock().unwrap();
        let mut child_opt = first_child_guard.clone();
        drop(first_child_guard);

        while let Some(child) = child_opt {
            count += 1;
            let next_guard = child.next_sibling.lock().unwrap();
            child_opt = next_guard.clone();
        }
        count
    }

    fn get_child_count_by_type(&self, token_type: i32) -> usize {
        let mut count = 0;
        let first_child_guard = self.first_child.lock().unwrap();
        let mut child_opt = first_child_guard.clone();
        drop(first_child_guard);

        while let Some(child) = child_opt {
            if child.token_type == token_type {
                count += 1;
            }
            let next_guard = child.next_sibling.lock().unwrap();
            child_opt = next_guard.clone();
        }
        count
    }

    fn get_parent_arc(&self) -> Option<Arc<dyn DetailAst>> {
        self.parent
            .as_ref()
            .map(|p| p.clone() as Arc<dyn DetailAst>)
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_type(&self) -> i32 {
        self.token_type
    }

    fn get_line_no(&self) -> usize {
        self.line_no
    }

    fn get_column_no(&self) -> usize {
        self.column_no
    }

    fn get_last_child_arc(&self) -> Option<Arc<dyn DetailAst>> {
        let first_child_guard = self.first_child.lock().unwrap();
        let mut child_opt = first_child_guard.clone();
        drop(first_child_guard);

        let mut last_child: Option<Arc<DetailAstImpl>> = None;
        while let Some(child) = child_opt {
            last_child = Some(child.clone());
            let next_guard = child.next_sibling.lock().unwrap();
            child_opt = next_guard.clone();
        }
        last_child.map(|c| c as Arc<dyn DetailAst>)
    }

    fn get_previous_sibling_arc(&self) -> Option<Arc<dyn DetailAst>> {
        // This requires parent traversal - simplified for now
        // Would need to traverse parent's children to find previous sibling
        None
    }

    fn find_first_token_arc(&self, token_type: i32) -> Option<Arc<dyn DetailAst>> {
        let first_child_guard = self.first_child.lock().unwrap();
        let mut child_opt = first_child_guard.clone();
        drop(first_child_guard);

        while let Some(child) = child_opt {
            if child.token_type == token_type {
                return Some(child as Arc<dyn DetailAst>);
            }
            // Also check children recursively
            if let Some(found) = child.find_first_token_arc(token_type) {
                return Some(found);
            }
            let next_guard = child.next_sibling.lock().unwrap();
            child_opt = next_guard.clone();
        }
        None
    }

    fn get_next_sibling_arc(&self) -> Option<Arc<dyn DetailAst>> {
        let next_sibling_guard = self.next_sibling.lock().unwrap();
        next_sibling_guard
            .as_ref()
            .map(|s| s.clone() as Arc<dyn DetailAst>)
    }

    fn get_first_child_arc(&self) -> Option<Arc<dyn DetailAst>> {
        let first_child_guard = self.first_child.lock().unwrap();
        first_child_guard
            .as_ref()
            .map(|c| c.clone() as Arc<dyn DetailAst>)
    }
}
