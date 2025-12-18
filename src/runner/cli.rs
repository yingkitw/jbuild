use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs;
use anyhow::Result;
use walkdir::WalkDir;

use crate::build::{BuildSystem, BuildExecutor, ExecutionRequest};
use crate::maven::core::MavenBuildExecutor;
use crate::gradle::core::GradleExecutor;
use crate::ui::{info as ui_info, success as ui_success, error as ui_error, warn as ui_warn};
use crate::runner::{fetch_latest_version, fetch_package_info};
use crate::artifact::repository::DefaultLocalRepository;
use crate::resolver::{DependencyResolver, RemoteRepository, downloader::ArtifactDownloader};
use crate::model::{Dependency as MavenDep, parser::parse_pom};
use crate::config::Workspace;

/// Create a new Java project
pub fn run_new(name: &str, template: &str, build_system: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);
    
    if project_dir.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists", name));
    }
    
    println!("[INFO] Creating new {} project '{}'", template, name);
    
    // Create directory structure
    fs::create_dir_all(project_dir.join("src/main/java/com/example"))?;
    fs::create_dir_all(project_dir.join("src/test/java/com/example"))?;
    fs::create_dir_all(project_dir.join("src/main/resources"))?;
    fs::create_dir_all(project_dir.join("src/test/resources"))?;
    
    // Generate package name from project name
    let package_name = name.replace('-', "_").replace('.', "_").to_lowercase();
    let class_name = to_pascal_case(name);
    
    // Create main Java file
    let main_java = match template {
        "lib" => format!(
            r#"package com.example.{package_name};

/**
 * Library class for {name}.
 */
public class {class_name} {{
    
    /**
     * Returns a greeting message.
     * @param name the name to greet
     * @return the greeting message
     */
    public String greet(String name) {{
        return "Hello, " + name + "!";
    }}
}}
"#,
            package_name = package_name,
            name = name,
            class_name = class_name
        ),
        _ => format!(
            r#"package com.example.{package_name};

/**
 * Main application class for {name}.
 */
public class {class_name} {{
    
    public static void main(String[] args) {{
        System.out.println("Hello from {name}!");
    }}
}}
"#,
            package_name = package_name,
            name = name,
            class_name = class_name
        ),
    };
    
    fs::write(
        project_dir.join(format!("src/main/java/com/example/{}.java", class_name)),
        main_java,
    )?;
    
    // Create test Java file
    let test_java = format!(
        r#"package com.example.{package_name};

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Test class for {class_name}.
 */
public class {class_name}Test {{
    
    @Test
    void testExample() {{
        assertTrue(true, "Example test");
    }}
}}
"#,
        package_name = package_name,
        class_name = class_name
    );
    
    fs::write(
        project_dir.join(format!("src/test/java/com/example/{}Test.java", class_name)),
        test_java,
    )?;
    
    // Create build file based on build system
    match build_system {
        "gradle" => {
            let build_gradle = format!(
                r#"plugins {{
    id 'java'
    id 'application'
}}

group = 'com.example'
version = '1.0.0-SNAPSHOT'

java {{
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}}

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'
}}

application {{
    mainClass = 'com.example.{package_name}.{class_name}'
}}

test {{
    useJUnitPlatform()
}}
"#,
                package_name = package_name,
                class_name = class_name
            );
            fs::write(project_dir.join("build.gradle"), build_gradle)?;
            
            // Create settings.gradle
            let settings_gradle = format!("rootProject.name = '{}'\n", name);
            fs::write(project_dir.join("settings.gradle"), settings_gradle)?;
        }
        _ => {
            // Default to Maven
            let pom_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>com.example</groupId>
    <artifactId>{name}</artifactId>
    <version>1.0.0-SNAPSHOT</version>
    <packaging>jar</packaging>

    <name>{name}</name>
    <description>A new Java project created with jbuild</description>

    <properties>
        <maven.compiler.source>17</maven.compiler.source>
        <maven.compiler.target>17</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>

    <dependencies>
        <dependency>
            <groupId>org.junit.jupiter</groupId>
            <artifactId>junit-jupiter</artifactId>
            <version>5.10.0</version>
            <scope>test</scope>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.11.0</version>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-surefire-plugin</artifactId>
                <version>3.1.2</version>
            </plugin>
        </plugins>
    </build>
</project>
"#,
                name = name
            );
            fs::write(project_dir.join("pom.xml"), pom_xml)?;
        }
    }
    
    // Create .gitignore
    let gitignore = r#"# Build outputs
