//! Unit of Work abstraction
//!
//! Inspired by Gradle's execution engine, this module provides a unified abstraction
//! for defining and executing units of work with well-defined inputs and outputs.
//!
//! A unit of work is an identifiable action with known inputs and expected outputs.
//! The action is a pure function with regard to its inputs - it produces equivalent
//! outputs for equivalent inputs.

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

/// Identity of a unit of work
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkIdentity {
    /// Unique identifier for this work unit
    pub id: String,
    /// Type of work (e.g., "compile", "test", "package")
    pub work_type: String,
}

impl WorkIdentity {
    pub fn new(id: impl Into<String>, work_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            work_type: work_type.into(),
        }
    }
}

/// Input fingerprint for caching and up-to-date checks
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFingerprint {
    /// Hash of scalar inputs (properties, configurations)
    pub scalar_hash: String,
    /// Hash of file inputs
    pub file_hash: String,
}

impl InputFingerprint {
    pub fn new(scalar_hash: impl Into<String>, file_hash: impl Into<String>) -> Self {
        Self {
            scalar_hash: scalar_hash.into(),
            file_hash: file_hash.into(),
        }
    }

    /// Compute a combined hash
    pub fn combined_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        self.scalar_hash.hash(&mut hasher);
        self.file_hash.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Output of work execution
#[derive(Debug, Clone)]
pub struct WorkOutput {
    /// Whether the work executed successfully
    pub success: bool,
    /// Output files produced
    pub output_files: Vec<PathBuf>,
    /// Any error messages
    pub errors: Vec<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl WorkOutput {
    pub fn success(output_files: Vec<PathBuf>, duration_ms: u64) -> Self {
        Self {
            success: true,
            output_files,
            errors: Vec::new(),
            duration_ms,
        }
    }

    pub fn failure(errors: Vec<String>, duration_ms: u64) -> Self {
        Self {
            success: false,
            output_files: Vec::new(),
            errors,
            duration_ms,
        }
    }
}

/// Execution context passed to work units
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Base directory for the work
    pub base_directory: PathBuf,
    /// Workspace directory for outputs
    pub workspace: PathBuf,
    /// Properties/configuration
    pub properties: HashMap<String, String>,
    /// Whether to run in incremental mode
    pub incremental: bool,
}

impl ExecutionContext {
    pub fn new(base_directory: PathBuf, workspace: PathBuf) -> Self {
        Self {
            base_directory,
            workspace,
            properties: HashMap::new(),
            incremental: false,
        }
    }

    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }
}

/// Trait for units of work
///
/// Inspired by Gradle's UnitOfWork interface, this trait defines the contract
/// for executable work units with well-defined inputs and outputs.
pub trait UnitOfWork: Send + Sync {
    /// Get the identity of this work unit
    fn identify(&self) -> WorkIdentity;

    /// Get a human-readable description
    fn description(&self) -> String;

    /// Execute the work
    fn execute(&self, context: &ExecutionContext) -> Result<WorkOutput>;

    /// Visit immutable inputs (used for identity calculation)
    fn visit_immutable_inputs(&self, visitor: &mut dyn InputVisitor) {
        let _ = visitor;
    }

    /// Visit mutable inputs (used for up-to-date checks)
    fn visit_mutable_inputs(&self, visitor: &mut dyn InputVisitor) {
        let _ = visitor;
    }

    /// Visit outputs
    fn visit_outputs(&self, visitor: &mut dyn OutputVisitor) {
        let _ = visitor;
    }

    /// Check if caching should be disabled
    fn should_disable_caching(&self) -> Option<String> {
        None
    }

    /// Get timeout for this work unit
    fn timeout(&self) -> Option<std::time::Duration> {
        None
    }
}

/// Visitor for inputs
pub trait InputVisitor {
    /// Visit a scalar input property
    fn visit_property(&mut self, name: &str, value: &str);

    /// Visit a file input
    fn visit_file(&mut self, name: &str, path: &PathBuf);

    /// Visit a directory input
    fn visit_directory(&mut self, name: &str, path: &PathBuf);
}

/// Visitor for outputs
pub trait OutputVisitor {
    /// Visit a file output
    fn visit_file(&mut self, name: &str, path: &PathBuf);

    /// Visit a directory output
    fn visit_directory(&mut self, name: &str, path: &PathBuf);
}

/// Default input visitor that collects inputs for fingerprinting
pub struct FingerprintingInputVisitor {
    properties: HashMap<String, String>,
    files: Vec<(String, PathBuf)>,
}

