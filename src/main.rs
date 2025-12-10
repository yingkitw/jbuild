use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use jbuild::build::{BuildSystem, BuildExecutor, ExecutionRequest, BuildWrapper, GoalMapper};
use jbuild::maven::core::MavenBuildExecutor;
use jbuild::gradle::core::GradleExecutor;
use jbuild::checkstyle::{Checker, ConfigurationLoader, DefaultLogger};
use jbuild::ui::{info, success, error, warn, build_success, build_failure};

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
    /// Run the application
    Run {
        /// Arguments to pass to the application
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        /// Main class to run (auto-detected if not specified)
        #[arg(short = 'm', long = "main-class")]
        main_class: Option<String>,
    },
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
    /// Initialize jbuild in an existing project
    Init {
        /// Build system to use (maven, gradle)
        #[arg(short = 'b', long = "build-system", default_value = "maven")]
        build_system: String,
    },
    /// Remove a dependency from the project
    Remove {
        /// Dependency in format groupId:artifactId
        dependency: String,
    },
    /// Search Maven Central for packages
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short = 'n', long = "limit", default_value = "10")]
        limit: usize,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish)
        shell: clap_complete::Shell,
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
        Some(Commands::Init { build_system }) => return run_init(build_system),
        Some(Commands::Remove { dependency }) => return run_remove(dependency),
        Some(Commands::Search { query, limit }) => return run_search(query, *limit),
        Some(Commands::Completions { shell }) => {
            return run_completions(*shell);
        }
        Some(Commands::Run { args, main_class }) => {
            return run_app(args, main_class.as_deref());
        }
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
            Some(Commands::Run { .. }) => vec!["run".to_string()],
            Some(Commands::Lint { .. }) | Some(Commands::New { .. }) | 
            Some(Commands::Tree) | Some(Commands::Add { .. }) |
            Some(Commands::Init { .. }) | Some(Commands::Remove { .. }) |
            Some(Commands::Search { .. }) | Some(Commands::Completions { .. }) => unreachable!(), // Handled earlier
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
                build_success();
                Ok(())
            } else {
                build_failure();
                for err in &result.errors {
                    error(err);
                }
                std::process::exit(1);
            }
        }
        Err(e) => {
            error(&format!("Build failed: {}", e));
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
        warn("No Java files found to check");
        return Ok(());
    }
    
    info(&format!("Checking {} Java file(s)", files_to_check.len()));

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
        error(&format!("Checkstyle found {} error(s)", error_count));
        std::process::exit(1);
    } else {
        success("Checkstyle completed with no errors");
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
        // Fetch latest version from Maven Central
        use jbuild::runner::fetch_latest_version;
        use jbuild::ui::{info, warn};
        
        info(&format!("Fetching latest version for {}:{}...", group_id, artifact_id));
        match fetch_latest_version(group_id, artifact_id) {
            Ok(v) => {
                info(&format!("Found latest version: {}", v));
                v
            }
            Err(e) => {
                warn(&format!("Failed to fetch latest version: {}. Using 'LATEST' placeholder.", e));
                warn("Please specify version explicitly: jbuild add group:artifact:version");
                return Err(anyhow::anyhow!(
                    "Could not determine version for {}:{}. Please specify version explicitly.",
                    group_id,
                    artifact_id
                ));
            }
        }
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

/// Initialize jbuild in an existing project
fn run_init(build_system: &str) -> anyhow::Result<()> {
    use std::fs;
    use walkdir::WalkDir;
    
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
    
    // Check standard Maven/Gradle source directories
    let source_dirs = [
        "src/main/java",
        "src/test/java",
        "src",
        "java",
    ];
    
    for src_dir in &source_dirs {
        let src_path = base_dir.join(src_dir);
        if src_path.exists() {
            for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "java") {
                    java_files.push(path.to_path_buf());
                    
                    // Try to detect package and main class
                    if let Ok(content) = fs::read_to_string(path) {
                        // Extract package name
                        if let Some(pkg) = extract_package_name(&content) {
                            if !detected_packages.contains(&pkg) {
                                detected_packages.push(pkg);
                            }
                        }
                        
                        // Check for main method
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
    if let Some(ref main) = main_class {
        println!("[INFO] Detected main class: {}", main);
    }
    
    // Determine group ID from packages
    let group_id = if let Some(first_pkg) = detected_packages.first() {
        // Use first two parts of package as group ID
        let parts: Vec<&str> = first_pkg.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            "com.example".to_string()
        }
    } else {
        "com.example".to_string()
    };
    
    // Create build file
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
            
            // Create settings.gradle
            let settings_gradle = format!("rootProject.name = '{}'\n", project_name);
            fs::write(base_dir.join("settings.gradle"), settings_gradle)?;
            
            println!("[INFO] Created build.gradle and settings.gradle");
        }
        _ => {
            // Default to Maven
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
                group_id,
                project_name,
                project_name,
                if main_class.is_some() {
                    format!("\n        <exec.mainClass>{}</exec.mainClass>", main_class.as_ref().unwrap())
                } else {
                    String::new()
                },
                if main_class.is_some() {
                    r#"
            <plugin>
                <groupId>org.codehaus.mojo</groupId>
                <artifactId>exec-maven-plugin</artifactId>
                <version>3.1.0</version>
                <configuration>
                    <mainClass>${exec.mainClass}</mainClass>
                </configuration>
            </plugin>"#
                } else {
                    ""
                }
            );
            fs::write(base_dir.join("pom.xml"), pom_xml)?;
            
            println!("[INFO] Created pom.xml");
        }
    }
    
    // Create standard directories if they don't exist
    let dirs_to_create = [
        "src/main/java",
        "src/test/java",
        "src/main/resources",
        "src/test/resources",
    ];
    
    for dir in &dirs_to_create {
        let dir_path = base_dir.join(dir);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
            println!("[INFO] Created {}", dir);
        }
    }
    
    // Create .gitignore if it doesn't exist
    let gitignore_path = base_dir.join(".gitignore");
    if !gitignore_path.exists() {
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
        fs::write(&gitignore_path, gitignore)?;
        println!("[INFO] Created .gitignore");
    }
    
    println!("[INFO] ");
    println!("[INFO] Project initialized successfully!");
    println!("[INFO] ");
    println!("[INFO] Next steps:");
    println!("[INFO]   jbuild build    # Build the project");
    if main_class.is_some() {
        println!("[INFO]   jbuild run      # Run the application");
    }
    println!("[INFO]   jbuild test     # Run tests");
    
    Ok(())
}