target/
build/
out/

# IDE files
.idea/
*.iml
.vscode/
.project
.classpath
.settings/

# OS files
.DS_Store
Thumbs.db
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    
    // Create README.md
    let readme = format!(
        r#"# {name}

A Java project created with jbuild.

## Building

```bash
jbuild build
```

## Running

```bash
jbuild run
```

## Testing

```bash
jbuild test
```
"#,
        name = name
    );
    fs::write(project_dir.join("README.md"), readme)?;
    
    println!("[INFO] Created project '{}'", name);
    println!("[INFO] ");
    println!("[INFO] To get started:");
    println!("[INFO]   cd {}", name);
    println!("[INFO]   jbuild build");
    
    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c == '.')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

pub fn run_init(build_system: &str) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    
    // Check if build file already exists
    let pom_exists = base_dir.join("pom.xml").exists();
    let gradle_exists = base_dir.join("build.gradle").exists();
    
    if pom_exists || gradle_exists {
        return Err(anyhow::anyhow!(
            "Build file already exists. Use 'jbuild add' to add dependencies."
        ));
    }
    
    println!("[INFO] Initializing jbuild in current directory");
    
    // Detect project structure
    let project_name = base_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();
    
    // Scan for Java source files
    let mut java_files: Vec<PathBuf> = Vec::new();
    let mut main_class: Option<String> = None;
    let mut detected_packages: Vec<String> = Vec::new();
    
    let source_dirs = ["src/main/java", "src/test/java", "src", "java"];
    
    for src_dir in &source_dirs {
        let src_path = base_dir.join(src_dir);
        if src_path.exists() {
            for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "java") {
                    java_files.push(path.to_path_buf());
                    
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Some(pkg) = extract_package_name(&content) {
                            if !detected_packages.contains(&pkg) {
                                detected_packages.push(pkg);
                            }
                        }
                        
                        if main_class.is_none() && content.contains("public static void main") {
                            if let Some(class) = extract_class_name(&content) {
                                if let Some(pkg) = extract_package_name(&content) {
                                    main_class = Some(format!("{}.{}", pkg, class));
                                } else {
                                    main_class = Some(class);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("[INFO] Found {} Java file(s)", java_files.len());
    
    let group_id = if let Some(first_pkg) = detected_packages.first() {
        let parts: Vec<&str> = first_pkg.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            "com.example".to_string()
        }
    } else {
        "com.example".to_string()
    };
    
    match build_system {
        "gradle" => {
            let build_gradle = format!(
                r#"plugins {{
    id 'java'{}
}}

group = '{}'
version = '1.0.0-SNAPSHOT'

java {{
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}}

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'
}}
{}
test {{
    useJUnitPlatform()
}}
"#,
                if main_class.is_some() { "\n    id 'application'" } else { "" },
                group_id,
                if let Some(ref main) = main_class {
                    format!("\napplication {{\n    mainClass = '{}'\n}}\n", main)
                } else {
                    String::new()
                }
            );
            fs::write(base_dir.join("build.gradle"), build_gradle)?;
            fs::write(base_dir.join("settings.gradle"), format!("rootProject.name = '{}'\n", project_name))?;
        }
        _ => {
            let pom_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>1.0.0-SNAPSHOT</version>
    <packaging>jar</packaging>

    <name>{}</name>
    <description>Initialized with jbuild</description>

    <properties>
        <maven.compiler.source>17</maven.compiler.source>
        <maven.compiler.target>17</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>{}
    </properties>

    <dependencies>
        <dependency>
            <groupId>org.junit.jupiter</groupId>
            <artifactId>junit-jupiter</artifactId>
            <version>5.10.0</version>
            <scope>test</scope>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.11.0</version>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-surefire-plugin</artifactId>
                <version>3.1.2</version>
            </plugin>{}
        </plugins>
    </build>
</project>
"#,
                group_id, project_name, project_name,
                if let Some(ref main) = main_class { format!("\n        <exec.mainClass>{}</exec.mainClass>", main) } else { String::new() },
                if let Some(ref _main) = main_class {
                    format!(r#"
            <plugin>
                <groupId>org.codehaus.mojo</groupId>
                <artifactId>exec-maven-plugin</artifactId>
                <version>3.1.0</version>
                <configuration>
                    <mainClass>${{exec.mainClass}}</mainClass>
                </configuration>
            </plugin>"#)
                } else { String::new() }
            );
            fs::write(base_dir.join("pom.xml"), pom_xml)?;
        }
    }
    
    for dir in &["src/main/java", "src/test/java", "src/main/resources", "src/test/resources"] {
        fs::create_dir_all(base_dir.join(dir))?;
    }
    
    Ok(())
}

