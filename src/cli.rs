use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// jbuild - A high-performance build tool for Java projects (Maven & Gradle)
#[derive(Parser)]
#[command(name = "jbuild")]
#[command(version = "0.1.3")]
#[command(about = "jbuild - High-performance Java build tool supporting Maven and Gradle", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Goals/tasks to execute (e.g., clean compile test)
    #[arg(trailing_var_arg = true)]
    pub goals: Vec<String>,

    /// Define a system property
    #[arg(short = 'D', long = "define", value_name = "PROPERTY")]
    pub define: Vec<String>,

    /// Activate a profile by id
    #[arg(short = 'P', long = "activate-profiles", value_name = "PROFILES")]
    pub profiles: Vec<String>,

    /// Run in offline mode
    #[arg(long = "offline")]
    pub offline: bool,

    /// Run in non-interactive mode
    #[arg(long = "batch-mode", short = 'B')]
    pub batch_mode: bool,

    /// Show errors
    #[arg(long = "show-errors")]
    pub show_errors: bool,

    /// Suppress output
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Produce execution debug output
    #[arg(short = 'X', long = "debug")]
    pub debug: bool,

    /// File path to the build file (pom.xml or build.gradle)
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Use wrapper (mvnw/gradlew) if available
    #[arg(long = "use-wrapper")]
    pub use_wrapper: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Validate,
    Compile,
    Test,
    Package,
    Install,
    Deploy,
    Clean,
    Build,
    Check,
    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(short = 'm', long = "main-class")]
        main_class: Option<String>,
        #[arg(long = "example")]
        example: Option<String>,
    },
    Lint {
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
        #[arg(trailing_var_arg = true)]
        files: Vec<PathBuf>,
    },
    New {
        name: String,
        #[arg(short = 't', long = "template", default_value = "app")]
        template: String,
        #[arg(short = 'b', long = "build-system", default_value = "maven")]
        build_system: String,
    },
    Tree,
    Add {
        dependency: String,
        #[arg(long = "dev")]
        dev: bool,
    },
    Init {
        #[arg(short = 'b', long = "build-system", default_value = "maven")]
        build_system: String,
    },
    Remove {
        dependency: String,
    },
    Search {
        query: String,
        #[arg(short = 'n', long = "limit", default_value = "10")]
        limit: usize,
    },
    Completions {
        shell: clap_complete::Shell,
    },
    Update {
        #[arg(value_name = "DEPENDENCY")]
        dependency: Option<String>,
    },
    Info {
        package: String,
    },
    Outdated,
    Fmt {
        #[arg(trailing_var_arg = true)]
        files: Vec<PathBuf>,
        #[arg(long = "check")]
        check: bool,
    },
    Doc {
        #[arg(long = "open")]
        open: bool,
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },
    Audit,
    Watch {
        #[arg(long = "test")]
        test: bool,
        #[arg(short = 'w', long = "watch")]
        watch_paths: Vec<PathBuf>,
    },
    WorkspaceNew { name: String },
    WorkspaceAdd { path: String },
    WorkspaceRemove { path: String },
    WorkspaceList,
    WorkspaceBuild {
        #[arg(trailing_var_arg = true)]
        goals: Vec<String>,
    },
}