/// Extract package name from Java source
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

/// Extract class name from Java source
fn extract_class_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("public class ") || trimmed.contains("public final class ") {
            // Extract class name
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    let class_name = parts[i + 1]
                        .trim_end_matches('{')
                        .trim_end_matches('<')
                        .to_string();
                    return Some(class_name);
                }
            }
        }
    }
    None
}

/// Remove a dependency from the project
fn run_remove(dependency: &str) -> anyhow::Result<()> {
    let base_dir = std::env::current_dir()?;
    
    // Parse dependency string (groupId:artifactId)
    let parts: Vec<&str> = dependency.split(':').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!(
            "Invalid dependency format. Use groupId:artifactId"
        ));
    }
    
    let group_id = parts[0];
    let artifact_id = parts[1];
    
    // Detect build system
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;
    
    println!("[INFO] Removing {}:{} from {:?} project", group_id, artifact_id, build_system);
    
    match build_system {
        BuildSystem::Maven => {
            let pom_path = base_dir.join("pom.xml");
            let pom_content = std::fs::read_to_string(&pom_path)?;
            
            // Find and remove the dependency block
            let new_content = remove_maven_dependency(&pom_content, group_id, artifact_id)?;
            
            if new_content == pom_content {
                println!("[WARN] Dependency {}:{} not found in pom.xml", group_id, artifact_id);
                return Ok(());
            }
            
            std::fs::write(&pom_path, new_content)?;
        }
        BuildSystem::Gradle => {
            let build_path = base_dir.join("build.gradle");
            let build_content = std::fs::read_to_string(&build_path)?;
            
            // Find and remove the dependency line
            let new_content = remove_gradle_dependency(&build_content, group_id, artifact_id)?;
            
            if new_content == build_content {
                println!("[WARN] Dependency {}:{} not found in build.gradle", group_id, artifact_id);
                return Ok(());
            }
            
            std::fs::write(&build_path, new_content)?;
        }
    }
    
    println!("[INFO] Removed dependency successfully");
    
    Ok(())
}

/// Remove a dependency from Maven pom.xml
fn remove_maven_dependency(content: &str, group_id: &str, artifact_id: &str) -> anyhow::Result<String> {
    let mut result = String::new();
    let mut in_target_dependency = false;
    let mut skip_until_close = false;
    let mut depth = 0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if skip_until_close {
            if trimmed.contains("</dependency>") {
                depth -= 1;
                if depth == 0 {
                    skip_until_close = false;
                    in_target_dependency = false;
                }
                continue;
            }
            if trimmed.contains("<dependency>") {
                depth += 1;
            }
            continue;
        }
        
        if trimmed.starts_with("<dependency>") || trimmed == "<dependency>" {
            // Look ahead to check if this is the target dependency
            in_target_dependency = false;
        }
        
        // Check if this line contains our target groupId and artifactId
        if trimmed.contains(&format!("<groupId>{}</groupId>", group_id)) {
            in_target_dependency = true;
        }
        
        if in_target_dependency && trimmed.contains(&format!("<artifactId>{}</artifactId>", artifact_id)) {
            // This is the dependency to remove - go back and remove from <dependency>
            // Find the last <dependency> in result and remove from there
            if let Some(pos) = result.rfind("<dependency>") {
                // Find the start of the line containing <dependency>
                let line_start = result[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                result.truncate(line_start);
                skip_until_close = true;
                depth = 1;
                continue;
            }
        }
        
        result.push_str(line);
        result.push('\n');
    }
    
    // Clean up extra blank lines
    let cleaned = result
        .lines()
        .collect::<Vec<_>>()
        .join("\n");
    
    Ok(cleaned)
}

