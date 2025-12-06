//! Gradle build script parser
//! 
//! Parses build.gradle (Groovy DSL) and build.gradle.kts (Kotlin DSL) files.

use std::path::Path;
use anyhow::{Context, Result};
use crate::gradle::model::{GradleProject, Task, Dependency, Repository, RepositoryType, Plugin};

/// Parse a Gradle build script
pub fn parse_gradle_build_script(
    build_file: &Path,
    base_dir: &Path,
) -> Result<GradleProject> {
    let content = std::fs::read_to_string(build_file)
        .with_context(|| format!("Failed to read build file: {:?}", build_file))?;

    // Determine if it's Kotlin DSL or Groovy DSL
    let is_kotlin = build_file.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "kts")
        .unwrap_or(false);

    if is_kotlin {
        parse_kotlin_dsl(&content, build_file, base_dir)
    } else {
        parse_groovy_dsl(&content, build_file, base_dir)
    }
}

/// Parse Groovy DSL build script
fn parse_groovy_dsl(
    content: &str,
    build_file: &Path,
    base_dir: &Path,
) -> Result<GradleProject> {
    let mut project = GradleProject::new(base_dir.to_path_buf(), build_file.to_path_buf());

    // Basic parsing - extract key information
    // This is a simplified parser. A full implementation would need a Groovy parser.
    
    // Extract plugins
    if let Some(plugins_block) = extract_block(content, "plugins") {
        project.plugins = parse_plugins(&plugins_block);
    }

    // Extract group
    if let Some(group) = extract_string_property(content, "group") {
        project.group = Some(group);
    }

    // Extract version
    if let Some(version) = extract_string_property(content, "version") {
        project.version = Some(version);
    }

    // Extract name
    if let Some(name) = extract_string_property(content, "name") {
        project.name = name;
    }

    // Extract sourceCompatibility
    if let Some(compat) = extract_string_property(content, "sourceCompatibility") {
        project.source_compatibility = Some(compat);
    }

    // Extract targetCompatibility
    if let Some(compat) = extract_string_property(content, "targetCompatibility") {
        project.target_compatibility = Some(compat);
    }

    // Extract repositories
    if let Some(repos_block) = extract_block(content, "repositories") {
        project.repositories = parse_repositories(&repos_block);
    }

    // Extract dependencies
    if let Some(deps_block) = extract_block(content, "dependencies") {
        project.dependencies = parse_dependencies(&deps_block);
    }

    // Extract tasks (basic - just task names for now)
    project.tasks = parse_tasks(content);

    Ok(project)
}

/// Parse Kotlin DSL build script
fn parse_kotlin_dsl(
    content: &str,
    build_file: &Path,
    base_dir: &Path,
) -> Result<GradleProject> {
    // Kotlin DSL parsing is similar but with different syntax
    // For now, we'll use a similar approach
    parse_groovy_dsl(content, build_file, base_dir)
}

