//! Maven Project Dependencies Resolver
//!
//! Implements Maven's ProjectDependenciesResolver for resolving project dependencies
//! at different scopes and phases.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use anyhow::Result;

use crate::maven::dependency_context::{DependencyContext, DependencyScope, ResolvedDependency};

/// Resolution scope for dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionScope {
    /// Compile scope - compile + provided + system
    Compile,
    /// Compile + runtime scope
    CompilePlusRuntime,
    /// Runtime scope - compile + runtime
    Runtime,
    /// Test scope - all scopes
    Test,
}

impl ResolutionScope {
    /// Get the scopes included in this resolution scope
    pub fn included_scopes(&self) -> HashSet<DependencyScope> {
        match self {
            ResolutionScope::Compile => {
                [DependencyScope::Compile, DependencyScope::Provided, DependencyScope::System]
                    .into_iter().collect()
            }
            ResolutionScope::CompilePlusRuntime => {
                [DependencyScope::Compile, DependencyScope::Provided, 
                 DependencyScope::System, DependencyScope::Runtime]
                    .into_iter().collect()
            }
            ResolutionScope::Runtime => {
                [DependencyScope::Compile, DependencyScope::Runtime]
                    .into_iter().collect()
            }
            ResolutionScope::Test => {
                [DependencyScope::Compile, DependencyScope::Provided, 
                 DependencyScope::System, DependencyScope::Runtime, DependencyScope::Test]
                    .into_iter().collect()
            }
        }
    }
}

/// Request for dependency resolution
#[derive(Debug, Clone)]
pub struct DependencyResolutionRequest {
    /// Project group ID
    pub project_group_id: String,
    /// Project artifact ID
    pub project_artifact_id: String,
    /// Project version
    pub project_version: String,
    /// Dependencies to resolve
    pub dependencies: Vec<DependencySpec>,
    /// Managed dependencies (for version management)
    pub managed_dependencies: HashMap<String, String>,
    /// Resolution scope
    pub scope: ResolutionScope,
    /// Exclusions
    pub exclusions: HashSet<String>,
    /// Local repository path
    pub local_repository: PathBuf,
    /// Remote repositories
    pub remote_repositories: Vec<String>,
}

/// Dependency specification
#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub scope: DependencyScope,
    pub optional: bool,
    pub classifier: Option<String>,
    pub artifact_type: String,
    pub exclusions: Vec<String>,
}

impl DependencySpec {
    pub fn new(group_id: impl Into<String>, artifact_id: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
            version: None,
            scope: DependencyScope::Compile,
            optional: false,
            classifier: None,
            artifact_type: "jar".to_string(),
            exclusions: Vec::new(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_scope(mut self, scope: DependencyScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn key(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }
}

impl DependencyResolutionRequest {
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            project_group_id: group_id.into(),
            project_artifact_id: artifact_id.into(),
            project_version: version.into(),
            dependencies: Vec::new(),
            managed_dependencies: HashMap::new(),
            scope: ResolutionScope::Compile,
            exclusions: HashSet::new(),
            local_repository: PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
            ).join(".m2/repository"),
            remote_repositories: vec!["https://repo.maven.apache.org/maven2".to_string()],
        }
    }

