use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Workspace configuration for multi-project builds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JbuildWorkspace {
    /// Workspace metadata
    #[serde(flatten)]
    pub workspace: WorkspaceConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Workspace members (project paths relative to workspace root)
    pub members: Vec<String>,

    /// Default members to build when no specific members are specified
    #[serde(default)]
    pub default_members: Vec<String>,

    /// Workspace-wide dependency resolution strategy
    #[serde(default)]
    pub resolver: ResolverConfig,

    /// Workspace metadata
    #[serde(default)]
    pub package: WorkspacePackage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspacePackage {
    /// Workspace name
    pub name: Option<String>,

    /// Workspace version
    pub version: Option<String>,

    /// Workspace authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// Workspace description
    pub description: Option<String>,

    /// Workspace documentation URL
    pub documentation: Option<String>,

    /// Workspace repository URL
    pub repository: Option<String>,

    /// Workspace homepage URL
    pub homepage: Option<String>,

    /// Workspace license
    pub license: Option<String>,

    /// Workspace keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Workspace categories
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResolverConfig {
    /// Version resolution strategy
    #[serde(default)]
    pub version_resolution: VersionResolution,

    /// Conflict resolution strategy
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum VersionResolution {
    /// Use highest version (default)
    #[default]
    Highest,
    /// Use lowest version
    Lowest,
    /// Fail on version conflicts
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ConflictResolution {
    /// Choose highest version (default)
    #[default]
    Highest,
    /// Choose lowest version
    Lowest,
    /// Fail on conflicts
    Fail,
}

/// Resolved workspace with absolute paths
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Root directory of the workspace
    pub root: PathBuf,

    /// Workspace configuration
    pub config: JbuildWorkspace,

    /// Resolved member projects with absolute paths
    pub members: Vec<WorkspaceMember>,

    /// Dependency graph between members
    pub dependency_graph: Vec<WorkspaceDependency>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    /// Project name (derived from directory name or package name)
    pub name: String,

    /// Absolute path to the project directory
    pub path: PathBuf,

    /// Relative path from workspace root
    pub relative_path: String,

    /// Build system detected for this member
    pub build_system: Option<crate::build::BuildSystem>,

    /// Dependencies on other workspace members
    pub workspace_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceDependency {
    /// Dependent project name
    pub from: String,

    /// Dependency project name
    pub to: String,

    /// Type of dependency
    pub dependency_type: WorkspaceDependencyType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceDependencyType {
    /// Direct dependency in build file
    Direct,
    /// Transitive dependency through other members
    Transitive,
}

impl Default for JbuildWorkspace {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig {
                members: vec![],
                default_members: vec![],
                resolver: ResolverConfig::default(),
                package: WorkspacePackage::default(),
            },
        }
    }
}

impl JbuildWorkspace {
    /// Load workspace configuration from jbuild-workspace.toml
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: JbuildWorkspace = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save workspace configuration to jbuild-workspace.toml
    pub fn save_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Create a new workspace configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member to the workspace
    pub fn add_member(&mut self, member_path: String) {
        if !self.workspace.members.contains(&member_path) {
            self.workspace.members.push(member_path);
        }
    }

    /// Remove a member from the workspace
    pub fn remove_member(&mut self, member_path: &str) {
        self.workspace.members.retain(|m| m != member_path);
        self.workspace.default_members.retain(|m| m != member_path);
    }

    /// Set default members
    pub fn set_default_members(&mut self, members: Vec<String>) {
        self.workspace.default_members = members;
    }
}

impl Workspace {
    /// Load workspace from workspace root directory
    pub fn from_directory(root: &Path) -> anyhow::Result<Self> {
        let workspace_file = root.join("jbuild-workspace.toml");

        if !workspace_file.exists() {
            return Err(anyhow::anyhow!(
                "No jbuild-workspace.toml found in {}",
                root.display()
            ));
        }

        let config = JbuildWorkspace::from_file(&workspace_file)?;

        // Resolve members to absolute paths
        let members = Self::resolve_members(root, &config)?;
        let dependency_graph = Self::build_dependency_graph(&members)?;

        Ok(Workspace {
            root: root.to_path_buf(),
            config,
            members,
            dependency_graph,
        })
    }

    /// Resolve workspace members to WorkspaceMember structs
    fn resolve_members(root: &Path, config: &JbuildWorkspace) -> anyhow::Result<Vec<WorkspaceMember>> {
        let mut members = Vec::new();

        for member_path in &config.workspace.members {
            let member_dir = root.join(member_path);
            if !member_dir.exists() {
                return Err(anyhow::anyhow!(
                    "Workspace member directory does not exist: {}",
                    member_dir.display()
                ));
            }

            // Detect build system
            let build_system = crate::build::BuildSystem::detect(&member_dir);

            // Extract project name from directory or build file
            let name = Self::extract_project_name(&member_dir)?;

            // Analyze workspace dependencies
            let workspace_dependencies = Self::analyze_workspace_dependencies(&member_dir, &members)?;

            members.push(WorkspaceMember {
                name,
                path: member_dir,
                relative_path: member_path.clone(),
                build_system,
                workspace_dependencies,
            });
        }

        Ok(members)
    }

    /// Extract project name from directory or build files
    fn extract_project_name(member_dir: &Path) -> anyhow::Result<String> {
        // Try pom.xml first
        let pom_path = member_dir.join("pom.xml");
        if pom_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pom_path) {
                if let Some(name) = Self::extract_name_from_pom(&content) {
                    return Ok(name);
                }
            }
        }

        // Try build.gradle
        let gradle_path = member_dir.join("build.gradle");
        if gradle_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gradle_path) {
                if let Some(name) = Self::extract_name_from_gradle(&content) {
                    return Ok(name);
                }
            }
        }

        // Fall back to directory name
        Ok(member_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string())
    }

    /// Extract name from pom.xml
    fn extract_name_from_pom(content: &str) -> Option<String> {
        // Simple regex to extract artifactId
        let re = regex::Regex::new(r"<artifactId>([^<]+)</artifactId>").ok()?;
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                return Some(name.as_str().to_string());
            }
        }
        None
    }

    /// Extract name from build.gradle
    fn extract_name_from_gradle(content: &str) -> Option<String> {
        // Look for rootProject.name = "..."
        let re = regex::Regex::new(r#"rootProject\.name\s*=\s*['"]([^'"]+)['"]"#).ok()?;
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                return Some(name.as_str().to_string());
            }
        }
        None
    }

    /// Analyze workspace dependencies for a member
    fn analyze_workspace_dependencies(member_dir: &Path, existing_members: &[WorkspaceMember]) -> anyhow::Result<Vec<String>> {
        let mut deps = Vec::new();

        // Check pom.xml for workspace dependencies
        let pom_path = member_dir.join("pom.xml");
        if pom_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pom_path) {
                deps.extend(Self::extract_workspace_deps_from_pom(&content, existing_members));
            }
        }

        // Check build.gradle for workspace dependencies
        let gradle_path = member_dir.join("build.gradle");
        if gradle_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gradle_path) {
                deps.extend(Self::extract_workspace_deps_from_gradle(&content, existing_members));
            }
        }

        Ok(deps)
    }

    /// Extract workspace dependencies from pom.xml
    fn extract_workspace_deps_from_pom(content: &str, members: &[WorkspaceMember]) -> Vec<String> {
        let mut deps = Vec::new();
        let re = regex::Regex::new(r"<artifactId>([^<]+)</artifactId>").unwrap();

        for cap in re.captures_iter(content) {
            if let Some(artifact_id) = cap.get(1) {
                let artifact_name = artifact_id.as_str();
                // Check if this artifact matches any workspace member
                if members.iter().any(|m| m.name == artifact_name) {
                    deps.push(artifact_name.to_string());
                }
            }
        }

        deps
    }

    /// Extract workspace dependencies from build.gradle
    fn extract_workspace_deps_from_gradle(content: &str, members: &[WorkspaceMember]) -> Vec<String> {
        let mut deps = Vec::new();
        // Look for project(':member') or project(':path:to:member')
        let re = regex::Regex::new(r#"project\s*\(\s*['"]:([^'"]+)['"]\s*\)"#).unwrap();

        for cap in re.captures_iter(content) {
            if let Some(dep_path) = cap.get(1) {
                let dep_name = dep_path.as_str().split(':').last().unwrap_or(dep_path.as_str());
                if members.iter().any(|m| m.name == dep_name) {
                    deps.push(dep_name.to_string());
                }
            }
        }

        deps
    }

    /// Build dependency graph between workspace members
    fn build_dependency_graph(members: &[WorkspaceMember]) -> anyhow::Result<Vec<WorkspaceDependency>> {
        let mut graph = Vec::new();

        for member in members {
            for dep_name in &member.workspace_dependencies {
                if members.iter().any(|m| &m.name == dep_name) {
                    graph.push(WorkspaceDependency {
                        from: member.name.clone(),
                        to: dep_name.clone(),
                        dependency_type: WorkspaceDependencyType::Direct,
                    });
                }
            }
        }

        // Add transitive dependencies
        let mut transitive_deps = Vec::new();
        for dep in &graph {
            // Find all dependencies of the target
            for target_dep in &graph {
                if target_dep.from == dep.to {
                    transitive_deps.push(WorkspaceDependency {
                        from: dep.from.clone(),
                        to: target_dep.to.clone(),
                        dependency_type: WorkspaceDependencyType::Transitive,
                    });
                }
            }
        }

        graph.extend(transitive_deps);
        Ok(graph)
    }

    /// Get build order for workspace members
    pub fn get_build_order(&self) -> Vec<&WorkspaceMember> {
        // Simple topological sort for build ordering
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for member in &self.members {
            if !visited.contains(&member.name) {
                self.visit_member(member, &mut visited, &mut result);
            }
        }

        result
    }

    fn visit_member<'a>(
        &'a self,
        member: &'a WorkspaceMember,
        visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<&'a WorkspaceMember>,
    ) {
        visited.insert(member.name.clone());

        // Visit dependencies first
        for dep_name in &member.workspace_dependencies {
            if let Some(dep_member) = self.members.iter().find(|m| &m.name == dep_name) {
                if !visited.contains(&dep_member.name) {
                    self.visit_member(dep_member, visited, result);
                }
            }
        }

        result.push(member);
    }

    /// Check if a directory is a workspace root
    pub fn is_workspace_root(dir: &Path) -> bool {
        dir.join("jbuild-workspace.toml").exists()
    }
}