/// Extract a block from the build script
fn extract_block(content: &str, block_name: &str) -> Option<String> {
    // Look for "block_name {" pattern
    let pattern = format!("{} {{", block_name);
    if let Some(start) = content.find(&pattern) {
        let start_pos = start + pattern.len();
        let mut depth = 1;
        let mut pos = start_pos;
        let chars: Vec<char> = content[start_pos..].chars().collect();
        
        for (i, ch) in chars.iter().enumerate() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(content[start_pos..start_pos + i].to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Extract a string property value
fn extract_string_property(content: &str, property: &str) -> Option<String> {
    // Look for "property = 'value'" or "property = \"value\"" pattern
    let patterns = vec![
        format!("{} = '", property),
        format!("{} = \"", property),
        format!("{}='", property),
        format!("{}=\"", property),
    ];

    for pattern in patterns {
        if let Some(start) = content.find(&pattern) {
            let start_pos = start + pattern.len();
            let quote_char = pattern.chars().last().unwrap();
            if let Some(end) = content[start_pos..].find(quote_char) {
                return Some(content[start_pos..start_pos + end].to_string());
            }
        }
    }
    None
}

/// Parse plugins from a plugins block
fn parse_plugins(plugins_block: &str) -> Vec<Plugin> {
    let mut plugins = Vec::new();
    
    // Look for "id 'plugin-id'" or "id(\"plugin-id\")" patterns
    let id_patterns = vec!["id '", "id \"", "id('", "id(\""];
    
    for line in plugins_block.lines() {
        for pattern in &id_patterns {
            if let Some(start) = line.find(pattern) {
                let start_pos = start + pattern.len();
                let quote_char = if pattern.contains('\'') { '\'' } else { '"' };
                if let Some(end) = line[start_pos..].find(quote_char) {
                    let plugin_id = line[start_pos..start_pos + end].to_string();
                    plugins.push(Plugin {
                        id: plugin_id,
                        version: None, // Version parsing would need more sophisticated parsing
                    });
                }
            }
        }
    }
    
    plugins
}

/// Parse repositories from a repositories block
fn parse_repositories(repos_block: &str) -> Vec<Repository> {
    let mut repositories = Vec::new();
    
    // Look for common repository declarations
    if repos_block.contains("mavenCentral()") {
        repositories.push(Repository {
            name: "MavenCentral".to_string(),
            repo_type: RepositoryType::MavenCentral,
            url: Some("https://repo1.maven.org/maven2/".to_string()),
        });
    }
    
    if repos_block.contains("jcenter()") {
        repositories.push(Repository {
            name: "JCenter".to_string(),
            repo_type: RepositoryType::JCenter,
            url: Some("https://jcenter.bintray.com/".to_string()),
        });
    }
    
    if repos_block.contains("google()") {
        repositories.push(Repository {
            name: "Google".to_string(),
            repo_type: RepositoryType::Google,
            url: Some("https://dl.google.com/dl/android/maven2/".to_string()),
        });
    }
    
    // Parse maven { url "..." } blocks
    if let Some(maven_block) = extract_block(repos_block, "maven") {
        if let Some(url) = extract_string_property(&maven_block, "url") {
            repositories.push(Repository {
                name: "Maven".to_string(),
                repo_type: RepositoryType::Maven,
                url: Some(url),
            });
        }
    }
    
    repositories
}

/// Parse dependencies from a dependencies block
fn parse_dependencies(deps_block: &str) -> Vec<Dependency> {
    let mut dependencies = Vec::new();
    
    // Look for dependency declarations like:
    // implementation 'group:artifact:version'
    // testImplementation "group:artifact:version"
    let dependency_patterns = vec![
        "implementation", "compile", "runtime", "testImplementation",
        "testCompile", "testRuntime", "api", "compileOnly", "runtimeOnly",
    ];
    
    for line in deps_block.lines() {
        for config in &dependency_patterns {
            let pattern = format!("{} ", config);
            if let Some(start) = line.find(&pattern) {
                let dep_start = start + pattern.len();
                let dep_str = line[dep_start..].trim();
                
                // Extract dependency notation (e.g., 'group:artifact:version')
                let notation = dep_str
                    .trim_matches(|c: char| c == '\'' || c == '"')
                    .to_string();
                
                // Parse group:artifact:version
                let parts: Vec<&str> = notation.split(':').collect();
                if parts.len() >= 2 {
                    dependencies.push(Dependency {
                        configuration: config.to_string(),
                        notation: notation.clone(),
                        group: Some(parts[0].to_string()),
                        artifact: Some(parts[1].to_string()),
                        version: parts.get(2).map(|s| s.to_string()),
                        classifier: None,
                        extension: None,
                    });
                } else {
                    // Could be a project dependency or file dependency
                    dependencies.push(Dependency {
                        configuration: config.to_string(),
                        notation,
                        group: None,
                        artifact: None,
                        version: None,
                        classifier: None,
                        extension: None,
                    });
                }
            }
        }
    }
    
    dependencies
}

/// Parse tasks from the build script
fn parse_tasks(content: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    
    // Look for task declarations like:
    // task taskName(type: TaskType) { ... }
    // task taskName { ... }
    let task_pattern = "task ";
    let mut pos = 0;
    
    while let Some(start) = content[pos..].find(task_pattern) {
        let task_start = pos + start + task_pattern.len();
        let remaining = &content[task_start..];
        
        // Find task name (until space, '(', or '{')
        let name_end = remaining
            .find(|c: char| c == ' ' || c == '(' || c == '{')
            .unwrap_or(remaining.len());
        
        let task_name = remaining[..name_end].trim().to_string();
        
        if !task_name.is_empty() {
            tasks.push(Task {
                name: task_name,
                task_type: None, // Would need more parsing to extract type
                description: None,
                group: None,
                depends_on: Vec::new(),
                actions: Vec::new(),
            });
        }
        
        pos = task_start + name_end;
    }
    
    // Add standard tasks if Java plugin is present
    if content.contains("java") || content.contains("id 'java'") || content.contains("id(\"java\")") {
        let standard_tasks = vec![
            "compileJava", "processResources", "classes", "jar",
            "compileTestJava", "processTestResources", "testClasses", "test",
            "clean", "build", "assemble", "check",
        ];
        
        for task_name in standard_tasks {
            if !tasks.iter().any(|t| t.name == task_name) {
                tasks.push(Task {
                    name: task_name.to_string(),
                    task_type: Some("Standard".to_string()),
                    description: None,
                    group: Some("build".to_string()),
                    depends_on: Vec::new(),
                    actions: Vec::new(),
                });
            }
        }
    }
    
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_simple_gradle_build() {
        let content = r#"
plugins {
    id 'java'
}

group = 'com.example'
version = '1.0.0'

repositories {
    mavenCentral()
}

dependencies {
    implementation 'org.slf4j:slf4j-api:1.7.36'
    testImplementation 'junit:junit:4.13.2'
}
"#;
        
        let build_file = PathBuf::from("build.gradle");
        let base_dir = PathBuf::from("/test");
        
        let project = parse_groovy_dsl(content, &build_file, &base_dir).unwrap();
        
        assert_eq!(project.plugins.len(), 1);
        assert_eq!(project.plugins[0].id, "java");
        assert_eq!(project.group, Some("com.example".to_string()));
        assert_eq!(project.version, Some("1.0.0".to_string()));
        assert!(project.repositories.len() > 0);
        assert!(project.dependencies.len() >= 2);
    }
}

