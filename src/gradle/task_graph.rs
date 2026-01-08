//! Gradle Task Graph
//!
//! Implements task graph construction and execution ordering,
//! inspired by Gradle's TaskExecutionGraph.

use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Result, anyhow};

/// Task node in the execution graph
#[derive(Debug, Clone)]
pub struct TaskNode {
    /// Task name
    pub name: String,
    /// Task type (e.g., "JavaCompile", "Test", "Jar")
    pub task_type: Option<String>,
    /// Tasks this task depends on
    pub dependencies: Vec<String>,
    /// Tasks that depend on this task
    pub dependents: Vec<String>,
    /// Whether this task should be skipped
    pub skip: bool,
    /// Whether this task is up-to-date
    pub up_to_date: bool,
}

impl TaskNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            task_type: None,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            skip: false,
            up_to_date: false,
        }
    }

    pub fn with_type(mut self, task_type: impl Into<String>) -> Self {
        self.task_type = Some(task_type.into());
        self
    }

    pub fn depends_on(mut self, task: impl Into<String>) -> Self {
        self.dependencies.push(task.into());
        self
    }
}

/// Task execution graph
#[derive(Debug, Default)]
pub struct TaskGraph {
    /// All tasks in the graph
    tasks: HashMap<String, TaskNode>,
    /// Requested tasks to execute
    requested_tasks: Vec<String>,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task to the graph
    pub fn add_task(&mut self, task: TaskNode) {
        self.tasks.insert(task.name.clone(), task);
    }

    /// Add a dependency between tasks
    pub fn add_dependency(&mut self, task: &str, depends_on: &str) -> Result<()> {
        // Add dependency to the task
        if let Some(t) = self.tasks.get_mut(task) {
            if !t.dependencies.contains(&depends_on.to_string()) {
                t.dependencies.push(depends_on.to_string());
            }
        } else {
            return Err(anyhow!("Task not found: {task}"));
        }

        // Add dependent to the dependency
        if let Some(d) = self.tasks.get_mut(depends_on) {
            if !d.dependents.contains(&task.to_string()) {
                d.dependents.push(task.to_string());
            }
        }

        Ok(())
    }

    /// Set the requested tasks to execute
    pub fn request_tasks(&mut self, tasks: Vec<String>) {
        self.requested_tasks = tasks;
    }

    /// Get all tasks that need to be executed (including dependencies)
    pub fn get_execution_plan(&self) -> Result<Vec<String>> {
        let mut to_execute: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        // Start with requested tasks
        for task in &self.requested_tasks {
            if !self.tasks.contains_key(task) {
                return Err(anyhow!("Requested task not found: {task}"));
            }
            queue.push_back(task.clone());
        }

        // Collect all tasks including dependencies
        while let Some(task_name) = queue.pop_front() {
            if to_execute.contains(&task_name) {
                continue;
            }
            to_execute.insert(task_name.clone());

            if let Some(task) = self.tasks.get(&task_name) {
                for dep in &task.dependencies {
                    if !to_execute.contains(dep) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        // Topological sort
        self.topological_sort(&to_execute)
    }

    /// Perform topological sort on the tasks
    fn topological_sort(&self, tasks: &HashSet<String>) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for task in tasks {
            if !visited.contains(task) {
                self.visit(task, tasks, &mut visited, &mut temp_visited, &mut result)?;
            }
        }

        Ok(result)
    }

    /// DFS visit for topological sort
    fn visit(
        &self,
        task: &str,
        tasks: &HashSet<String>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(task) {
            return Err(anyhow!("Circular dependency detected involving task: {task}"));
        }
        if visited.contains(task) {
            return Ok(());
        }

        temp_visited.insert(task.to_string());

        if let Some(node) = self.tasks.get(task) {
            for dep in &node.dependencies {
                if tasks.contains(dep) {
                    self.visit(dep, tasks, visited, temp_visited, result)?;
                }
            }
        }

        temp_visited.remove(task);
        visited.insert(task.to_string());
        result.push(task.to_string());

        Ok(())
    }

    /// Check if a task exists
    pub fn has_task(&self, name: &str) -> bool {
        self.tasks.contains_key(name)
    }

    /// Get a task by name
    pub fn get_task(&self, name: &str) -> Option<&TaskNode> {
        self.tasks.get(name)
    }

    /// Get all task names
    pub fn all_tasks(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Mark a task as up-to-date
    pub fn mark_up_to_date(&mut self, task: &str) {
        if let Some(t) = self.tasks.get_mut(task) {
            t.up_to_date = true;
        }
    }

    /// Check if a task is up-to-date
    pub fn is_up_to_date(&self, task: &str) -> bool {
        self.tasks.get(task).map(|t| t.up_to_date).unwrap_or(false)
    }
}

/// Build a standard Java task graph
pub fn build_java_task_graph() -> TaskGraph {
    let mut graph = TaskGraph::new();

    // Add standard Java tasks
    graph.add_task(TaskNode::new("clean").with_type("Delete"));
    graph.add_task(TaskNode::new("compileJava").with_type("JavaCompile"));
    graph.add_task(TaskNode::new("processResources").with_type("Copy"));
    graph.add_task(TaskNode::new("classes").with_type("DefaultTask"));
    graph.add_task(TaskNode::new("compileTestJava").with_type("JavaCompile"));
    graph.add_task(TaskNode::new("processTestResources").with_type("Copy"));
    graph.add_task(TaskNode::new("testClasses").with_type("DefaultTask"));
    graph.add_task(TaskNode::new("test").with_type("Test"));
    graph.add_task(TaskNode::new("jar").with_type("Jar"));
    graph.add_task(TaskNode::new("assemble").with_type("DefaultTask"));
    graph.add_task(TaskNode::new("check").with_type("DefaultTask"));
    graph.add_task(TaskNode::new("build").with_type("DefaultTask"));

    // Add dependencies
    let _ = graph.add_dependency("classes", "compileJava");
    let _ = graph.add_dependency("classes", "processResources");
    let _ = graph.add_dependency("compileTestJava", "classes");
    let _ = graph.add_dependency("testClasses", "compileTestJava");
    let _ = graph.add_dependency("testClasses", "processTestResources");
    let _ = graph.add_dependency("test", "testClasses");
    let _ = graph.add_dependency("jar", "classes");
    let _ = graph.add_dependency("assemble", "jar");
    let _ = graph.add_dependency("check", "test");
    let _ = graph.add_dependency("build", "assemble");
    let _ = graph.add_dependency("build", "check");

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_node_creation() {
        let task = TaskNode::new("compile")
            .with_type("JavaCompile")
            .depends_on("processResources");

        assert_eq!(task.name, "compile");
        assert_eq!(task.task_type, Some("JavaCompile".to_string()));
        assert_eq!(task.dependencies, vec!["processResources".to_string()]);
    }

    #[test]
    fn test_task_graph_add_task() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("test"));

        assert!(graph.has_task("test"));
        assert!(!graph.has_task("nonexistent"));
    }

    #[test]
    fn test_task_graph_add_dependency() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("compile"));
        graph.add_task(TaskNode::new("test"));

