//! Java language support

use std::path::Path;

/// Java compiler information
#[derive(Debug, Clone)]
pub struct JavaCompiler {
    pub version: u32,
    pub source_version: Option<String>,
    pub target_version: Option<String>,
}

impl JavaCompiler {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            source_version: None,
            target_version: None,
        }
    }

    pub fn with_source_version(mut self, version: String) -> Self {
        self.source_version = Some(version);
        self
    }

    pub fn with_target_version(mut self, version: String) -> Self {
        self.target_version = Some(version);
        self
    }

    /// Get compiler arguments
    pub fn get_compiler_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(source) = &self.source_version {
            args.push("-source".to_string());
            args.push(source.clone());
        }

        if let Some(target) = &self.target_version {
            args.push("-target".to_string());
            args.push(target.clone());
        }

        args
    }
}

/// Check if a file is a Java source file
pub fn is_java_source(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "java")
        .unwrap_or(false)
}

/// Get Java package from file path
pub fn get_package_from_path(path: &Path, source_root: &Path) -> Option<String> {
    path.strip_prefix(source_root)
        .ok()?
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('/', "."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_java_source() {
        assert!(is_java_source(Path::new("Test.java")));
        assert!(!is_java_source(Path::new("Test.kt")));
    }

    #[test]
    fn test_java_compiler() {
        let compiler = JavaCompiler::new(17)
            .with_source_version("17".to_string())
            .with_target_version("17".to_string());

        assert_eq!(compiler.version, 17);
        assert_eq!(compiler.get_compiler_args().len(), 4);
    }
}
