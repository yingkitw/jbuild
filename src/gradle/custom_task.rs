//! Custom Task Support
//!
//! Implements support for custom Gradle tasks with actions.

use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;
use crate::core::unit_of_work::{UnitOfWork, WorkIdentity, WorkOutput, ExecutionContext};

/// Task action type
#[derive(Debug, Clone)]
pub enum TaskAction {
    /// Copy files from source to destination
    Copy { from: PathBuf, into: PathBuf },
    /// Delete files/directories
    Delete { paths: Vec<PathBuf> },
    /// Execute a command
    Exec { command: String, args: Vec<String>, working_dir: Option<PathBuf> },
    /// Create a directory
    Mkdir { path: PathBuf },
    /// Write content to a file
    WriteFile { path: PathBuf, content: String },
    /// Custom action (for extensibility)
    Custom { name: String, properties: HashMap<String, String> },
}

/// A custom task definition
#[derive(Debug, Clone)]
pub struct CustomTask {
    /// Task name
    pub name: String,
    /// Task type (e.g., "Copy", "Delete", "Exec")
    pub task_type: String,
    /// Task group
    pub group: Option<String>,
    /// Task description
    pub description: Option<String>,
    /// Tasks this depends on
    pub depends_on: Vec<String>,
    /// Tasks that must run after this
    pub finalized_by: Vec<String>,
    /// Actions to execute
    pub actions: Vec<TaskAction>,
    /// Input files (for up-to-date checking)
    pub inputs: Vec<PathBuf>,
    /// Output files (for up-to-date checking)
    pub outputs: Vec<PathBuf>,
    /// Whether this task is enabled
    pub enabled: bool,
    /// Condition for execution
    pub only_if: Option<String>,
}

impl CustomTask {
    pub fn new(name: impl Into<String>, task_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            task_type: task_type.into(),
            group: None,
            description: None,
            depends_on: Vec::new(),
            finalized_by: Vec::new(),
            actions: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            only_if: None,
        }
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn depends_on(mut self, task: impl Into<String>) -> Self {
        self.depends_on.push(task.into());
        self
    }

    pub fn add_action(&mut self, action: TaskAction) {
        self.actions.push(action);
    }

    pub fn with_action(mut self, action: TaskAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<PathBuf>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<PathBuf>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Execute all actions for this task
    pub fn execute(&self, base_dir: &PathBuf) -> Result<()> {
        if !self.enabled {
            tracing::info!("Task {} is disabled, skipping", self.name);
            return Ok(());
        }

        tracing::info!("Executing task: {} ({})", self.name, self.task_type);

        for action in &self.actions {
            self.execute_action(action, base_dir)?;
        }

        Ok(())
    }

    /// Execute a single action
    fn execute_action(&self, action: &TaskAction, base_dir: &PathBuf) -> Result<()> {
        match action {
            TaskAction::Copy { from, into } => {
                let from_path = if from.is_absolute() { from.clone() } else { base_dir.join(from) };
                let into_path = if into.is_absolute() { into.clone() } else { base_dir.join(into) };

                tracing::debug!("Copying from {:?} to {:?}", from_path, into_path);

                if from_path.is_dir() {
                    copy_dir_recursive(&from_path, &into_path)?;
                } else if from_path.is_file() {
                    std::fs::create_dir_all(into_path.parent().unwrap_or(&into_path))?;
                    std::fs::copy(&from_path, &into_path)?;
                }
            }
            TaskAction::Delete { paths } => {
                for path in paths {
                    let full_path = if path.is_absolute() { path.clone() } else { base_dir.join(path) };
                    tracing::debug!("Deleting {:?}", full_path);

                    if full_path.is_dir() {
                        std::fs::remove_dir_all(&full_path).ok();
                    } else if full_path.is_file() {
                        std::fs::remove_file(&full_path).ok();
                    }
                }
            }
            TaskAction::Exec { command, args, working_dir } => {
                let cwd = working_dir.clone().unwrap_or_else(|| base_dir.clone());
                tracing::debug!("Executing: {} {:?} in {:?}", command, args, cwd);

                let status = std::process::Command::new(command)
                    .args(args)
                    .current_dir(&cwd)
                    .status()?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Command failed with exit code: {:?}", status.code()));
                }
            }
            TaskAction::Mkdir { path } => {
                let full_path = if path.is_absolute() { path.clone() } else { base_dir.join(path) };
                tracing::debug!("Creating directory {:?}", full_path);
                std::fs::create_dir_all(&full_path)?;
            }
            TaskAction::WriteFile { path, content } => {
                let full_path = if path.is_absolute() { path.clone() } else { base_dir.join(path) };
                tracing::debug!("Writing file {:?}", full_path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&full_path, content)?;
            }
            TaskAction::Custom { name, properties } => {
                tracing::info!("Custom action: {} with properties: {:?}", name, properties);
                // Custom actions would need to be handled by registered handlers
            }
        }

        Ok(())
    }
}

