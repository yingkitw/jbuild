//! Main class detection from Java source files

use std::path::Path;
use anyhow::{Result, Context};
use walkdir::WalkDir;

/// Detect main class from Java source files
pub fn detect_main_class(base_dir: &Path) -> Result<Option<String>> {
    // Try Maven structure first
    let maven_src = base_dir.join("src/main/java");
    if maven_src.exists() {
        if let Some(main_class) = find_main_class_in_dir(&maven_src)? {
            return Ok(Some(main_class));
        }
    }

    // Try Gradle structure
    let gradle_src = base_dir.join("src/main/java");
    if gradle_src.exists() {
        if let Some(main_class) = find_main_class_in_dir(&gradle_src)? {
            return Ok(Some(main_class));
        }
    }

    Ok(None)
}

/// Find main class in a directory by scanning Java files
fn find_main_class_in_dir(src_dir: &Path) -> Result<Option<String>> {
    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "java")
                .unwrap_or(false)
        })
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            // Check for main method
            if content.contains("public static void main") {
                // Extract package name
                let package = extract_package_name(&content);
                
                // Extract class name from file path
                let relative = path.strip_prefix(src_dir)
                    .context("Failed to get relative path")?;
                let class_name = relative
                    .to_string_lossy()
                    .replace(['/', '\\'], ".")
                    .trim_end_matches(".java")
                    .to_string();

                // Combine package and class name
                let full_class_name = if let Some(pkg) = package {
                    format!("{pkg}.{class_name}")
                } else {
                    class_name
                };

                return Ok(Some(full_class_name));
            }
        }
    }

    Ok(None)
}

/// Extract package name from Java source content
fn extract_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("package ") && trimmed.ends_with(';') {
            let pkg = trimmed
                .strip_prefix("package ")?
                .strip_suffix(';')?
                .trim();
            return Some(pkg.to_string());
        }
    }
    None
}

/// Extract main class from POM or build.gradle configuration
pub fn extract_main_class_from_config(base_dir: &Path) -> Result<Option<String>> {
    // Try Maven POM first
    let pom_path = base_dir.join("pom.xml");
    if pom_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pom_path) {
            // Look for mainClass in configuration
            if let Some(start) = content.find("<mainClass>") {
                if let Some(end) = content[start..].find("</mainClass>") {
                    let main_class = content[start + 11..start + end].trim();
                    if !main_class.is_empty() {
                        return Ok(Some(main_class.to_string()));
                    }
                }
            }
        }
    }

    // Try Gradle build.gradle
    let gradle_path = base_dir.join("build.gradle");
    if gradle_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&gradle_path) {
            // Look for mainClass in application block
            if let Some(start) = content.find("mainClass") {
                // Try to extract from mainClass = '...' or mainClass = "..."
                let remaining = &content[start..];
                if let Some(quote_start) = remaining.find('=') {
                    let after_eq = &remaining[quote_start + 1..].trim();
                    if let Some(quote_char) = after_eq.chars().next() {
                        if quote_char == '\'' || quote_char == '"' {
                            if let Some(quote_end) = after_eq[1..].find(quote_char) {
                                let main_class = after_eq[1..quote_end + 1].trim();
                                if !main_class.is_empty() {
                                    return Ok(Some(main_class.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