fn extract_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("package ") && trimmed.ends_with(';') {
            return Some(trimmed[8..trimmed.len()-1].trim().to_string());
        }
    }
    None
}

fn extract_class_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("class ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    return Some(parts[i+1].trim_end_matches('{').to_string());
                }
            }
        }
    }
    None
}

pub fn run_add(dependency: &str, dev: bool) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let parts: Vec<&str> = dependency.split(':').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid dependency format. Use groupId:artifactId[:version]"));
    }
    
    let group_id = parts[0];
    let artifact_id = parts[1];
    let version = if parts.len() > 2 { parts[2].to_string() } else {
        ui_info(&format!("Fetching latest version for {}:{}...", group_id, artifact_id));
        fetch_latest_version(group_id, artifact_id)?
    };
    
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    
    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let mut pom_content = fs::read_to_string(&pom_path)?;
            let dep_xml = format!(
                r#"        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>{}
        </dependency>
"#,
                group_id, artifact_id, version,
                if dev { "\n            <scope>test</scope>" } else { "" }
            );
            
            if pom_content.contains("</dependencies>") {
                pom_content = pom_content.replace("</dependencies>", &format!("{}\n    </dependencies>", dep_xml));
            } else {
                pom_content = pom_content.replace("</project>", &format!("    <dependencies>\n{}\n    </dependencies>\n</project>", dep_xml));
            }
            fs::write(pom_path, pom_content)?;
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let mut build_content = fs::read_to_string(&build_path)?;
            let config = if dev { "testImplementation" } else { "implementation" };
            let dep_line = format!("    {} '{}:{}:{}'\n", config, group_id, artifact_id, version);
            
            if build_content.contains("dependencies {") {
                build_content = build_content.replace("dependencies {", &format!("dependencies {{\n{}", dep_line));
            } else {
                build_content.push_str(&format!("\ndependencies {{\n{}}}\n", dep_line));
            }
            fs::write(build_path, build_content)?;
        }
    }
    ui_success("Added dependency successfully");
    Ok(())
}

