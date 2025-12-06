use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use jbuild::build::{BuildSystem, BuildExecutor, ExecutionRequest};
use jbuild::maven::core::MavenBuildExecutor;
use jbuild::gradle::core::GradleExecutor;

/// jbuild - A high-performance build tool for Java projects (Maven & Gradle)
#[derive(Parser)]
#[command(name = "jbuild")]
#[command(version = "0.1.0")]
#[command(about = "jbuild - High-performance Java build tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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

    /// Suppress Maven output
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Produce execution debug output
    #[arg(short = 'X', long = "debug")]
    debug: bool,

    /// Produce execution output
    #[arg(short = 'e', long = "errors")]
    errors: bool,

    /// File path to the POM file
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,
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
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

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

    // Determine goals from command
    let goals = match &cli.command {
        Some(Commands::Validate) => vec!["validate".to_string()],
        Some(Commands::Compile) => vec!["compile".to_string()],
        Some(Commands::Test) => vec!["test".to_string()],
        Some(Commands::Package) => vec!["package".to_string()],
        Some(Commands::Install) => vec!["install".to_string()],
        Some(Commands::Deploy) => vec!["deploy".to_string()],
        Some(Commands::Clean) => vec!["clean".to_string()],
        None => {
            // Default to compile if no command specified
            vec!["compile".to_string()]
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
        goals,
        system_properties,
        show_errors: cli.show_errors,
        offline: cli.offline,
    };

    info!("Starting build execution");
    info!("Build system: {:?}", build_system);
    info!("Base directory: {:?}", request.base_directory);
    info!("Goals: {:?}", request.goals);

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

