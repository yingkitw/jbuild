use std::path::PathBuf;
use std::fmt;

/// Classpath builder for Java compilation
#[derive(Debug, Clone)]
pub struct ClasspathBuilder {
    entries: Vec<PathBuf>,
}

impl ClasspathBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a classpath entry
    pub fn add_entry(mut self, entry: PathBuf) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add multiple classpath entries
    pub fn add_entries(mut self, entries: Vec<PathBuf>) -> Self {
        self.entries.extend(entries);
        self
    }

    /// Add a JAR file to the classpath
    pub fn add_jar(mut self, jar_path: PathBuf) -> Self {
        self.entries.push(jar_path);
        self
    }

    /// Add a directory to the classpath
    pub fn add_directory(mut self, dir_path: PathBuf) -> Self {
        self.entries.push(dir_path);
        self
    }

    /// Build the classpath string (platform-specific separator)
    pub fn build(&self) -> String {
        let separator = if cfg!(windows) { ";" } else { ":" };
        self.entries
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Get the classpath entries
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Check if classpath is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ClasspathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ClasspathBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build())
    }
}