pub fn run_remove(dependency: &str) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let parts: Vec<&str> = dependency.split(':').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid dependency format. Use groupId:artifactId"));
    }
    
    let group_id = parts[0];
    let artifact_id = parts[1];
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    
    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let pom_content = fs::read_to_string(&pom_path)?;
            let pattern = format!("<artifactId>{}</artifactId>", artifact_id);
            if !pom_content.contains(&pattern) {
                ui_warn("Dependency not found");
                return Ok(());
            }
            let mut lines: Vec<String> = Vec::new();
            let mut skip = false;
            let mut dep_lines = Vec::new();
            for line in pom_content.lines() {
                if line.contains("<dependency>") {
                    skip = true;
                    dep_lines.clear();
                }
                if skip {
                    dep_lines.push(line.to_string());
                    if line.contains("</dependency>") {
                        skip = false;
                        let dep_block = dep_lines.join("\n");
                        if !dep_block.contains(&pattern) || !dep_block.contains(group_id) {
                            lines.extend(dep_lines.clone());
                        }
                    }
                } else {
                    lines.push(line.to_string());
                }
            }
            fs::write(pom_path, lines.join("\n"))?;
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let build_content = fs::read_to_string(&build_path)?;
            let pattern = format!("{}:{}", group_id, artifact_id);
            let new_content: Vec<&str> = build_content.lines()
                .filter(|line| !line.contains(&pattern))
                .collect();
            fs::write(build_path, new_content.join("\n"))?;
        }
    }
    ui_success("Removed dependency successfully");
    Ok(())
}

pub fn run_tree() -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    
    println!("[INFO] Dependency tree for {:?} project", build_system);
    
    let local_repo = DefaultLocalRepository::default();
    let resolver = DependencyResolver::new(Box::new(local_repo));
    let downloader = ArtifactDownloader::new();
    let remotes = resolver.remote_repositories().to_vec();

    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let model = parse_pom(&fs::read_to_string(pom_path)?)?;
            println!("{}:{}:{}", model.group_id, model.artifact_id, model.version);
            if let Some(deps) = model.dependencies {
                let mut visited = HashSet::new();
                walk_maven(&deps.dependencies, &downloader, &remotes, "", &mut visited)?;
            }
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let project = crate::gradle::parse_gradle_build_script(&build_path, &base_dir)?;
            println!("{}:{}", project.group.as_deref().unwrap_or("unknown"), project.name);
            let mut visited = HashSet::new();
            let maven_deps: Vec<MavenDep> = project.dependencies.iter().filter_map(|d| {
                if let (Some(g), Some(a), Some(v)) = (&d.group, &d.artifact, &d.version) {
                    Some(MavenDep {
                        group_id: g.clone(),
                        artifact_id: a.clone(),
                        version: Some(v.clone()),
                        scope: Some(d.configuration.clone()),
                        ..Default::default()
                    })
                } else { None }
            }).collect();
            walk_maven(&maven_deps, &downloader, &remotes, "", &mut visited)?;
        }
    }
    Ok(())
}

fn walk_maven(deps: &[MavenDep], _downloader: &ArtifactDownloader, _remotes: &[RemoteRepository], prefix: &str, visited: &mut HashSet<String>) -> Result<()> {
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let id = format!("{}:{}", dep.group_id, dep.artifact_id);
        let connector = if is_last { "└──" } else { "├──" };
        println!("{}{} {}:{}:{}", prefix, connector, dep.group_id, dep.artifact_id, dep.version.as_deref().unwrap_or("?"));
        
        if !visited.contains(&id) {
            visited.insert(id);
            // let next_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        }
    }
    Ok(())
}

pub fn run_search(query: &str, limit: usize) -> Result<()> {
    ui_info(&format!("Searching Maven Central for '{}'...", query));
    let results = crate::runner::maven_central::search_maven_central(query, limit)?;
    for res in results {
        println!("{}:{} ({})", res.group_id, res.artifact_id, res.version);
    }
    Ok(())
}

pub fn run_info(package: &str) -> Result<()> {
    let parts: Vec<&str> = package.split(':').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid package format. Use groupId:artifactId"));
    }
    let info = fetch_package_info(parts[0], parts[1])?;
    println!("Package: {}:{}", info.group_id, info.artifact_id);
    println!("Latest Version: {}", info.latest_version);
    println!("Updated: {}", info.updated);
    println!("\nVersions:");
    for v in info.all_versions.iter().take(10) {
        println!("  - {}", v);
    }
    Ok(())
}

pub fn run_outdated() -> Result<()> {
    ui_info("Checking for outdated dependencies...");
    Ok(())
}

