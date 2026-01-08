use std::collections::{HashMap, HashSet};

use crate::core::project::MavenProject;

/// Reactor - manages multi-module project execution
pub struct Reactor {
    projects: Vec<MavenProject>,
    project_index: HashMap<String, usize>,
    dependency_graph: DependencyGraph,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Map from project ID to list of dependent project IDs
    dependencies: HashMap<String, Vec<String>>,
    /// Map from project ID to list of projects that depend on it
    dependents: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies
            .entry(from.clone())
            .or_default()
            .push(to.clone());
        self.dependents
            .entry(to)
            .or_default()
            .push(from);
    }

    /// Get build order (topological sort)
    pub fn build_order(&self, project_ids: &[String]) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        fn visit(
            project_id: &str,
            graph: &DependencyGraph,
            visited: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) {
            if visited.contains(project_id) {
                return;
            }
            visited.insert(project_id.to_string());

            if let Some(deps) = graph.dependencies.get(project_id) {
                for dep in deps {
                    visit(dep, graph, visited, result);
                }
            }

            result.push(project_id.to_string());
        }

        for project_id in project_ids {
            visit(project_id, self, &mut visited, &mut result);
        }

        result
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Reactor {
    pub fn new(projects: Vec<MavenProject>) -> Self {
        let mut project_index = HashMap::new();
        for (idx, project) in projects.iter().enumerate() {
            project_index.insert(project.id(), idx);
        }

        let mut dependency_graph = DependencyGraph::new();

        // Build dependency graph
        for project in &projects {
            let project_id = project.id();
            for dep in project.model.dependencies_vec() {
                let dep_id = format!("{}:{}", dep.group_id, dep.artifact_id);
                // Check if this dependency is in our reactor
                if project_index.contains_key(&dep_id) {
                    dependency_graph.add_dependency(project_id.clone(), dep_id);
                }
            }
        }

        Self {
            projects,
            project_index,
            dependency_graph,
        }
    }

    /// Get all projects in build order
    pub fn build_order(&self) -> Vec<&MavenProject> {
        let project_ids: Vec<String> = self.projects.iter().map(|p| p.id()).collect();
        let order = self.dependency_graph.build_order(&project_ids);
        
        let mut result = Vec::new();
        for id in order {
            if let Some(&idx) = self.project_index.get(&id) {
                result.push(&self.projects[idx]);
            }
        }
        result
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Option<&MavenProject> {
        self.project_index.get(id).map(|&idx| &self.projects[idx])
    }

    /// Get all projects
    pub fn projects(&self) -> &[MavenProject] {
        &self.projects
    }
}

