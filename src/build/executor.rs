//! Build executor abstraction
//! 
//! Defines the trait for executing builds across different build systems.

use std::path::PathBuf;
use anyhow::Result;

/// Generic execution request
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    /// Base directory of the project
    pub base_directory: PathBuf,
    /// Goals/tasks to execute
    pub goals: Vec<String>,
    /// System properties
    pub system_properties: std::collections::HashMap<String, String>,
    /// Whether to show errors
    pub show_errors: bool,
    /// Whether to use offline mode
    pub offline: bool,
}

/// Generic execution result
#[derive(Debug)]
pub struct ExecutionResult {
    /// Whether the build succeeded
    pub success: bool,
    /// Error messages if any
    pub errors: Vec<String>,
}

/// Trait for build system executors
pub trait BuildExecutor: Send + Sync {
    /// Execute a build request
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult>;
    
    /// Get the build system this executor handles
    fn build_system(&self) -> crate::build::BuildSystem;
}

