//! File-related types for Checkstyle-rs

use std::path::PathBuf;

/// Represents file text content
#[derive(Debug, Clone)]
pub struct FileText {
    /// Path to the file
    pub path: PathBuf,
    /// Full text content of the file
    pub full_text: String,
    /// Lines of the file
    pub lines: Vec<String>,
}

impl FileText {
    /// Create a new FileText from a path and content
    pub fn new(path: PathBuf, content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            path,
            full_text: content,
            lines,
        }
    }

    /// Get the full text as a string
    pub fn get_full_text(&self) -> &str {
        &self.full_text
    }

    /// Get a specific line (1-based)
    pub fn get_line(&self, line_no: usize) -> Option<&str> {
        if line_no > 0 && line_no <= self.lines.len() {
            Some(&self.lines[line_no - 1])
        } else {
            None
        }
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// Represents file contents with additional metadata
#[derive(Debug, Clone)]
pub struct FileContents {
    /// File text
    pub file_text: FileText,
    /// File name
    pub file_name: String,
}

impl FileContents {
    /// Create a new FileContents
    pub fn new(file_text: FileText) -> Self {
        let file_name = file_text
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        Self {
            file_text,
            file_name,
        }
    }

    /// Get the file text
    pub fn get_text(&self) -> &FileText {
        &self.file_text
    }
}