    pub fn with_scope(mut self, scope: ResolutionScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn add_dependency(&mut self, dep: DependencySpec) {
        self.dependencies.push(dep);
    }

    pub fn add_managed_dependency(&mut self, key: impl Into<String>, version: impl Into<String>) {
        self.managed_dependencies.insert(key.into(), version.into());
    }
}

/// Result of dependency resolution
#[derive(Debug)]
pub struct DependencyResolutionResult {
    /// Resolved dependency context
    pub context: DependencyContext,
    /// Resolution errors
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl DependencyResolutionResult {
    pub fn new(context: DependencyContext) -> Self {
        Self {
            context,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Project dependencies resolver
pub struct ProjectDependenciesResolver {
    /// Local repository path
    local_repository: PathBuf,
}

impl ProjectDependenciesResolver {
    pub fn new(local_repository: PathBuf) -> Self {
        Self { local_repository }
    }

    /// Resolve dependencies for a request
    pub fn resolve(&self, request: &DependencyResolutionRequest) -> Result<DependencyResolutionResult> {
        let context = DependencyContext::new();
        let mut result = DependencyResolutionResult::new(context);

        let included_scopes = request.scope.included_scopes();

        for dep_spec in &request.dependencies {
            // Skip if scope not included
            if !included_scopes.contains(&dep_spec.scope) {
                continue;
            }

            // Skip if excluded
            if request.exclusions.contains(&dep_spec.key()) {
                continue;
            }

            // Resolve version from managed dependencies if not specified
            let version = dep_spec.version.clone()
                .or_else(|| request.managed_dependencies.get(&dep_spec.key()).cloned())
                .unwrap_or_else(|| "LATEST".to_string());

            // Try to find the artifact in local repository
            let artifact_path = self.find_artifact(
                &dep_spec.group_id,
                &dep_spec.artifact_id,
                &version,
                dep_spec.classifier.as_deref(),
                &dep_spec.artifact_type,
            );

            let mut resolved = ResolvedDependency::new(
                &dep_spec.group_id,
                &dep_spec.artifact_id,
                &version,
            ).with_scope(dep_spec.scope);

            if let Some(path) = artifact_path {
                resolved = resolved.with_file(path);
            } else {
                result.add_warning(format!(
                    "Artifact not found in local repository: {}:{}:{}",
                    dep_spec.group_id, dep_spec.artifact_id, version
                ));
            }

            if let Some(ref classifier) = dep_spec.classifier {
                resolved = resolved.with_classifier(classifier);
            }

            result.context.add(resolved);
        }

        Ok(result)
    }

    /// Find an artifact in the local repository
    fn find_artifact(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        classifier: Option<&str>,
        artifact_type: &str,
    ) -> Option<PathBuf> {
        let group_path = group_id.replace('.', "/");
        let filename = if let Some(classifier) = classifier {
            format!("{}-{}-{}.{}", artifact_id, version, classifier, artifact_type)
        } else {
            format!("{}-{}.{}", artifact_id, version, artifact_type)
        };

        let path = self.local_repository
            .join(&group_path)
            .join(artifact_id)
            .join(version)
            .join(&filename);

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_scope_included_scopes() {
        let compile_scopes = ResolutionScope::Compile.included_scopes();
        assert!(compile_scopes.contains(&DependencyScope::Compile));
        assert!(compile_scopes.contains(&DependencyScope::Provided));
        assert!(!compile_scopes.contains(&DependencyScope::Test));

        let test_scopes = ResolutionScope::Test.included_scopes();
        assert!(test_scopes.contains(&DependencyScope::Compile));
        assert!(test_scopes.contains(&DependencyScope::Test));
    }

    #[test]
    fn test_dependency_spec() {
        let spec = DependencySpec::new("com.example", "lib")
            .with_version("1.0.0")
            .with_scope(DependencyScope::Test);

        assert_eq!(spec.key(), "com.example:lib");
        assert_eq!(spec.version, Some("1.0.0".to_string()));
        assert_eq!(spec.scope, DependencyScope::Test);
    }

    #[test]
    fn test_dependency_resolution_request() {
        let mut request = DependencyResolutionRequest::new("com.example", "app", "1.0.0")
            .with_scope(ResolutionScope::Test);

        request.add_dependency(DependencySpec::new("junit", "junit").with_version("4.13.2"));
        request.add_managed_dependency("org.slf4j:slf4j-api", "2.0.0");

        assert_eq!(request.dependencies.len(), 1);
        assert!(request.managed_dependencies.contains_key("org.slf4j:slf4j-api"));
    }

    #[test]
    fn test_project_dependencies_resolver() {
        let resolver = ProjectDependenciesResolver::new(PathBuf::from("/tmp/repo"));
        
        let request = DependencyResolutionRequest::new("com.example", "app", "1.0.0");
        let result = resolver.resolve(&request).unwrap();

        assert!(result.is_success());
    }
}
