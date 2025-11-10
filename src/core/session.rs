use std::collections::HashMap;
use std::path::PathBuf;

use crate::settings::Settings;
use crate::core::project::MavenProject;

/// Maven execution session
#[derive(Debug, Clone)]
pub struct MavenSession {
    /// Current project
    pub current_project: Option<MavenProject>,

    /// All projects in the reactor
    pub projects: Vec<MavenProject>,

    /// Settings
    pub settings: Settings,

    /// System properties
    pub system_properties: HashMap<String, String>,

    /// User properties
    pub user_properties: HashMap<String, String>,

    /// Execution root directory
    pub execution_root: PathBuf,

    /// Local repository path
    pub local_repository: PathBuf,
}

impl MavenSession {
    pub fn new(execution_root: PathBuf, settings: Settings) -> Self {
        let local_repo = settings
            .local_repository
            .as_ref()
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let mut path = PathBuf::from(home);
                path.push(".m2");
                path.push("repository");
                path
            });

        Self {
            current_project: None,
            projects: Vec::new(),
            settings,
            system_properties: HashMap::new(),
            user_properties: HashMap::new(),
            execution_root,
            local_repository: local_repo,
        }
    }

    pub fn with_project(mut self, project: MavenProject) -> Self {
        self.current_project = Some(project.clone());
        self.projects.push(project);
        self
    }
}