/// Copy a directory recursively
fn copy_dir_recursive(from: &PathBuf, to: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(to)?;

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        let dest = to.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }

    Ok(())
}

impl UnitOfWork for CustomTask {
    fn identify(&self) -> WorkIdentity {
        WorkIdentity::new(&self.name, &self.task_type)
    }

    fn description(&self) -> String {
        self.description.clone().unwrap_or_else(|| format!("Task: {}", self.name))
    }

    fn execute(&self, context: &ExecutionContext) -> Result<WorkOutput> {
        let start = std::time::Instant::now();

        match self.execute(&context.base_directory) {
            Ok(()) => Ok(WorkOutput::success(self.outputs.clone(), start.elapsed().as_millis() as u64)),
            Err(e) => Ok(WorkOutput::failure(vec![e.to_string()], start.elapsed().as_millis() as u64)),
        }
    }

    fn visit_mutable_inputs(&self, visitor: &mut dyn crate::core::unit_of_work::InputVisitor) {
        for input in &self.inputs {
            visitor.visit_file("input", input);
        }
    }

    fn visit_outputs(&self, visitor: &mut dyn crate::core::unit_of_work::OutputVisitor) {
        for output in &self.outputs {
            if output.is_dir() {
                visitor.visit_directory("output", output);
            } else {
                visitor.visit_file("output", output);
            }
        }
    }
}

/// Task registry for custom tasks
#[derive(Debug, Default)]
pub struct TaskRegistry {
    tasks: HashMap<String, CustomTask>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom task
    pub fn register(&mut self, task: CustomTask) {
        self.tasks.insert(task.name.clone(), task);
    }

    /// Get a task by name
    pub fn get(&self, name: &str) -> Option<&CustomTask> {
        self.tasks.get(name)
    }

    /// Check if a task exists
    pub fn has(&self, name: &str) -> bool {
        self.tasks.contains_key(name)
    }

    /// Get all task names
    pub fn names(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Execute a task by name
    pub fn execute(&self, name: &str, base_dir: &PathBuf) -> Result<()> {
        let task = self.tasks.get(name)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", name))?;
        task.execute(base_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_custom_task_creation() {
        let task = CustomTask::new("myTask", "Copy")
            .with_group("build")
            .with_description("Copies files")
            .depends_on("compileJava");

        assert_eq!(task.name, "myTask");
        assert_eq!(task.task_type, "Copy");
        assert_eq!(task.group, Some("build".to_string()));
        assert_eq!(task.depends_on, vec!["compileJava".to_string()]);
    }

    #[test]
    fn test_mkdir_action() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_mkdir");
        let _ = fs::remove_dir_all(&temp_dir);

        let task = CustomTask::new("createDir", "Mkdir")
            .with_action(TaskAction::Mkdir { path: PathBuf::from("subdir") });

        task.execute(&temp_dir).unwrap();

        assert!(temp_dir.join("subdir").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_write_file_action() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_write");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let task = CustomTask::new("writeFile", "WriteFile")
            .with_action(TaskAction::WriteFile { 
                path: PathBuf::from("test.txt"), 
                content: "Hello, World!".to_string() 
            });

        task.execute(&temp_dir).unwrap();

        let content = fs::read_to_string(temp_dir.join("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_delete_action() {
        let temp_dir = std::env::temp_dir().join("jbuild_test_delete");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("to_delete.txt"), "delete me").unwrap();

        let task = CustomTask::new("deleteFile", "Delete")
            .with_action(TaskAction::Delete { 
                paths: vec![PathBuf::from("to_delete.txt")] 
            });

        task.execute(&temp_dir).unwrap();

        assert!(!temp_dir.join("to_delete.txt").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_task_registry() {
        let mut registry = TaskRegistry::new();
        registry.register(CustomTask::new("task1", "Copy"));
        registry.register(CustomTask::new("task2", "Delete"));

        assert!(registry.has("task1"));
        assert!(registry.has("task2"));
        assert!(!registry.has("task3"));
        assert_eq!(registry.names().len(), 2);
    }

    #[test]
    fn test_disabled_task() {
        let mut task = CustomTask::new("disabled", "Exec");
        task.enabled = false;
        task.add_action(TaskAction::Exec { 
            command: "false".to_string(), 
            args: vec![], 
            working_dir: None 
        });

        // Should succeed because task is disabled
        let result = task.execute(&PathBuf::from("/tmp"));
        assert!(result.is_ok());
    }
}