pub fn run_update(dependency: Option<&str>) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    
    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let mut pom_content = fs::read_to_string(&pom_path)?;
            let model = parse_pom(&pom_content)?;
            if let Some(deps) = model.dependencies {
                let mut updated = 0;
                for dep in deps.dependencies {
                    if let Some(filter) = dependency {
                        let parts: Vec<&str> = filter.split(':').collect();
                        if parts.len() >= 2 && (dep.group_id != parts[0] || dep.artifact_id != parts[1]) { continue; }
                    }
                    if let Some(current) = dep.version {
                        ui_info(&format!("Checking {}:{}...", dep.group_id, dep.artifact_id));
                        if let Ok(latest) = fetch_latest_version(&dep.group_id, &dep.artifact_id) {
                            if latest != current {
                                ui_info(&format!("  {} -> {}", current, latest));
                                let old = format!("<groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>", dep.group_id, dep.artifact_id, current);
                                let new = format!("<groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>", dep.group_id, dep.artifact_id, latest);
                                pom_content = pom_content.replace(&old, &new);
                                updated += 1;
                            }
                        }
                    }
                }
                if updated > 0 { fs::write(pom_path, pom_content)?; ui_success(&format!("Updated {} dependencies", updated)); }
                else { ui_info("All dependencies are up to date"); }
            }
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let mut build_content = fs::read_to_string(&build_path)?;
            let project = crate::gradle::parse_gradle_build_script(&build_path, &base_dir)?;
            let mut updated = 0;
            for dep in project.dependencies {
                if let Some(filter) = dependency {
                    let parts: Vec<&str> = filter.split(':').collect();
                    if parts.len() >= 2 && (dep.group.as_deref() != Some(parts[0]) || dep.artifact.as_deref() != Some(parts[1])) { continue; }
                }
                if let (Some(g), Some(a), Some(current)) = (dep.group, dep.artifact, dep.version) {
                    ui_info(&format!("Checking {}:{}...", g, a));
                    if let Ok(latest) = fetch_latest_version(&g, &a) {
                        if latest != current {
                            ui_info(&format!("  {} -> {}", current, latest));
                            build_content = build_content.replace(&format!("'{}:{}:{}'", g, a, current), &format!("'{}:{}:{}'", g, a, latest));
                            build_content = build_content.replace(&format!("\"{}:{}:{}\"", g, a, current), &format!("\"{}:{}:{}\"", g, a, latest));
                            updated += 1;
                        }
                    }
                }
            }
            if updated > 0 { fs::write(build_path, build_content)?; ui_success(&format!("Updated {} dependencies", updated)); }
            else { ui_info("All dependencies are up to date"); }
        }
    }
    Ok(())
}

pub fn run_fmt(files: Vec<PathBuf>, check: bool) -> Result<()> {
    use std::process::Command;
    let base_dir = std::env::current_dir()?;
    let mut files_to_format = Vec::new();
    if files.is_empty() {
        if base_dir.join("src").exists() {
            for entry in WalkDir::new(base_dir.join("src")).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() && entry.path().extension().is_some_and(|e| e == "java") {
                    files_to_format.push(entry.path().to_path_buf());
                }
            }
        }
    } else {
        for path in files {
            let full = if path.is_absolute() { path } else { base_dir.join(path) };
            if full.is_dir() {
                for entry in WalkDir::new(full).into_iter().filter_map(|e| e.ok()) {
                    if entry.path().is_file() && entry.path().extension().is_some_and(|e| e == "java") {
                        files_to_format.push(entry.path().to_path_buf());
                    }
                }
            } else if full.extension().is_some_and(|e| e == "java") { files_to_format.push(full); }
        }
    }

    if files_to_format.is_empty() { ui_warn("No Java files found to format"); return Ok(()); }
    ui_info(&format!("Formatting {} Java file(s)...", files_to_format.len()));

    let formatter = which::which("google-java-format").or_else(|_| which::which("gjf")).map_err(|_| anyhow::anyhow!("google-java-format not found"))?;
    let mut cmd = Command::new(formatter);
    if check { cmd.arg("--dry-run").arg("--set-exit-if-changed"); } else { cmd.arg("--replace"); }
    for file in files_to_format { cmd.arg(file); }
    
    let status = cmd.status()?;
    if !status.success() {
        if check { ui_error("Code is not properly formatted"); std::process::exit(1); }
        else { return Err(anyhow::anyhow!("Formatting failed")); }
    }
    ui_success("Formatting completed");
    Ok(())
}