impl FingerprintingInputVisitor {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            files: Vec::new(),
        }
    }

    pub fn compute_fingerprint(&self) -> InputFingerprint {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        // Hash scalar properties
        let mut scalar_hasher = DefaultHasher::new();
        let mut sorted_props: Vec<_> = self.properties.iter().collect();
        sorted_props.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_props {
            k.hash(&mut scalar_hasher);
            v.hash(&mut scalar_hasher);
        }

        // Hash file paths (in production, would hash file contents)
        let mut file_hasher = DefaultHasher::new();
        let mut sorted_files: Vec<_> = self.files.iter().collect();
        sorted_files.sort_by_key(|(k, _)| k.clone());
        for (name, path) in sorted_files {
            name.hash(&mut file_hasher);
            path.hash(&mut file_hasher);
        }

        InputFingerprint::new(
            format!("{:x}", scalar_hasher.finish()),
            format!("{:x}", file_hasher.finish()),
        )
    }
}

impl Default for FingerprintingInputVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl InputVisitor for FingerprintingInputVisitor {
    fn visit_property(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), value.to_string());
    }

    fn visit_file(&mut self, name: &str, path: &PathBuf) {
        self.files.push((name.to_string(), path.clone()));
    }

    fn visit_directory(&mut self, name: &str, path: &PathBuf) {
        self.files.push((name.to_string(), path.clone()));
    }
}

/// Default output visitor that collects outputs
pub struct CollectingOutputVisitor {
    pub files: Vec<(String, PathBuf)>,
    pub directories: Vec<(String, PathBuf)>,
}

impl CollectingOutputVisitor {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            directories: Vec::new(),
        }
    }
}

impl Default for CollectingOutputVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputVisitor for CollectingOutputVisitor {
    fn visit_file(&mut self, name: &str, path: &PathBuf) {
        self.files.push((name.to_string(), path.clone()));
    }

    fn visit_directory(&mut self, name: &str, path: &PathBuf) {
        self.directories.push((name.to_string(), path.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWork {
        id: String,
    }

    impl UnitOfWork for TestWork {
        fn identify(&self) -> WorkIdentity {
            WorkIdentity::new(&self.id, "test")
        }

        fn description(&self) -> String {
            format!("Test work: {}", self.id)
        }

        fn execute(&self, _context: &ExecutionContext) -> Result<WorkOutput> {
            Ok(WorkOutput::success(vec![], 100))
        }

        fn visit_immutable_inputs(&self, visitor: &mut dyn InputVisitor) {
            visitor.visit_property("id", &self.id);
        }
    }

    #[test]
    fn test_work_identity() {
        let identity = WorkIdentity::new("compile-main", "compile");
        assert_eq!(identity.id, "compile-main");
        assert_eq!(identity.work_type, "compile");
    }

    #[test]
    fn test_input_fingerprint() {
        let fp = InputFingerprint::new("abc123", "def456");
        assert!(!fp.combined_hash().is_empty());
    }

    #[test]
    fn test_work_output_success() {
        let output = WorkOutput::success(vec![PathBuf::from("out.jar")], 500);
        assert!(output.success);
        assert_eq!(output.output_files.len(), 1);
        assert!(output.errors.is_empty());
    }

    #[test]
    fn test_work_output_failure() {
        let output = WorkOutput::failure(vec!["Compilation failed".to_string()], 100);
        assert!(!output.success);
        assert!(output.output_files.is_empty());
        assert_eq!(output.errors.len(), 1);
    }

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::new(
            PathBuf::from("/project"),
            PathBuf::from("/project/build"),
        )
        .with_incremental(true);

        assert!(ctx.incremental);
        assert_eq!(ctx.base_directory, PathBuf::from("/project"));
    }

    #[test]
    fn test_unit_of_work_trait() {
        let work = TestWork { id: "test-1".to_string() };
        let identity = work.identify();
        assert_eq!(identity.id, "test-1");
        assert_eq!(identity.work_type, "test");
    }

    #[test]
    fn test_fingerprinting_visitor() {
        let mut visitor = FingerprintingInputVisitor::new();
        visitor.visit_property("version", "1.0.0");
        visitor.visit_file("source", &PathBuf::from("src/main.java"));

        let fingerprint = visitor.compute_fingerprint();
        assert!(!fingerprint.scalar_hash.is_empty());
        assert!(!fingerprint.file_hash.is_empty());
    }

    #[test]
    fn test_collecting_output_visitor() {
        let mut visitor = CollectingOutputVisitor::new();
        visitor.visit_file("jar", &PathBuf::from("build/app.jar"));
        visitor.visit_directory("classes", &PathBuf::from("build/classes"));

        assert_eq!(visitor.files.len(), 1);
        assert_eq!(visitor.directories.len(), 1);
    }
}
