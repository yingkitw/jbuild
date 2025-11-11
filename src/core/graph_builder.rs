use std::collections::{HashMap, HashSet};

use crate::core::project::MavenProject;

/// Project dependency graph builder
pub struct GraphBuilder;

/// Project dependency graph
#[derive(Debug, Clone)]
pub struct ProjectDependencyGraph {
    /// Map from project ID to list of project IDs it depends on
    dependencies: HashMap<String, Vec<String>>,
    /// All project IDs
    project_ids: Vec<String>,
}

impl ProjectDependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            project_ids: Vec::new(),
        }
    }

    /// Add a project and its dependencies
    pub fn add_project(&mut self, project_id: String, dependencies: Vec<String>) {
        self.project_ids.push(project_id.clone());
        self.dependencies.insert(project_id, dependencies);
    }

    /// Get dependencies for a project
    pub fn get_dependencies(&self, project_id: &str) -> &[String] {
        self.dependencies
            .get(project_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all project IDs in build order (topological sort)
    pub fn build_order(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        fn visit(
            project_id: &str,
            graph: &ProjectDependencyGraph,
            visited: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) {
            if visited.contains(project_id) {
                return;
            }
            visited.insert(project_id.to_string());

            // Visit dependencies first
            for dep in graph.get_dependencies(project_id) {
                visit(dep, graph, visited, result);
            }

            result.push(project_id.to_string());
        }

        for project_id in &self.project_ids {
            visit(project_id, self, &mut visited, &mut result);
        }

        result
    }

    /// Check for cycles
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn has_cycle(
            project_id: &str,
            graph: &ProjectDependencyGraph,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
        ) -> bool {
            visited.insert(project_id.to_string());
            rec_stack.insert(project_id.to_string());

            for dep in graph.get_dependencies(project_id) {
                if !visited.contains(dep) {
                    if has_cycle(dep, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }

            rec_stack.remove(project_id);
            false
        }

        for project_id in &self.project_ids {
            if !visited.contains(project_id) {
                if has_cycle(project_id, self, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for ProjectDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build dependency graph from projects
    pub fn build(&self, projects: &[MavenProject]) -> ProjectDependencyGraph {
        let mut graph = ProjectDependencyGraph::new();
        let project_map: HashMap<String, &MavenProject> = projects
            .iter()
            .map(|p| (p.id(), p))
            .collect();

        for project in projects {
            let project_id = project.id();
            let mut deps = Vec::new();

            // Find dependencies that are in the reactor
            for dep in project.model.dependencies_vec() {
                let dep_id = format!("{}:{}", dep.group_id, dep.artifact_id);
                if project_map.contains_key(&dep_id) {
                    deps.push(dep_id);
                }
            }

            graph.add_project(project_id, deps);
        }

        graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