pub fn run_doc(open: bool, output: Option<PathBuf>) -> Result<()> {
    use std::process::Command;
    let base_dir = std::env::current_dir()?;
    let javadoc = which::which("javadoc").or_else(|_| {
        std::env::var("JAVA_HOME").ok().map(|home| PathBuf::from(home).join("bin").join(if cfg!(windows) { "javadoc.exe" } else { "javadoc" }))
            .filter(|p| p.exists()).ok_or(anyhow::anyhow!("javadoc not found"))
    })?;

    let output_dir = output.unwrap_or_else(|| base_dir.join("target/site/apidocs"));
    fs::create_dir_all(&output_dir)?;

    let mut source_files = Vec::new();
    for dir in &["src/main/java", "src/test/java"] {
        let path = base_dir.join(dir);
        if path.exists() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() && entry.path().extension().is_some_and(|e| e == "java") {
                    source_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    if source_files.is_empty() { ui_warn("No Java files found"); return Ok(()); }
    ui_info(&format!("Generating Javadoc for {} files...", source_files.len()));

    let mut cmd = Command::new(javadoc);
    for file in source_files { cmd.arg(file); }
    cmd.arg("-d").arg(&output_dir).arg("-quiet");
    
    if !cmd.status()?.success() { return Err(anyhow::anyhow!("Javadoc generation failed")); }
    ui_success(&format!("Javadoc generated at {}", output_dir.display()));

    if open {
        let index = output_dir.join("index.html");
        if index.exists() {
            #[cfg(target_os = "macos")] { Command::new("open").arg(&index).status()?; }
            #[cfg(target_os = "linux")] { Command::new("xdg-open").arg(&index).status()?; }
            #[cfg(target_os = "windows")] { Command::new("cmd").arg("/C").arg("start").arg(&index).status()?; }
        }
    }
    Ok(())
}

pub fn run_audit() -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    ui_info(&format!("Auditing dependencies for {:?} project...", build_system));

    let mut dependencies = Vec::new();
    match build_system {
        BuildSystem::Maven => {
            let pom = fs::read_to_string(base_dir.join("pom.xml"))?;
            let model = parse_pom(&pom)?;
            if let Some(deps) = model.dependencies {
                for dep in deps.dependencies {
                    if let Some(v) = dep.version { dependencies.push((dep.group_id, dep.artifact_id, v)); }
                }
            }
        }
        BuildSystem::Gradle => {
            let project = crate::gradle::parse_gradle_build_script(&base_dir.join("build.gradle"), &base_dir)?;
            for dep in project.dependencies {
                if let (Some(g), Some(a), Some(v)) = (dep.group, dep.artifact, dep.version) { dependencies.push((g, a, v)); }
            }
        }
    }

    if dependencies.is_empty() { ui_info("No dependencies found to audit"); return Ok(()); }
    let mut vulnerabilities = 0;
    for (g, a, v) in &dependencies {
        if v.contains("alpha") || v.contains("beta") || v.contains("SNAPSHOT") {
            ui_warn(&format!("  ⚠️  {}:{}:{} - Pre-release version may have issues", g, a, v));
            vulnerabilities += 1;
        }
    }

    if vulnerabilities == 0 { ui_success("No issues found"); }
    else { ui_warn(&format!("Found {} potential issue(s)", vulnerabilities)); }
    Ok(())
}

pub fn run_watch(run_tests: bool, watch_paths: Vec<PathBuf>) -> Result<()> {
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    use std::sync::mpsc;
    use std::time::Duration;
    
    let base_dir = std::env::current_dir()?;
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;
    
    ui_info(&format!("Watching for changes in {:?} project...", build_system));
    ui_info("Press Ctrl+C to stop");
    println!();
    
    let paths_to_watch: Vec<PathBuf> = if watch_paths.is_empty() {
        vec![base_dir.join("src")]
    } else {
        watch_paths.iter().map(|p| if p.is_absolute() { p.clone() } else { base_dir.join(p) }).collect()
    };
    
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |result| {
        if let Ok(event) = result { let _ = tx.send(event); }
    })?;
    
    for path in &paths_to_watch {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            ui_info(&format!("Watching: {}", path.display()));
        } else {
            ui_warn(&format!("Path does not exist, skipping: {}", path.display()));
        }
    }
    
    let executor: Box<dyn BuildExecutor> = match build_system {
        BuildSystem::Maven => Box::new(MavenBuildExecutor::new()),
        BuildSystem::Gradle => Box::new(GradleExecutor::new()),
    };
    
    let build_goals = if run_tests {
        match build_system {
            BuildSystem::Maven => vec!["compile".to_string(), "test-compile".to_string(), "test".to_string()],
            BuildSystem::Gradle => vec!["compileJava".to_string(), "compileTestJava".to_string(), "test".to_string()],
        }
    } else {
        match build_system {
            BuildSystem::Maven => vec!["compile".to_string(), "test-compile".to_string()],
            BuildSystem::Gradle => vec!["compileJava".to_string(), "compileTestJava".to_string()],
        }
    };
    
    let mut last_change = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(500);
    let mut pending_build = false;
    
    ui_info("Performing initial build...");
    let request = ExecutionRequest {
        base_directory: base_dir.clone(),
        goals: build_goals.clone(),
        system_properties: HashMap::new(),
        show_errors: true,
        offline: false,
    };
    
    if let Ok(result) = executor.execute(request.clone()) {
        if result.success { ui_success("Initial build succeeded"); }
        else { ui_warn("Initial build had errors"); }
    }
    
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Event { kind: EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_), paths, .. }) => {
                let relevant_changes: Vec<_> = paths.iter()
                    .filter(|p| {
                        let path_str = p.to_string_lossy();
                        path_str.ends_with(".java") || path_str.ends_with(".xml") || 
                        path_str.ends_with(".gradle") || path_str.ends_with(".gradle.kts")
                    })
                    .collect();
                
                if !relevant_changes.is_empty() {
                    for path in &relevant_changes { ui_info(&format!("Change detected: {}", path.display())); }
                    last_change = std::time::Instant::now();
                    pending_build = true;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if pending_build && last_change.elapsed() >= debounce_duration {
                    pending_build = false;
                    ui_info("Rebuilding...");
                    if let Ok(result) = executor.execute(request.clone()) {
                        if result.success { ui_success("Build succeeded"); }
                        else {
                            ui_warn("Build had errors");
                            for err in &result.errors { ui_warn(err); }
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(anyhow::anyhow!("File watcher disconnected")),
            _ => {}
        }
    }
}

pub fn run_completions(shell: clap_complete::Shell) -> Result<()> {
    use clap::CommandFactory;
    use crate::cli::Cli;
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}

pub fn run_workspace_new(name: &str) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let workspace_file = base_dir.join("jbuild-workspace.toml");
    if workspace_file.exists() { return Err(anyhow::anyhow!("Workspace already exists")); }
    fs::write(workspace_file, format!("[package]\nname = \"{}\"\nversion = \"1.0.0\"\n\nmembers = []\n", name))?;
    ui_success(&format!("Created workspace '{}'", name));
    Ok(())
}

pub fn run_workspace_add(path: &str) -> Result<()> {
    use crate::config::JbuildWorkspace;
    let base_dir = std::env::current_dir()?;
    let workspace_file = base_dir.join("jbuild-workspace.toml");
    if !workspace_file.exists() { return Err(anyhow::anyhow!("No workspace found")); }
    let mut config = JbuildWorkspace::from_file(&workspace_file)?;
    config.add_member(path.to_string());
    config.save_to_file(&workspace_file)?;
    ui_success(&format!("Added project '{}' to workspace", path));
    Ok(())
}

pub fn run_workspace_remove(path: &str) -> Result<()> {
    use crate::config::JbuildWorkspace;
    let base_dir = std::env::current_dir()?;
    let workspace_file = base_dir.join("jbuild-workspace.toml");
    if !workspace_file.exists() { return Err(anyhow::anyhow!("No workspace found")); }
    let mut config = JbuildWorkspace::from_file(&workspace_file)?;
    config.remove_member(path);
    config.save_to_file(&workspace_file)?;
    ui_success(&format!("Removed project '{}' from workspace", path));
    Ok(())
}

pub fn run_workspace_list() -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let workspace = Workspace::from_directory(&base_dir)?;
    println!("Workspace members:");
    for member in workspace.members {
        println!("  {} ({})", member.name, member.relative_path);
    }
    Ok(())
}

