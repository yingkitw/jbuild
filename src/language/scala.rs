//! Scala language support

use std::path::Path;

/// Scala compiler information
#[derive(Debug, Clone)]
pub struct ScalaCompiler {
    pub version: String,
    pub scala_version: Option<String>,
}

impl ScalaCompiler {
    pub fn new(version: String) -> Self {
        Self {
            version,
            scala_version: None,
        }
    }

    pub fn with_scala_version(mut self, version: String) -> Self {
        self.scala_version = Some(version);
        self
    }
}

/// Check if a file is a Scala source file
pub fn is_scala_source(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "scala")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_scala_source() {
        assert!(is_scala_source(Path::new("Test.scala")));
        assert!(!is_scala_source(Path::new("Test.java")));
    }
}
