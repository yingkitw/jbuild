//! Kotlin language support

use std::path::Path;

/// Kotlin compiler information
#[derive(Debug, Clone)]
pub struct KotlinCompiler {
    pub version: String,
    pub api_version: Option<String>,
    pub language_version: Option<String>,
}

impl KotlinCompiler {
    pub fn new(version: String) -> Self {
        Self {
            version,
            api_version: None,
            language_version: None,
        }
    }

    pub fn with_api_version(mut self, version: String) -> Self {
        self.api_version = Some(version);
        self
    }

    pub fn with_language_version(mut self, version: String) -> Self {
        self.language_version = Some(version);
        self
    }
}

/// Check if a file is a Kotlin source file
pub fn is_kotlin_source(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "kt" || s == "kts")
        .unwrap_or(false)
}

/// Get Kotlin script file extension
pub fn is_kotlin_script(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "kts")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_kotlin_source() {
        assert!(is_kotlin_source(Path::new("Test.kt")));
        assert!(is_kotlin_source(Path::new("Test.kts")));
        assert!(!is_kotlin_source(Path::new("Test.java")));
    }

    #[test]
    fn test_kotlin_compiler() {
        let compiler = KotlinCompiler::new("1.9.0".to_string());
        assert_eq!(compiler.version, "1.9.0");
    }
}
