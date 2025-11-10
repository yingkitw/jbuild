use std::collections::HashMap;
use std::path::PathBuf;

use crate::settings::Settings;
use crate::core::project::MavenProject;

/// Maven execution request
#[derive(Debug, Clone)]
pub struct MavenExecutionRequest {
    /// Base directory of the project
    pub base_directory: PathBuf,

    /// Goals to execute
    pub goals: Vec<String>,

    /// Profiles to activate
    pub active_profiles: Vec<String>,

    /// System properties
    pub system_properties: HashMap<String, String>,

    /// User properties
    pub user_properties: HashMap<String, String>,

    /// Whether to show errors
    pub show_errors: bool,

    /// Whether to use reactor
    pub reactor_active: bool,

    /// Settings
    pub settings: Option<Settings>,

    /// POM file path
    pub pom_file: Option<PathBuf>,
}

impl MavenExecutionRequest {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            base_directory,
            goals: Vec::new(),
            active_profiles: Vec::new(),
            system_properties: HashMap::new(),
            user_properties: HashMap::new(),
            show_errors: false,
            reactor_active: true,
            settings: None,
            pom_file: None,
        }
    }

    pub fn with_goals(mut self, goals: Vec<String>) -> Self {
        self.goals = goals;
        self
    }

    pub fn with_pom_file(mut self, pom_file: PathBuf) -> Self {
        self.pom_file = Some(pom_file);
        self
    }
}

/// Maven execution result
#[derive(Debug)]
pub struct MavenExecutionResult {
    /// Projects that were built
    pub projects: Vec<MavenProject>,

    /// Execution exceptions
    pub exceptions: Vec<anyhow::Error>,

    /// Whether the execution was successful
    pub success: bool,
}

impl MavenExecutionResult {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            exceptions: Vec::new(),
            success: true,
        }
    }

    pub fn add_exception(&mut self, error: anyhow::Error) {
        self.exceptions.push(error);
        self.success = false;
    }
}

impl Default for MavenExecutionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Maven execution engine
pub trait Maven {
    /// Execute a Maven build
    fn execute(&mut self, request: MavenExecutionRequest) -> anyhow::Result<MavenExecutionResult>;
}

