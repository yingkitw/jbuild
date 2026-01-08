//! Gradle core execution engine
//! 
//! Implements task execution and build lifecycle for Gradle projects.

pub mod tasks;
pub mod dependency_resolver;
pub mod multi_project;

use std::path::PathBuf;
use anyhow::Result;
use crate::build::{BuildExecutor, BuildSystem, ExecutionRequest, ExecutionResult};
use crate::gradle::model::GradleProject;

pub use multi_project::{load_project, load_settings, load_multi_project, is_multi_project};
pub use dependency_resolver::resolve_dependency;

/// Gradle executor implementation
pub struct GradleExecutor;

impl Default for GradleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl GradleExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Load a Gradle project from a directory
    pub fn load_project(&self, base_dir: &PathBuf) -> Result<GradleProject> {
        load_project(base_dir)
    }

    /// Execute a task
    fn execute_task(&self, project: &GradleProject, task_name: &str) -> Result<()> {
        let task = project.find_task(task_name)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {task_name}"))?;

        // Execute task dependencies first
        for dep_name in &task.depends_on {
            self.execute_task(project, dep_name)?;
        }

        // Execute the task itself
        match task.name.as_str() {
            "clean" => tasks::execute_clean(project)?,
            "compileJava" => tasks::execute_compile_java(project, resolve_dependency)?,
            "test" => tasks::execute_test(project)?,
            "jar" => tasks::execute_jar(project)?,
            "build" => tasks::execute_build(project, resolve_dependency)?,
            _ => {
                tracing::info!("Executing task: {}", task_name);
            }
        }

        Ok(())
    }

    /// Load settings for a multi-project build
    pub fn load_settings(&self, base_dir: &PathBuf) -> Result<Option<crate::gradle::settings::GradleSettings>> {
        load_settings(base_dir)
    }

    /// Load all projects in a multi-project build
    pub fn load_multi_project(&self, base_dir: &PathBuf) -> Result<Vec<GradleProject>> {
        load_multi_project(base_dir)
    }

    /// Execute tasks on all projects in a multi-project build
    fn execute_multi_project(&self, base_dir: &PathBuf, tasks_to_run: &[String]) -> Result<ExecutionResult> {
        multi_project::execute_multi_project(base_dir, tasks_to_run, |project, task| {
            self.execute_task(project, task)
        })
    }
}

impl BuildExecutor for GradleExecutor {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        // Check if this is a multi-project build
        if is_multi_project(&request.base_directory) {
            return self.execute_multi_project(&request.base_directory, &request.goals);
        }

        // Single project build
        let project = self.load_project(&request.base_directory)?;

        let mut errors = Vec::new();
        let mut success = true;

        for goal in &request.goals {
            match self.execute_task(&project, goal) {
                Ok(()) => {
                    tracing::info!("Successfully executed task: {}", goal);
                }
                Err(e) => {
                    let error_msg = format!("Failed to execute task '{goal}': {e}");
                    errors.push(error_msg.clone());
                    tracing::error!("{}", error_msg);
                    success = false;
                    
                    if request.show_errors {
                        eprintln!("[ERROR] {error_msg}");
                    }
                }
            }
        }

        Ok(ExecutionResult { success, errors })
    }

    fn build_system(&self) -> BuildSystem {
        BuildSystem::Gradle
    }
}
