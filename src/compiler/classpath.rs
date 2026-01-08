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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classpath_builder_new() {
        let builder = ClasspathBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.entries().len(), 0);
    }

    #[test]
    fn test_classpath_builder_add_entry() {
        let builder = ClasspathBuilder::new()
            .add_entry(PathBuf::from("/path/to/classes"));
        
        assert!(!builder.is_empty());
        assert_eq!(builder.entries().len(), 1);
    }

    #[test]
    fn test_classpath_builder_add_jar() {
        let builder = ClasspathBuilder::new()
            .add_jar(PathBuf::from("/path/to/lib.jar"));
        
        assert_eq!(builder.entries().len(), 1);
    }

    #[test]
    fn test_classpath_builder_add_directory() {
        let builder = ClasspathBuilder::new()
            .add_directory(PathBuf::from("/path/to/classes"));
        
        assert_eq!(builder.entries().len(), 1);
    }

    #[test]
    fn test_classpath_builder_add_entries() {
        let builder = ClasspathBuilder::new()
            .add_entries(vec![
                PathBuf::from("/path/to/classes"),
                PathBuf::from("/path/to/lib.jar"),
            ]);
        
        assert_eq!(builder.entries().len(), 2);
    }

    #[test]
    fn test_classpath_builder_build() {
        let builder = ClasspathBuilder::new()
            .add_entry(PathBuf::from("/path/to/classes"))
            .add_jar(PathBuf::from("/path/to/lib.jar"));
        
        let classpath = builder.build();
        let separator = if cfg!(windows) { ";" } else { ":" };
        assert!(classpath.contains(separator));
        assert!(classpath.contains("/path/to/classes"));
        assert!(classpath.contains("/path/to/lib.jar"));
    }

    #[test]
    fn test_classpath_builder_display() {
        let builder = ClasspathBuilder::new()
            .add_entry(PathBuf::from("/path/to/classes"));
        
        let display = format!("{builder}");
        assert!(!display.is_empty());
    }

    #[test]
    fn test_classpath_builder_chaining() {
        let builder = ClasspathBuilder::new()
            .add_entry(PathBuf::from("/path1"))
            .add_jar(PathBuf::from("/path2.jar"))
            .add_directory(PathBuf::from("/path3"));
        
        assert_eq!(builder.entries().len(), 3);
    }
}

