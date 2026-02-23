//! Groovy language support

use std::path::Path;

/// Groovy compiler information
#[derive(Debug, Clone)]
pub struct GroovyCompiler {
    pub version: String,
}

impl GroovyCompiler {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

/// Check if a file is a Groovy source file
pub fn is_groovy_source(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "groovy" || s == "gvy" || s == "gy" || s == "gsh")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_groovy_source() {
        assert!(is_groovy_source(Path::new("Test.groovy")));
        assert!(is_groovy_source(Path::new("Test.gvy")));
        assert!(!is_groovy_source(Path::new("Test.java")));
    }
}
