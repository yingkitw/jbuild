//! Enhanced Reactor for multi-module project execution with parallel processing

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use crate::core::project::MavenProject;

/// Reactor - manages multi-module project execution with optimization
pub struct Reactor {
    projects: Vec<MavenProject>,
    project_index: HashMap<String, usize>,
    dependency_graph: DependencyGraph,
}

/// Build execution result
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub project_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub output: String,
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

    /// Identify projects that can be built in parallel (independent batches)
    pub fn parallel_batches(&self, project_ids: &[String]) -> Vec<Vec<String>> {
        let build_order = self.build_order(project_ids);
        let mut batches = Vec::new();
        let mut built = HashSet::new();

        for project_id in build_order {
            // Check if all dependencies are built
            let deps = self.dependencies.get(&project_id);
            let can_build = deps
                .map(|d| d.iter().all(|dep| built.contains(dep)))
                .unwrap_or(true);

            if can_build {
                // Find all projects that can be built with this project in parallel
                let parallel_group: Vec<String> = project_ids
                    .iter()
                    .filter(|id| {
                        !built.contains(*id) && {
                            let d = self.dependencies.get(*id);
                            d.map(|deps| deps.iter().all(|dep| built.contains(dep) || *dep == project_id))
                                .unwrap_or(true)
                        }
                    })
                    .cloned()
                    .collect();

                if !parallel_group.is_empty() {
                    for id in &parallel_group {
                        built.insert(id.clone());
                    }
                    batches.push(parallel_group);
                }
            }
        }

        batches
    }

    /// Calculate the critical path (longest dependency chain)
    pub fn critical_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        let mut visited = HashSet::new();

        fn dfs(
            project_id: &str,
            graph: &DependencyGraph,
            visited: &mut HashSet<String>,
            current_path: &mut Vec<String>,
            longest_path: &mut Vec<String>,
        ) {
            visited.insert(project_id.to_string());
            current_path.push(project_id.to_string());

            let dependents = graph.dependents.get(project_id);
            let has_unvisited_dependents = dependents
                .map(|deps| deps.iter().any(|dep| !visited.contains(dep)))
                .unwrap_or(false);

            if !has_unvisited_dependents {
                // Leaf node
                if current_path.len() > longest_path.len() {
                    longest_path.clone_from(current_path);
                }
            } else if let Some(deps) = dependents {
                for dep in deps {
                    if !visited.contains(dep) {
                        dfs(dep, graph, visited, current_path, longest_path);
                    }
                }
            }

            current_path.pop();
            visited.remove(project_id);
        }

        // Find all root projects (no dependencies)
        let roots: Vec<_> = self
            .dependencies
            .keys()
            .filter(|id| {
                self.dependencies
                    .get(*id)
                    .map(|d| d.is_empty())
                    .unwrap_or(true)
            })
            .collect();

        for root in roots {
            let mut current = Vec::new();
            dfs(root, self, &mut visited, &mut current, &mut path);
        }

        path
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

    /// Execute builds in parallel where possible
    pub fn execute_parallel<F>(&self, build_fn: F) -> Vec<BuildResult>
    where
        F: FnMut(&MavenProject) -> BuildResult + Sync + Send,
    {
        let project_ids: Vec<String> = self.projects.iter().map(|p| p.id()).collect();
        let batches = self.dependency_graph.parallel_batches(&project_ids);

        let mut all_results = Vec::new();
        let build_fn = Arc::new(Mutex::new(build_fn));

        for batch in batches {
            let batch_results: Vec<BuildResult> = batch
                .par_iter()
                .filter_map(|project_id| {
                    self.get_project(project_id).and_then(|project| {
                        let mut fn_lock = build_fn.lock().ok()?;
                        Some((*fn_lock)(project))
                    })
                })
                .collect();

            all_results.extend(batch_results);
        }

        all_results
    }

    /// Get execution statistics
    pub fn execution_stats(&self) -> ExecutionStats {
        let project_ids: Vec<String> = self.projects.iter().map(|p| p.id()).collect();
        let batches = self.dependency_graph.parallel_batches(&project_ids);
        let critical_path = self.dependency_graph.critical_path();

        ExecutionStats {
            total_projects: self.projects.len(),
            parallel_batches: batches.len(),
            max_parallelism: batches.iter().map(|b| b.len()).max().unwrap_or(0),
            critical_path_length: critical_path.len(),
            estimated_speedup: if batches.len() > 1 {
                let total_work = self.projects.len();
                let critical_work = critical_path.len();
                (total_work as f64 / critical_work as f64 * 10.0).round() / 10.0
            } else {
                1.0
            },
        }
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Option<&MavenProject> {
        self.project_index.get(id).map(|&idx| &self.projects[idx])
    }

    /// Get all projects
    pub fn projects(&self) -> &[MavenProject] {
        &self.projects
    }

    /// Get dependency graph reference
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }
}

/// Execution statistics for the reactor
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_projects: usize,
    pub parallel_batches: usize,
    pub max_parallelism: usize,
    pub critical_path_length: usize,
    pub estimated_speedup: f64,
}
