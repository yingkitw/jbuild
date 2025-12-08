use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use jbuild::build::{BuildSystem, BuildExecutor, ExecutionRequest, BuildWrapper, GoalMapper};
use jbuild::maven::core::MavenBuildExecutor;
use jbuild::gradle::core::GradleExecutor;
use jbuild::checkstyle::{Checker, ConfigurationLoader, DefaultLogger};

/// jbuild - A high-performance build tool for Java projects (Maven & Gradle)
#[derive(Parser)]
#[command(name = "jbuild")]
#[command(version = "0.1.0")]
#[command(about = "jbuild - High-performance Java build tool supporting Maven and Gradle", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Goals/tasks to execute (e.g., clean compile test)
    #[arg(trailing_var_arg = true)]
    goals: Vec<String>,

    /// Define a system property
    #[arg(short = 'D', long = "define", value_name = "PROPERTY")]
    define: Vec<String>,

    /// Activate a profile by id
    #[arg(short = 'P', long = "activate-profiles", value_name = "PROFILES")]
    profiles: Vec<String>,

    /// Run in offline mode
    #[arg(long = "offline")]
    offline: bool,

    /// Run in non-interactive mode
    #[arg(long = "batch-mode", short = 'B')]
    batch_mode: bool,

    /// Show errors
    #[arg(long = "show-errors")]
    show_errors: bool,

    /// Suppress output
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Produce execution debug output
    #[arg(short = 'X', long = "debug")]
    debug: bool,

    /// Produce execution output
    #[arg(short = 'e', long = "errors")]
    errors: bool,

    /// File path to the build file (pom.xml or build.gradle)
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,

    /// Use wrapper (mvnw/gradlew) if available
    #[arg(long = "use-wrapper")]
    use_wrapper: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate the project
    Validate,
    /// Compile the project
    Compile,
    /// Run tests
    Test,
    /// Package the project
    Package,
    /// Install the project
    Install,
    /// Deploy the project
    Deploy,
    /// Clean the project
    Clean,
    /// Build the project (compile + test + package)
    Build,
    /// Run the application (Gradle only)
    Run,
    /// Lint Java code using Checkstyle
    Lint {
        /// Configuration file for Checkstyle (XML format)
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
        /// Files or directories to check
        #[arg(trailing_var_arg = true)]
        files: Vec<PathBuf>,
    },
    /// Create a new Java project
    New {
        /// Project name
        name: String,
        /// Project template (app, lib, multi)
        #[arg(short = 't', long = "template", default_value = "app")]
        template: String,
        /// Build system to use (maven, gradle)
        #[arg(short = 'b', long = "build-system", default_value = "maven")]
        build_system: String,
    },
    /// Display dependency tree
    Tree,
    /// Add a dependency to the project
    Add {
        /// Dependency in format groupId:artifactId or groupId:artifactId:version
        dependency: String,
        /// Add as dev/test dependency
        #[arg(long = "dev")]
        dev: bool,
    },
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    // Handle commands that don't require build system detection
    match &cli.command {
        Some(Commands::Lint { .. }) => return run_lint(&cli),
        Some(Commands::New { name, template, build_system }) => {
            return run_new(name, template, build_system);
        }
        Some(Commands::Tree) => return run_tree(),
        Some(Commands::Add { dependency, dev }) => return run_add(dependency, *dev),
        _ => {}
    }

    // Determine base directory
    let base_dir = if let Some(file) = &cli.file {
        file.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    } else {
        std::env::current_dir()?
    };

    // Detect build system
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;
    
    info!("Detected build system: {:?}", build_system);

    // Check for wrapper if requested
    if cli.use_wrapper {
        if let Some(wrapper) = BuildWrapper::detect(&base_dir) {
            info!("Using wrapper: {:?}", wrapper.script_path);
            if let Some(version) = wrapper.get_version() {
                info!("Wrapper version: {}", version);
            }
        }
    }

    // Determine goals from command or arguments
    let goals = if !cli.goals.is_empty() {
        cli.goals.clone()
    } else {
        match &cli.command {
            Some(Commands::Validate) => vec!["validate".to_string()],
            Some(Commands::Compile) => vec!["compile".to_string()],
            Some(Commands::Test) => vec!["test".to_string()],
            Some(Commands::Package) => vec!["package".to_string()],
            Some(Commands::Install) => vec!["install".to_string()],
            Some(Commands::Deploy) => vec!["deploy".to_string()],
            Some(Commands::Clean) => vec!["clean".to_string()],
            Some(Commands::Build) => vec!["build".to_string()],
            Some(Commands::Run) => vec!["run".to_string()],
            Some(Commands::Lint { .. }) | Some(Commands::New { .. }) | 
            Some(Commands::Tree) | Some(Commands::Add { .. }) => unreachable!(), // Handled earlier
            None => vec!["compile".to_string()],
        }
    };

    // Map goals to appropriate build system tasks
    let goal_mapper = GoalMapper::new();
    let mapped_goals = match build_system {
        BuildSystem::Maven => goals.clone(), // Keep Maven goals as-is
        BuildSystem::Gradle => {
            // Convert Maven-style goals to Gradle tasks if needed
            goals.iter().map(|g| {
                if GoalMapper::is_lifecycle_phase(g) {
                    goal_mapper.maven_to_gradle(g).first().cloned().unwrap_or_else(|| g.clone())
                } else {
                    g.clone()
                }
            }).collect()
        }
    };

    // Parse system properties
    let mut system_properties = std::collections::HashMap::new();
    for prop in &cli.define {
        if let Some((key, value)) = prop.split_once('=') {
            system_properties.insert(key.to_string(), value.to_string());
        }
    }

    // Create generic execution request
    let request = ExecutionRequest {
        base_directory: base_dir.clone(),
        goals: mapped_goals.clone(),
        system_properties,
        show_errors: cli.show_errors,
        offline: cli.offline,
    };

    info!("Starting build execution");
    info!("Build system: {:?}", build_system);
    info!("Base directory: {:?}", request.base_directory);
    info!("Goals: {:?}", request.goals);
    if goals != mapped_goals {
        info!("Mapped from: {:?}", goals);
    }

    // Execute build based on detected system
    let executor: Box<dyn BuildExecutor> = match build_system {
        BuildSystem::Maven => Box::new(MavenBuildExecutor::new()),
        BuildSystem::Gradle => Box::new(GradleExecutor::new()),
    };

    match executor.execute(request) {
        Ok(result) => {
            if result.success {
                println!("[INFO] BUILD SUCCESS");
                Ok(())
            } else {
                eprintln!("[ERROR] BUILD FAILURE");
                for error in &result.errors {
                    eprintln!("[ERROR] {}", error);
                }
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Build failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run checkstyle lint on Java files
fn run_lint(cli: &Cli) -> anyhow::Result<()> {
    let (config_file, files) = match &cli.command {
        Some(Commands::Lint { config, files }) => (config.clone(), files.clone()),
        _ => unreachable!(),
    };

    // Determine base directory
    let base_dir = std::env::current_dir()?;

    // Collect files to check
    let files_to_check: Vec<PathBuf> = if files.is_empty() {
        // Default: check src/main/java and src/test/java
        let mut collected = Vec::new();
        for dir in &["src/main/java", "src/test/java"] {
            let path = base_dir.join(dir);
            if path.exists() {
                collected.extend(collect_java_files(&path)?);
            }
        }
        collected
    } else {
        let mut collected = Vec::new();
        for path in &files {
            let full_path = if path.is_absolute() {
                path.clone()
            } else {
                base_dir.join(path)
            };
            if full_path.is_dir() {
                collected.extend(collect_java_files(&full_path)?);
            } else if full_path.extension().is_some_and(|e| e == "java") {
                collected.push(full_path);
            }
        }
        collected
    };

    if files_to_check.is_empty() {
        println!("[INFO] No Java files found to check");
        return Ok(());
    }

    println!("[INFO] Checking {} Java file(s)", files_to_check.len());

    // Load configuration or use defaults
    let config = if let Some(config_path) = config_file {
        ConfigurationLoader::load_configuration(&config_path)?
    } else {
        // Create default configuration with common checks
        ConfigurationLoader::create_default_configuration()
    };

    // Create checker and configure it
    let mut checker = Checker::new();
    checker.configure(&config)?;

    // Add default logger
    let logger = DefaultLogger::new();
    checker.add_listener(Box::new(logger));

    // Run checks
    let error_count = checker.process(&files_to_check)?;

    if error_count > 0 {
        eprintln!("[ERROR] Checkstyle found {} error(s)", error_count);
        std::process::exit(1);
    } else {
        println!("[INFO] Checkstyle completed with no errors");
        Ok(())
    }
}

/// Collect all Java files from a directory recursively
fn collect_java_files(dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    use walkdir::WalkDir;
    
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "java") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

/// Create a new Java project
fn run_new(name: &str, template: &str, build_system: &str) -> anyhow::Result<()> {
    use std::fs;
    
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

/// Convert a string to PascalCase
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

/// Display dependency tree
fn run_tree() -> anyhow::Result<()> {
    let base_dir = std::env::current_dir()?;
    
    // Detect build system
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;
    
    println!("[INFO] Dependency tree for {:?} project", build_system);
    println!();
    
    match build_system {
        BuildSystem::Maven => {
            // Parse pom.xml
            let pom_path = base_dir.join("pom.xml");
            let pom_content = std::fs::read_to_string(&pom_path)?;
            let model = jbuild::model::parser::parse_pom(&pom_content)?;
            
            println!("{}:{}:{}", 
                &model.group_id,
                &model.artifact_id,
                &model.version
            );
            
            if let Some(deps) = &model.dependencies {
                let dep_count = deps.dependencies.len();
                for (i, dep) in deps.dependencies.iter().enumerate() {
                    let scope = dep.scope.as_deref().unwrap_or("compile");
                    let prefix = if i == dep_count - 1 { "└──" } else { "├──" };
                    println!("{} {}:{}:{} ({})",
                        prefix,
                        &dep.group_id,
                        &dep.artifact_id,
                        dep.version.as_deref().unwrap_or("?"),
                        scope
                    );
                }
            }
        }
        BuildSystem::Gradle => {
            // Parse build.gradle
            let build_path = base_dir.join("build.gradle");
            let project = jbuild::gradle::parse_gradle_build_script(&build_path, &base_dir)?;
            
            println!("{}:{}", 
                project.group.as_deref().unwrap_or("unknown"),
                &project.name
            );
            
            let dep_count = project.dependencies.len();
            for (i, dep) in project.dependencies.iter().enumerate() {
                let prefix = if i == dep_count - 1 { "└──" } else { "├──" };
                println!("{} {}:{}:{} ({})",
                    prefix,
                    dep.group.as_deref().unwrap_or("?"),
                    dep.artifact.as_deref().unwrap_or("?"),
                    dep.version.as_deref().unwrap_or("?"),
                    &dep.configuration
                );
            }
        }
    }
    
    Ok(())
}

/// Add a dependency to the project
fn run_add(dependency: &str, dev: bool) -> anyhow::Result<()> {
    let base_dir = std::env::current_dir()?;
    
    // Parse dependency string (groupId:artifactId or groupId:artifactId:version)
    let parts: Vec<&str> = dependency.split(':').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!(
            "Invalid dependency format. Use groupId:artifactId or groupId:artifactId:version"
        ));
    }
    
    let group_id = parts[0];
    let artifact_id = parts[1];
    let version = if parts.len() > 2 { 
        parts[2].to_string() 
    } else {
        // TODO: Fetch latest version from Maven Central
        "LATEST".to_string()
    };
    
    // Detect build system
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;
    
    let scope = if dev { "test" } else { "compile" };
    
    println!("[INFO] Adding {}:{}:{} ({}) to {:?} project", 
        group_id, artifact_id, version, scope, build_system);
    
    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let pom_content = std::fs::read_to_string(&pom_path)?;
            
            // Simple XML insertion - find </dependencies> and insert before it
            let dep_xml = format!(
                r#"        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>{}
        </dependency>
    </dependencies>"#,
                group_id,
                artifact_id,
                version,
                if dev { "\n            <scope>test</scope>" } else { "" }
            );
            
            let new_content = if pom_content.contains("</dependencies>") {
                pom_content.replace("</dependencies>", &dep_xml)
            } else if pom_content.contains("</project>") {
                // No dependencies section, create one
                let deps_section = format!(
                    r#"    <dependencies>
        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>{}
        </dependency>
    </dependencies>

</project>"#,
                    group_id,
                    artifact_id,
                    version,
                    if dev { "\n            <scope>test</scope>" } else { "" }
                );
                pom_content.replace("</project>", &deps_section)
            } else {
                return Err(anyhow::anyhow!("Could not parse pom.xml"));
            };
            
            std::fs::write(&pom_path, new_content)?;
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let build_content = std::fs::read_to_string(&build_path)?;
            
            let config = if dev { "testImplementation" } else { "implementation" };
            let dep_line = format!("    {} '{}:{}:{}'\n", config, group_id, artifact_id, version);
            
            // Find dependencies block and add to it
            let new_content = if build_content.contains("dependencies {") {
                // Insert after "dependencies {"
                build_content.replace(
                    "dependencies {",
                    &format!("dependencies {{\n{}", dep_line)
                )
            } else {
                // Add dependencies block at the end
                format!("{}\n\ndependencies {{\n{}}}\n", build_content, dep_line)
            };
            
            std::fs::write(&build_path, new_content)?;
        }
    }
    
    println!("[INFO] Added dependency successfully");
    println!("[INFO] Run 'jbuild build' to download and compile");
    
    Ok(())
}
