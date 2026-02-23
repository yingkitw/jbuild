//! Language detection for source files

use std::path::Path;
use super::{java, kotlin, scala, groovy};

/// Detected programming language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Java,
    Kotlin,
    Scala,
    Groovy,
    Unknown,
}

/// Detect the language of a source file
pub fn detect_language(path: &Path) -> Language {
    if java::is_java_source(path) {
        Language::Java
    } else if kotlin::is_kotlin_source(path) {
        Language::Kotlin
    } else if scala::is_scala_source(path) {
        Language::Scala
    } else if groovy::is_groovy_source(path) {
        Language::Groovy
    } else {
        Language::Unknown
    }
}

/// Get source file extensions for a language
pub fn get_source_extensions(language: Language) -> Vec<&'static str> {
    match language {
        Language::Java => vec!["java"],
        Language::Kotlin => vec!["kt", "kts"],
        Language::Scala => vec!["scala"],
        Language::Groovy => vec!["groovy", "gvy", "gy", "gsh"],
        Language::Unknown => vec![],
    }
}

/// Check if a file is a compilable source
pub fn is_source_file(path: &Path) -> bool {
    detect_language(path) != Language::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(Path::new("Test.java")), Language::Java);
        assert_eq!(detect_language(Path::new("Test.kt")), Language::Kotlin);
        assert_eq!(detect_language(Path::new("Test.scala")), Language::Scala);
        assert_eq!(detect_language(Path::new("Test.groovy")), Language::Groovy);
        assert_eq!(detect_language(Path::new("Test.txt")), Language::Unknown);
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("Test.java")));
        assert!(!is_source_file(Path::new("Test.txt")));
    }
}