pub fn run_workspace_build(goals: Vec<String>) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    let workspace = Workspace::from_directory(&base_dir)?;
    ui_info(&format!("Building workspace with {} members...", workspace.members.len()));
    let order = workspace.get_build_order();
    for member in order {
        ui_info(&format!("Building {}...", member.name));
        let path = base_dir.join(&member.relative_path);
        std::env::set_current_dir(&path)?;
        if let Some(bs) = member.build_system {
            let request = ExecutionRequest {
                base_directory: path,
                goals: if goals.is_empty() { bs.default_goals() } else { goals.clone() },
                system_properties: HashMap::new(),
                show_errors: true,
                offline: false,
            };
            let executor: Box<dyn BuildExecutor> = match bs {
                BuildSystem::Maven => Box::new(MavenBuildExecutor::new()),
                BuildSystem::Gradle => Box::new(GradleExecutor::new()),
            };
            let result = executor.execute(request)?;
            if !result.success { return Err(anyhow::anyhow!("Build failed for {}", member.name)); }
        }
        std::env::set_current_dir(&base_dir)?;
    }
    ui_success("Workspace build completed");
    Ok(())
}

pub fn run_run(args: Vec<String>, main_class: Option<String>, example: Option<String>) -> Result<()> {
    let base_dir = std::env::current_dir()?;
    if let Some(example_name) = example {
        return run_example(&base_dir, &example_name, &args);
    }
    run_app(&args, main_class.as_deref())
}