        graph.add_dependency("test", "compile").unwrap();

        let test_task = graph.get_task("test").unwrap();
        assert!(test_task.dependencies.contains(&"compile".to_string()));
    }

    #[test]
    fn test_execution_plan_simple() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("a"));
        graph.add_task(TaskNode::new("b"));
        graph.add_task(TaskNode::new("c"));

        graph.add_dependency("b", "a").unwrap();
        graph.add_dependency("c", "b").unwrap();

        graph.request_tasks(vec!["c".to_string()]);
        let plan = graph.get_execution_plan().unwrap();

        // a must come before b, b must come before c
        let pos_a = plan.iter().position(|x| x == "a").unwrap();
        let pos_b = plan.iter().position(|x| x == "b").unwrap();
        let pos_c = plan.iter().position(|x| x == "c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_execution_plan_diamond() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("a"));
        graph.add_task(TaskNode::new("b"));
        graph.add_task(TaskNode::new("c"));
        graph.add_task(TaskNode::new("d"));

        // Diamond: d depends on b and c, both depend on a
        graph.add_dependency("b", "a").unwrap();
        graph.add_dependency("c", "a").unwrap();
        graph.add_dependency("d", "b").unwrap();
        graph.add_dependency("d", "c").unwrap();

        graph.request_tasks(vec!["d".to_string()]);
        let plan = graph.get_execution_plan().unwrap();

        assert_eq!(plan.len(), 4);
        let pos_a = plan.iter().position(|x| x == "a").unwrap();
        let pos_b = plan.iter().position(|x| x == "b").unwrap();
        let pos_c = plan.iter().position(|x| x == "c").unwrap();
        let pos_d = plan.iter().position(|x| x == "d").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("a"));
        graph.add_task(TaskNode::new("b"));

        graph.add_dependency("a", "b").unwrap();
        graph.add_dependency("b", "a").unwrap();

        graph.request_tasks(vec!["a".to_string()]);
        let result = graph.get_execution_plan();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_java_task_graph() {
        let graph = build_java_task_graph();

        assert!(graph.has_task("compileJava"));
        assert!(graph.has_task("test"));
        assert!(graph.has_task("jar"));
        assert!(graph.has_task("build"));
    }

    #[test]
    fn test_java_build_execution_plan() {
        let mut graph = build_java_task_graph();
        graph.request_tasks(vec!["build".to_string()]);

        let plan = graph.get_execution_plan().unwrap();

        // Verify key ordering constraints
        let pos_compile = plan.iter().position(|x| x == "compileJava").unwrap();
        let pos_test = plan.iter().position(|x| x == "test").unwrap();
        let pos_jar = plan.iter().position(|x| x == "jar").unwrap();
        let pos_build = plan.iter().position(|x| x == "build").unwrap();

        assert!(pos_compile < pos_test);
        assert!(pos_compile < pos_jar);
        assert!(pos_test < pos_build);
        assert!(pos_jar < pos_build);
    }

    #[test]
    fn test_up_to_date_marking() {
        let mut graph = TaskGraph::new();
        graph.add_task(TaskNode::new("compile"));

        assert!(!graph.is_up_to_date("compile"));

        graph.mark_up_to_date("compile");

        assert!(graph.is_up_to_date("compile"));
    }
}
