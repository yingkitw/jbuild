use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use crate::compiler::{ClasspathBuilder, SourceDiscovery};

/// Java compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Source Java version (e.g., "1.8", "11", "17")
    pub source_version: Option<String>,
    /// Target Java version (e.g., "1.8", "11", "17")
    pub target_version: Option<String>,
    /// Output directory for compiled classes
    pub output_directory: PathBuf,
    /// Source directories
    pub source_roots: Vec<PathBuf>,
    /// Classpath entries
    pub classpath: ClasspathBuilder,
    /// Additional compiler arguments
    pub additional_args: Vec<String>,
    /// Verbose output
    pub verbose: bool,
}

impl CompilerConfig {
    pub fn new(output_directory: PathBuf) -> Self {
        Self {
            source_version: None,
            target_version: None,
            output_directory,
            source_roots: Vec::new(),
            classpath: ClasspathBuilder::new(),
            additional_args: Vec::new(),
            verbose: false,
        }
    }

    pub fn with_source_version(mut self, version: String) -> Self {
        self.source_version = Some(version);
        self
    }

    pub fn with_target_version(mut self, version: String) -> Self {
        self.target_version = Some(version);
        self
    }

    pub fn with_source_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.source_roots = roots;
        self
    }

    pub fn with_classpath(mut self, classpath: ClasspathBuilder) -> Self {
        self.classpath = classpath;
        self
    }
}

/// Java compiler invocation result
#[derive(Debug)]
pub struct CompilationResult {
    pub success: bool,
    pub output: String,
    pub error_output: String,
    pub compiled_files: usize,
}

/// Java compiler
pub struct JavaCompiler;

impl JavaCompiler {
    /// Find the Java compiler (javac)
    pub fn find_javac() -> Result<PathBuf> {
        // Try JAVA_HOME first
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let javac_path = PathBuf::from(java_home)
                .join("bin")
                .join(if cfg!(windows) { "javac.exe" } else { "javac" });
            
            if javac_path.exists() {
                return Ok(javac_path);
            }
        }

        // Try to find javac in PATH
        let javac_name = if cfg!(windows) { "javac.exe" } else { "javac" };
        
        if let Ok(path) = which::which(javac_name) {
            return Ok(path);
        }

        Err(anyhow::anyhow!(
            "Java compiler (javac) not found. Please set JAVA_HOME or ensure javac is in PATH"
        ))
    }

    /// Compile Java sources
    pub fn compile(config: &CompilerConfig) -> Result<CompilationResult> {
        let javac_path = Self::find_javac()?;

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&config.output_directory)
            .with_context(|| format!("Failed to create output directory: {:?}", config.output_directory))?;

        // Discover source files
        let source_files = SourceDiscovery::discover_from_roots(&config.source_roots)
            .context("Failed to discover source files")?;

        if source_files.is_empty() {
            tracing::info!("No Java source files found to compile");
            return Ok(CompilationResult {
                success: true,
                output: String::new(),
                error_output: String::new(),
                compiled_files: 0,
            });
        }

        tracing::info!(
            "Compiling {} Java source file(s) to {:?}",
            source_files.len(),
            config.output_directory
        );

        // Build javac command
        let mut cmd = Command::new(&javac_path);

        // Set output directory
        cmd.arg("-d").arg(&config.output_directory);

        // Set source version
        if let Some(ref source_version) = config.source_version {
            cmd.arg("-source").arg(source_version);
        }

        // Set target version
        if let Some(ref target_version) = config.target_version {
            cmd.arg("-target").arg(target_version);
        }

        // Set classpath
        let classpath = config.classpath.build();
        if !classpath.is_empty() {
            cmd.arg("-classpath").arg(&classpath);
        }

        // Add additional arguments
        for arg in &config.additional_args {
            cmd.arg(arg);
        }

        // Add source files
        for source_file in &source_files {
            cmd.arg(source_file);
        }

        // Execute compilation
        let output = cmd.output()
            .with_context(|| format!("Failed to execute javac: {javac_path:?}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();

        if !success {
            tracing::error!("Compilation failed:\n{}", stderr);
            return Err(anyhow::anyhow!("Java compilation failed: {stderr}"));
        }

        if config.verbose && !stdout.is_empty() {
            tracing::info!("Compiler output:\n{}", stdout);
        }

        Ok(CompilationResult {
            success: true,
            output: stdout,
            error_output: stderr,
            compiled_files: source_files.len(),
        })
    }

    /// Compile test sources
    pub fn compile_tests(config: &CompilerConfig) -> Result<CompilationResult> {
        // Test compilation is similar to regular compilation
        // but typically uses test source roots and includes main classes in classpath
        Self::compile(config)
    }
}

