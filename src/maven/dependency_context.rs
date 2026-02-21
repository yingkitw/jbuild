//! Maven Dependency Context
//!
//! Implements Maven's DependencyContext for managing dependency resolution
//! during build execution.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;

/// Dependency scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyScope {
    /// Compile scope - available in all classpaths
    Compile,
    /// Provided scope - expected to be provided by JDK or container
    Provided,
    /// Runtime scope - not needed for compilation, only runtime
    Runtime,
    /// Test scope - only for test compilation and execution
    Test,
    /// System scope - similar to provided, but must specify path
    System,
    /// Import scope - only for dependency management
    Import,
}

impl DependencyScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyScope::Compile => "compile",
            DependencyScope::Provided => "provided",
            DependencyScope::Runtime => "runtime",
            DependencyScope::Test => "test",
            DependencyScope::System => "system",
            DependencyScope::Import => "import",
        }
    }

    pub fn in_compile_classpath(&self) -> bool {
        matches!(
            self,
            DependencyScope::Compile | DependencyScope::Provided | DependencyScope::System
        )
    }

    pub fn in_runtime_classpath(&self) -> bool {
        matches!(self, DependencyScope::Compile | DependencyScope::Runtime)
    }

    pub fn in_test_classpath(&self) -> bool {
        // Test classpath includes everything except import
        !matches!(self, DependencyScope::Import)
    }
}

impl FromStr for DependencyScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compile" => Ok(DependencyScope::Compile),
            "provided" => Ok(DependencyScope::Provided),
            "runtime" => Ok(DependencyScope::Runtime),
            "test" => Ok(DependencyScope::Test),
            "system" => Ok(DependencyScope::System),
            "import" => Ok(DependencyScope::Import),
            _ => Err(format!("Invalid dependency scope: {}", s)),
        }
    }
}

/// A resolved dependency
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// Group ID
    pub group_id: String,
    /// Artifact ID
    pub artifact_id: String,
    /// Version
    pub version: String,
    /// Scope
    pub scope: DependencyScope,
    /// Path to the artifact file
    pub file: Option<PathBuf>,
    /// Whether this is optional
    pub optional: bool,
    /// Classifier (e.g., "sources", "javadoc")
    pub classifier: Option<String>,
    /// Type (e.g., "jar", "pom")
    pub artifact_type: String,
    /// Transitive dependencies
    pub dependencies: Vec<ResolvedDependency>,
}

