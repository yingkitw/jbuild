//! Maven Reactor Build
//!
//! Implements Maven's reactor build functionality for multi-module projects.
//! Based on Maven's ReactorBuildStatus, ProjectBuildList, and related classes.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use anyhow::Result;

/// Build status for a project in the reactor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBuildStatus {
    /// Not yet started
    Pending,
    /// Currently building
    Building,
    /// Build succeeded
    Success,
    /// Build failed
    Failed,
    /// Skipped due to dependency failure
    Skipped,
}

/// A project in the reactor build
#[derive(Debug, Clone)]
pub struct ReactorProject {
    /// Project identifier (groupId:artifactId)
    pub id: String,
    /// Group ID
    pub group_id: String,
    /// Artifact ID
    pub artifact_id: String,
    /// Version
    pub version: String,
    /// Project directory
    pub base_dir: PathBuf,
    /// Dependencies on other reactor projects
    pub reactor_dependencies: Vec<String>,
    /// Build status
    pub status: ProjectBuildStatus,
    /// Build duration in milliseconds
    pub build_time_ms: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ReactorProject {
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
        base_dir: PathBuf,
    ) -> Self {
        let group_id = group_id.into();
        let artifact_id = artifact_id.into();
        Self {
            id: format!("{}:{}", group_id, artifact_id),
            group_id,
            artifact_id: artifact_id.clone(),
            version: version.into(),
            base_dir,
            reactor_dependencies: Vec::new(),
            status: ProjectBuildStatus::Pending,
            build_time_ms: None,
            error: None,
        }
    }

    pub fn add_dependency(&mut self, project_id: impl Into<String>) {
        self.reactor_dependencies.push(project_id.into());
    }

    pub fn mark_building(&mut self) {
        self.status = ProjectBuildStatus::Building;
    }

    pub fn mark_success(&mut self, build_time_ms: u64) {
        self.status = ProjectBuildStatus::Success;
        self.build_time_ms = Some(build_time_ms);
    }

    pub fn mark_failed(&mut self, error: impl Into<String>, build_time_ms: u64) {
        self.status = ProjectBuildStatus::Failed;
        self.error = Some(error.into());
        self.build_time_ms = Some(build_time_ms);
    }

    pub fn mark_skipped(&mut self) {
        self.status = ProjectBuildStatus::Skipped;
    }
}

/// Reactor build status tracking
#[derive(Debug, Default)]
pub struct ReactorBuildStatus {
    /// All projects in the reactor
    projects: Vec<ReactorProject>,
    /// Project ID to index mapping
    project_index: HashMap<String, usize>,
    /// Whether to fail fast on first error
    fail_fast: bool,
    /// Whether the build has been halted
    halted: bool,
}

