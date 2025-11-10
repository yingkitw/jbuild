use std::collections::HashMap;

/// Maven Mojo (Maven Old Java Object) - a plugin goal
pub trait Mojo {
    /// Execute the mojo
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Mojo execution context
#[derive(Debug, Clone)]
pub struct MojoExecutionContext {
    pub parameters: HashMap<String, String>,
    pub project: Option<MojoProject>,
}

#[derive(Debug, Clone)]
pub struct MojoProject {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub packaging: String,
}