/// Remove a dependency from Gradle build.gradle
fn remove_gradle_dependency(content: &str, group_id: &str, artifact_id: &str) -> anyhow::Result<String> {
    let pattern = format!("{}:{}", group_id, artifact_id);
    
    let new_lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.contains(&pattern))
        .collect();
    
    Ok(new_lines.join("\n"))
}

/// Search Maven Central for packages
fn run_search(query: &str, limit: usize) -> anyhow::Result<()> {
    use jbuild::runner::search_maven_central;
    use jbuild::ui::{info, warn};
    
    info(&format!("Searching Maven Central for '{}'...", query));
    println!();
    
    let results = match search_maven_central(query, limit) {
        Ok(r) => r,
        Err(e) => {
            warn(&format!("Failed to search Maven Central: {}", e));
            return Err(e);
        }
    };
    
    if results.is_empty() {
        warn(&format!("No packages found for '{}'", query));
        return Ok(());
    }
    
    println!("Found {} package(s):", results.len());
    println!();
    println!("{:<50} {:<15} {}", "PACKAGE", "VERSION", "UPDATED");
    println!("{}", "-".repeat(80));
    
    for result in &results {
        let package = format!("{}:{}", result.group_id, result.artifact_id);
        println!("{:<50} {:<15} {}", package, result.version, result.updated);
    }
    
    println!();
    println!("To add a dependency: jbuild add <package>:<version>");
    println!("Or use: jbuild add {}:{}  (auto-detects latest version)", 
        results[0].group_id, results[0].artifact_id);
    
    Ok(())
}

/// Generate shell completions
fn run_completions(shell: clap_complete::Shell) -> anyhow::Result<()> {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}

/// Run the Java application
fn run_app(args: &[String], main_class_override: Option<&str>) -> anyhow::Result<()> {
    use jbuild::runner::{detect_main_class, extract_main_class_from_config, build_classpath, run_java_app};
    use jbuild::build::{BuildSystem, BuildExecutor, ExecutionRequest};
    use jbuild::maven::core::MavenBuildExecutor;
    use jbuild::gradle::core::GradleExecutor;
    use jbuild::ui::{info, warn, success};

    let base_dir = std::env::current_dir()?;

    // Detect build system
    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected. Looking for pom.xml or build.gradle"))?;

    info(&format!("Detected build system: {:?}", build_system));

    // Auto-build the project before running
    info("Building project...");
    let executor: Box<dyn BuildExecutor> = match build_system {
        BuildSystem::Maven => Box::new(MavenBuildExecutor::new()),
        BuildSystem::Gradle => Box::new(GradleExecutor::new()),
    };

    let build_request = ExecutionRequest {
        base_directory: base_dir.clone(),
        goals: match build_system {
            BuildSystem::Maven => vec!["compile".to_string()],
            BuildSystem::Gradle => vec!["compileJava".to_string()],
        },
        system_properties: std::collections::HashMap::new(),
        show_errors: true,
        offline: false,
    };

    match executor.execute(build_request) {
        Ok(result) => {
            if result.success {
                success("Build completed successfully");
            } else {
                warn("Build completed with warnings");
                for error in &result.errors {
                    warn(error);
                }
            }
        }
        Err(e) => {
            warn(&format!("Build failed: {}. Attempting to run anyway...", e));
        }
    }

    // Find main class
    let main_class = if let Some(mc) = main_class_override {
        mc.to_string()
    } else {
        // Try to get from configuration first
        if let Some(mc) = extract_main_class_from_config(&base_dir)? {
            mc
        } else if let Some(mc) = detect_main_class(&base_dir)? {
            mc
        } else {
            return Err(anyhow::anyhow!(
                "Could not find main class. Please specify with --main-class or ensure your project has a main method."
            ));
        }
    };

    info(&format!("Running main class: {}", main_class));

    // Build classpath
    let classpath = build_classpath(&base_dir, &build_system)?;
    
    if classpath.is_empty() {
        warn("Classpath is empty. The project may need to be built first.");
        warn("Run 'jbuild build' or 'jbuild compile' first.");
        return Err(anyhow::anyhow!("Cannot run application: classpath is empty"));
    }

    // Run the application
    run_java_app(&base_dir, &main_class, &classpath, args)?;

    Ok(())
}