pub fn run_example(base_dir: &Path, example_name: &str, args: &[String]) -> Result<()> {
    ui_info(&format!("Looking for example: {}", example_name));
    let example_dirs = [base_dir.join("src/main/java"), base_dir.join("examples")];
    let mut example_class: Option<String> = None;
    for dir in &example_dirs {
        if !dir.exists() { continue; }
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "java") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem.eq_ignore_ascii_case(example_name) {
                        if let Ok(content) = fs::read_to_string(path) {
                            if let (Some(pkg), Some(class)) = (extract_package_name(&content), extract_class_name(&content)) {
                                example_class = Some(format!("{}.{}", pkg, class));
                                break;
                            }
                        }
                    }
                }
            }
        }
        if example_class.is_some() { break; }
    }
    let main_class = example_class.ok_or_else(|| anyhow::anyhow!("Example '{}' not found", example_name))?;
    run_app(args, Some(&main_class))
}

pub fn run_app(args: &[String], main_class_override: Option<&str>) -> Result<()> {
    use crate::runner::{detect_main_class, extract_main_class_from_config, build_classpath, run_java_app};
    let base_dir = std::env::current_dir()?;
    let build_system = BuildSystem::detect(&base_dir).ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    let main_class = if let Some(mc) = main_class_override { mc.to_string() } else {
        extract_main_class_from_config(&base_dir)?.or(detect_main_class(&base_dir)?).ok_or_else(|| anyhow::anyhow!("Main class not found"))?
    };
    let classpath = build_classpath(&base_dir, &build_system)?;
    run_java_app(&base_dir, &main_class, &classpath, args)
}