impl ResolvedDependency {
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
            version: version.into(),
            scope: DependencyScope::Compile,
            file: None,
            optional: false,
            classifier: None,
            artifact_type: "jar".to_string(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_scope(mut self, scope: DependencyScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_classifier(mut self, classifier: impl Into<String>) -> Self {
        self.classifier = Some(classifier.into());
        self
    }

    /// Get the GAV coordinate
    pub fn gav(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }

    /// Get the full coordinate including classifier
    pub fn full_coordinate(&self) -> String {
        if let Some(ref classifier) = self.classifier {
            format!(
                "{}:{}:{}:{}",
                self.group_id, self.artifact_id, self.version, classifier
            )
        } else {
            self.gav()
        }
    }
}

/// Dependency context for a build
#[derive(Debug, Default)]
pub struct DependencyContext {
    /// All resolved dependencies
    dependencies: Vec<ResolvedDependency>,
    /// Dependency index by GAV
    dependency_index: HashMap<String, usize>,
    /// Exclusions (groupId:artifactId patterns)
    exclusions: HashSet<String>,
    /// Scope filter
    scope_filter: Option<HashSet<DependencyScope>>,
}

impl DependencyContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a resolved dependency
    pub fn add(&mut self, dep: ResolvedDependency) {
        let gav = dep.gav();
        if !self.dependency_index.contains_key(&gav) {
            let index = self.dependencies.len();
            self.dependencies.push(dep);
            self.dependency_index.insert(gav, index);
        }
    }

    /// Add an exclusion pattern
    pub fn add_exclusion(&mut self, pattern: impl Into<String>) {
        self.exclusions.insert(pattern.into());
    }

    /// Check if a dependency is excluded
    pub fn is_excluded(&self, group_id: &str, artifact_id: &str) -> bool {
        let key = format!("{group_id}:{artifact_id}");
        self.exclusions.contains(&key)
            || self.exclusions.contains(&format!("{group_id}:*"))
            || self.exclusions.contains("*:*")
    }

    /// Set scope filter
    pub fn with_scope_filter(mut self, scopes: HashSet<DependencyScope>) -> Self {
        self.scope_filter = Some(scopes);
        self
    }

    /// Get all dependencies
    pub fn all(&self) -> &[ResolvedDependency] {
        &self.dependencies
    }

    /// Get dependencies for compile classpath
    pub fn compile_classpath(&self) -> Vec<&ResolvedDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.scope.in_compile_classpath())
            .collect()
    }

    /// Get dependencies for runtime classpath
    pub fn runtime_classpath(&self) -> Vec<&ResolvedDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.scope.in_runtime_classpath())
            .collect()
    }

    /// Get dependencies for test classpath
    pub fn test_classpath(&self) -> Vec<&ResolvedDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.scope.in_test_classpath())
            .collect()
    }

    /// Get classpath as a list of paths
    pub fn classpath_paths(&self, include_test: bool) -> Vec<PathBuf> {
        let deps = if include_test {
            self.test_classpath()
        } else {
            self.compile_classpath()
        };

        deps.iter().filter_map(|d| d.file.clone()).collect()
    }

    /// Get classpath as a string
    pub fn classpath_string(&self, include_test: bool) -> String {
        let paths = self.classpath_paths(include_test);
        let separator = if cfg!(windows) { ";" } else { ":" };
        paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Get a dependency by GAV
    pub fn get(&self, gav: &str) -> Option<&ResolvedDependency> {
        self.dependency_index
            .get(gav)
            .map(|&i| &self.dependencies[i])
    }

    /// Check if a dependency exists
    pub fn contains(&self, gav: &str) -> bool {
        self.dependency_index.contains_key(gav)
    }

    /// Get the number of dependencies
    pub fn len(&self) -> usize {
        self.dependencies.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_scope() {
        assert_eq!(
            DependencyScope::from_str("compile"),
            Ok(DependencyScope::Compile)
        );
        assert_eq!(DependencyScope::from_str("TEST"), Ok(DependencyScope::Test));
        assert_eq!(DependencyScope::Compile.as_str(), "compile");
    }

    #[test]
    fn test_scope_classpath_inclusion() {
        assert!(DependencyScope::Compile.in_compile_classpath());
        assert!(DependencyScope::Compile.in_runtime_classpath());
        assert!(DependencyScope::Compile.in_test_classpath());

        assert!(DependencyScope::Provided.in_compile_classpath());
        assert!(!DependencyScope::Provided.in_runtime_classpath());

        assert!(!DependencyScope::Test.in_compile_classpath());
        assert!(!DependencyScope::Test.in_runtime_classpath());
        assert!(DependencyScope::Test.in_test_classpath());
    }

    #[test]
    fn test_resolved_dependency() {
        let dep = ResolvedDependency::new("com.example", "lib", "1.0.0")
            .with_scope(DependencyScope::Test)
            .with_file(PathBuf::from("/repo/lib-1.0.0.jar"));

        assert_eq!(dep.gav(), "com.example:lib:1.0.0");
        assert_eq!(dep.scope, DependencyScope::Test);
        assert!(dep.file.is_some());
    }

    #[test]
    fn test_dependency_context() {
        let mut ctx = DependencyContext::new();

        ctx.add(
            ResolvedDependency::new("g", "a", "1.0")
                .with_scope(DependencyScope::Compile)
                .with_file(PathBuf::from("/a.jar")),
        );

        ctx.add(
            ResolvedDependency::new("g", "b", "1.0")
                .with_scope(DependencyScope::Test)
                .with_file(PathBuf::from("/b.jar")),
        );

        assert_eq!(ctx.len(), 2);
        assert_eq!(ctx.compile_classpath().len(), 1);
        assert_eq!(ctx.test_classpath().len(), 2);
    }

    #[test]
    fn test_exclusions() {
        let mut ctx = DependencyContext::new();
        ctx.add_exclusion("com.example:excluded");

        assert!(ctx.is_excluded("com.example", "excluded"));
        assert!(!ctx.is_excluded("com.example", "included"));
    }

    #[test]
    fn test_classpath_string() {
        let mut ctx = DependencyContext::new();
        ctx.add(ResolvedDependency::new("g", "a", "1.0").with_file(PathBuf::from("/a.jar")));
        ctx.add(ResolvedDependency::new("g", "b", "1.0").with_file(PathBuf::from("/b.jar")));

        let cp = ctx.classpath_string(false);
        assert!(cp.contains("/a.jar"));
        assert!(cp.contains("/b.jar"));
    }
}
