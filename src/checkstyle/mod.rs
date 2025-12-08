//! Checkstyle - Java code style checker
//!
//! This module provides Java code style checking capabilities integrated into jbuild.
//! It is a Rust implementation of Checkstyle, using tree-sitter for Java parsing.

pub mod api;
pub mod checks;
pub mod parser;
pub mod runner;
pub mod utils;

pub use api::*;
pub use checks::*;
pub use parser::*;
pub use runner::*;
pub use utils::*;
