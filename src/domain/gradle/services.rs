//! Domain services for Gradle context

use super::aggregates::{GradleProject, GradleTask};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

/// Gradle task execution service
/// Orchestrates the execution of Gradle tasks with dependency resolution
pub struct TaskExecutor;

impl TaskExecutor {
    /// Executes a task and all its dependencies
    pub fn execute_task(project: &GradleProject, task_name: &str) -> Result<TaskExecutionPlan> {
        // Get the task
        let _task = project
            .tasks()
            .get(task_name)
            .ok_or_else(|| anyhow!("Task {task_name} not found"))?;

        // Build execution order using topological sort
        let execution_order = Self::resolve_task_dependencies(project, task_name)?;

        let mut plan = TaskExecutionPlan::new();
        for task_name in execution_order {
            if let Some(task) = project.tasks().get(&task_name) {
                plan.add_task(task.clone());
            }
        }

        Ok(plan)
    }

    /// Resolves task dependencies using topological sort
    fn resolve_task_dependencies(project: &GradleProject, task_name: &str) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        Self::visit_task(
            project,
            task_name,
            &mut visited,
            &mut temp_mark,
            &mut result,
        )?;

        Ok(result)
    }

    fn visit_task(
        project: &GradleProject,
        task_name: &str,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<()> {
        if visited.contains(task_name) {
            return Ok(());
        }

        if temp_mark.contains(task_name) {
            return Err(anyhow!("Circular task dependency detected: {task_name}"));
        }

        temp_mark.insert(task_name.to_string());

        // Visit dependencies first
        if let Some(task) = project.tasks().get(task_name) {
            for dep in task.dependencies() {
                Self::visit_task(project, dep, visited, temp_mark, result)?;
            }
        }

        temp_mark.remove(task_name);
        visited.insert(task_name.to_string());
        result.push(task_name.to_string());

        Ok(())
    }

    /// Executes multiple tasks in parallel where possible
    pub fn execute_tasks_parallel(
        project: &GradleProject,
        task_names: &[String],
    ) -> Result<ParallelExecutionPlan> {
        let mut plan = ParallelExecutionPlan::new();

        // Build dependency graph for all tasks
        let mut all_tasks = HashSet::new();
        for task_name in task_names {
            let deps = Self::resolve_task_dependencies(project, task_name)?;
            all_tasks.extend(deps);
        }

        // Group tasks by execution level (tasks with no dependencies first)
        let levels = Self::compute_execution_levels(project, &all_tasks)?;

        for level in levels {
            plan.add_level(level);
        }

        Ok(plan)
    }

    fn compute_execution_levels(
        project: &GradleProject,
        tasks: &HashSet<String>,
    ) -> Result<Vec<Vec<String>>> {
        let mut levels = Vec::new();
        let mut remaining: HashSet<_> = tasks.iter().cloned().collect();
        let mut completed = HashSet::new();

        while !remaining.is_empty() {
            let mut current_level = Vec::new();

            // Find tasks whose dependencies are all completed
            for task_name in &remaining {
                if let Some(task) = project.tasks().get(task_name) {
                    let deps_completed = task
                        .dependencies()
                        .iter()
                        .all(|dep| completed.contains(dep) || !tasks.contains(dep));

                    if deps_completed {
                        current_level.push(task_name.clone());
                    }
                }
            }

            if current_level.is_empty() {
                return Err(anyhow!("Circular dependency detected in task graph"));
            }

            // Remove completed tasks from remaining
            for task_name in &current_level {
                remaining.remove(task_name);
                completed.insert(task_name.clone());
            }

            levels.push(current_level);
        }

        Ok(levels)
    }
}

/// Task execution plan with ordered tasks
#[derive(Debug, Clone)]
pub struct TaskExecutionPlan {
    tasks: Vec<GradleTask>,
}

impl TaskExecutionPlan {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: GradleTask) {
        self.tasks.push(task);
    }

    pub fn tasks(&self) -> &[GradleTask] {
        &self.tasks
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for TaskExecutionPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel execution plan with tasks grouped by execution level
#[derive(Debug, Clone)]
pub struct ParallelExecutionPlan {
    levels: Vec<Vec<String>>,
}

impl ParallelExecutionPlan {
    pub fn new() -> Self {
        Self { levels: Vec::new() }
    }

    pub fn add_level(&mut self, level: Vec<String>) {
        self.levels.push(level);
    }

    pub fn levels(&self) -> &[Vec<String>] {
        &self.levels
    }

    pub fn total_tasks(&self) -> usize {
        self.levels.iter().map(|l| l.len()).sum()
    }
}

impl Default for ParallelExecutionPlan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_single_task() {
        let mut project = GradleProject::new("test", "com.example", "1.0.0", "/tmp/test").unwrap();

        let task = GradleTask::new("compile", "JavaCompile");
        project.register_task(task).unwrap();

        let plan = TaskExecutor::execute_task(&project, "compile");
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn test_execute_task_with_dependencies() {
        let mut project = GradleProject::new("test", "com.example", "1.0.0", "/tmp/test").unwrap();

        let compile = GradleTask::new("compile", "JavaCompile");
        let mut test = GradleTask::new("test", "Test");
        test.add_dependency("compile".to_string()).unwrap();

        project.register_task(compile).unwrap();
        project.register_task(test).unwrap();

        let plan = TaskExecutor::execute_task(&project, "test");
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.len(), 2);

        // Compile should come before test
        assert_eq!(plan.tasks()[0].name(), "compile");
        assert_eq!(plan.tasks()[1].name(), "test");
    }

    #[test]
    fn test_execute_task_not_found() {
        let project = GradleProject::new("test", "com.example", "1.0.0", "/tmp/test").unwrap();

        let result = TaskExecutor::execute_task(&project, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_parallel_execution_plan() {
        let mut project = GradleProject::new("test", "com.example", "1.0.0", "/tmp/test").unwrap();

        // Create a diamond dependency graph:
        //     compile
        //    /       \
        //  test1    test2
        //    \       /
        //      build

        let compile = GradleTask::new("compile", "JavaCompile");

        let mut test1 = GradleTask::new("test1", "Test");
        test1.add_dependency("compile".to_string()).unwrap();

        let mut test2 = GradleTask::new("test2", "Test");
        test2.add_dependency("compile".to_string()).unwrap();

        let mut build = GradleTask::new("build", "DefaultTask");
        build.add_dependency("test1".to_string()).unwrap();
        build.add_dependency("test2".to_string()).unwrap();

        project.register_task(compile).unwrap();
        project.register_task(test1).unwrap();
        project.register_task(test2).unwrap();
        project.register_task(build).unwrap();

        let plan = TaskExecutor::execute_tasks_parallel(&project, &["build".to_string()]);
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.levels().len(), 3);

        // Level 0: compile
        assert_eq!(plan.levels()[0].len(), 1);
        assert!(plan.levels()[0].contains(&"compile".to_string()));

        // Level 1: test1 and test2 (can run in parallel)
        assert_eq!(plan.levels()[1].len(), 2);

        // Level 2: build
        assert_eq!(plan.levels()[2].len(), 1);
        assert!(plan.levels()[2].contains(&"build".to_string()));
    }

    #[test]
    fn test_task_execution_plan() {
        let mut plan = TaskExecutionPlan::new();
        assert!(plan.is_empty());

        let task = GradleTask::new("test", "Test");
        plan.add_task(task);

        assert!(!plan.is_empty());
        assert_eq!(plan.len(), 1);
    }
}

// - ToolchainResolver (finds matching JDK installations)
