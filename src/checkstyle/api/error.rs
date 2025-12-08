//! Error types for Checkstyle-rs

use thiserror::Error;

/// Main error type for Checkstyle operations
#[derive(Error, Debug)]
pub enum CheckstyleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("Check error: {0}")]
    Check(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for Checkstyle operations
pub type CheckstyleResult<T> = Result<T, CheckstyleError>;