impl ReactorBuildStatus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Add a project to the reactor
    pub fn add_project(&mut self, project: ReactorProject) {
        let id = project.id.clone();
        let index = self.projects.len();
        self.projects.push(project);
        self.project_index.insert(id, index);
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Option<&ReactorProject> {
        self.project_index.get(id).map(|&i| &self.projects[i])
    }

    /// Get a mutable project by ID
    pub fn get_project_mut(&mut self, id: &str) -> Option<&mut ReactorProject> {
        self.project_index.get(id).map(|&i| &mut self.projects[i])
    }

    /// Get all projects
    pub fn projects(&self) -> &[ReactorProject] {
        &self.projects
    }

    /// Get projects in build order (topological sort)
    pub fn build_order(&self) -> Result<Vec<&ReactorProject>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for project in &self.projects {
            if !visited.contains(&project.id) {
                self.visit(&project.id, &mut visited, &mut temp_visited, &mut result)?;
            }
        }

        Ok(result)
    }

    fn visit<'a>(
        &'a self,
        id: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<&'a ReactorProject>,
    ) -> Result<()> {
        if temp_visited.contains(id) {
            return Err(anyhow::anyhow!("Circular dependency detected: {}", id));
        }
        if visited.contains(id) {
            return Ok(());
        }

        temp_visited.insert(id.to_string());

        if let Some(project) = self.get_project(id) {
            for dep_id in &project.reactor_dependencies {
                self.visit(dep_id, visited, temp_visited, result)?;
            }
            result.push(project);
        }

        temp_visited.remove(id);
        visited.insert(id.to_string());

        Ok(())
    }

    /// Check if all dependencies of a project have succeeded
    pub fn can_build(&self, project_id: &str) -> bool {
        if let Some(project) = self.get_project(project_id) {
            for dep_id in &project.reactor_dependencies {
                if let Some(dep) = self.get_project(dep_id) {
                    if dep.status != ProjectBuildStatus::Success {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Check if the build should continue
    pub fn should_continue(&self) -> bool {
        if self.halted {
            return false;
        }

        if self.fail_fast {
            // Check if any project has failed
            !self.projects.iter().any(|p| p.status == ProjectBuildStatus::Failed)
        } else {
            // Continue if there are pending projects that can be built
            self.projects.iter().any(|p| {
                p.status == ProjectBuildStatus::Pending && self.can_build(&p.id)
            })
        }
    }

    /// Halt the build
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Get summary statistics
    pub fn summary(&self) -> ReactorSummary {
        let mut summary = ReactorSummary::default();
        
        for project in &self.projects {
            match project.status {
                ProjectBuildStatus::Success => {
                    summary.success += 1;
                    if let Some(time) = project.build_time_ms {
                        summary.total_time_ms += time;
                    }
                }
                ProjectBuildStatus::Failed => summary.failed += 1,
                ProjectBuildStatus::Skipped => summary.skipped += 1,
                _ => {}
            }
        }

        summary.total = self.projects.len();
        summary
    }

    /// Skip all projects that depend on failed projects
    pub fn skip_downstream(&mut self) {
        let failed_ids: HashSet<String> = self.projects
            .iter()
            .filter(|p| p.status == ProjectBuildStatus::Failed)
            .map(|p| p.id.clone())
            .collect();

        // Find all projects that transitively depend on failed projects
        let mut to_skip = HashSet::new();
        for project in &self.projects {
            if project.status == ProjectBuildStatus::Pending {
                if self.depends_on_any(&project.id, &failed_ids) {
                    to_skip.insert(project.id.clone());
                }
            }
        }

        // Mark them as skipped
        for id in to_skip {
            if let Some(project) = self.get_project_mut(&id) {
                project.mark_skipped();
            }
        }
    }

    fn depends_on_any(&self, project_id: &str, targets: &HashSet<String>) -> bool {
        if let Some(project) = self.get_project(project_id) {
            for dep_id in &project.reactor_dependencies {
                if targets.contains(dep_id) {
                    return true;
                }
                if self.depends_on_any(dep_id, targets) {
                    return true;
                }
            }
        }
        false
    }
}

/// Summary of reactor build
#[derive(Debug, Default)]
pub struct ReactorSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_time_ms: u64,
}

impl ReactorSummary {
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    pub fn format_time(&self) -> String {
        let seconds = self.total_time_ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        
        if minutes > 0 {
            format!("{}:{:02} min", minutes, remaining_seconds)
        } else {
            format!("{}.{:03} s", seconds, self.total_time_ms % 1000)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactor_project() {
        let project = ReactorProject::new("com.example", "my-app", "1.0.0", PathBuf::from("/project"));
        
        assert_eq!(project.id, "com.example:my-app");
        assert_eq!(project.status, ProjectBuildStatus::Pending);
    }

    #[test]
    fn test_project_status_transitions() {
        let mut project = ReactorProject::new("g", "a", "1.0", PathBuf::from("/"));
        
        project.mark_building();
        assert_eq!(project.status, ProjectBuildStatus::Building);
        
        project.mark_success(1000);
        assert_eq!(project.status, ProjectBuildStatus::Success);
        assert_eq!(project.build_time_ms, Some(1000));
    }

    #[test]
    fn test_reactor_build_order() {
        let mut reactor = ReactorBuildStatus::new();
        
        let mut core = ReactorProject::new("g", "core", "1.0", PathBuf::from("/core"));
        let mut api = ReactorProject::new("g", "api", "1.0", PathBuf::from("/api"));
        api.add_dependency("g:core");
        let mut app = ReactorProject::new("g", "app", "1.0", PathBuf::from("/app"));
        app.add_dependency("g:api");
        
        reactor.add_project(app);
        reactor.add_project(api);
        reactor.add_project(core);
        
        let order = reactor.build_order().unwrap();
        
        // core should come before api, api before app
        let core_pos = order.iter().position(|p| p.artifact_id == "core").unwrap();
        let api_pos = order.iter().position(|p| p.artifact_id == "api").unwrap();
        let app_pos = order.iter().position(|p| p.artifact_id == "app").unwrap();
        
        assert!(core_pos < api_pos);
        assert!(api_pos < app_pos);
    }

    #[test]
    fn test_can_build() {
        let mut reactor = ReactorBuildStatus::new();
        
        let mut core = ReactorProject::new("g", "core", "1.0", PathBuf::from("/core"));
        core.status = ProjectBuildStatus::Success;
        
        let mut api = ReactorProject::new("g", "api", "1.0", PathBuf::from("/api"));
        api.add_dependency("g:core");
        
        reactor.add_project(core);
        reactor.add_project(api);
        
        assert!(reactor.can_build("g:api"));
    }

    #[test]
    fn test_fail_fast() {
        let mut reactor = ReactorBuildStatus::new().with_fail_fast(true);
        
        let mut project = ReactorProject::new("g", "a", "1.0", PathBuf::from("/"));
        project.mark_failed("error", 100);
        reactor.add_project(project);
        
        assert!(!reactor.should_continue());
    }

    #[test]
    fn test_summary() {
        let mut reactor = ReactorBuildStatus::new();
        
        let mut p1 = ReactorProject::new("g", "a", "1.0", PathBuf::from("/a"));
        p1.mark_success(1000);
        
        let mut p2 = ReactorProject::new("g", "b", "1.0", PathBuf::from("/b"));
        p2.mark_failed("error", 500);
        
        reactor.add_project(p1);
        reactor.add_project(p2);
        
        let summary = reactor.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.success, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.is_success());
    }
}
