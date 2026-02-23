//! Scala compiler integration
//!
//! Provides support for compiling Scala source files

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

/// Scala compiler configuration
#[derive(Debug, Clone)]
pub struct ScalaCompilerConfig {
    /// Scala compiler home (scalac location)
    pub scala_home: Option<PathBuf>,
    /// Scala version (2.12, 2.13, 3.x)
    pub scala_version: String,
    /// Target JVM version
    pub target: String,
    /// Compiler options
    pub options: Vec<String>,
    /// Enable optimizations
    pub optimize: bool,
}

impl Default for ScalaCompilerConfig {
    fn default() -> Self {
        Self {
            scala_home: None,
            scala_version: "2.13".to_string(),
            target: "17".to_string(),
            options: Vec::new(),
            optimize: false,
        }
    }
}

/// Scala compiler
pub struct ScalaCompiler {
    config: ScalaCompilerConfig,
}

impl ScalaCompiler {
    pub fn new(config: ScalaCompilerConfig) -> Self {
        Self { config }
    }

    /// Detect Scala compiler from environment
    pub fn detect_scalac() -> Result<PathBuf> {
        // Try SCALA_HOME environment variable
        if let Ok(scala_home) = std::env::var("SCALA_HOME") {
            let scalac = PathBuf::from(scala_home).join("bin/scalac");
            if scalac.exists() {
                return Ok(scalac);
            }
        }

        // Try to find scalac in PATH
        which::which("scalac")
            .context("Scala compiler not found. Set SCALA_HOME or add scalac to PATH")
    }

    /// Compile Scala sources
    pub fn compile(
        &self,
        source_files: &[PathBuf],
        output_dir: &Path,
        classpath: &[PathBuf],
    ) -> Result<ScalaCompilationResult> {
        let scalac = if let Some(ref home) = self.config.scala_home {
            home.join("bin/scalac")
        } else {
            Self::detect_scalac()?
        };

        std::fs::create_dir_all(output_dir)?;

        let mut cmd = Command::new(&scalac);

        // Target JVM version
        cmd.arg("-target").arg(format!("jvm-{}", self.config.target));

        // Classpath
        if !classpath.is_empty() {
            let cp = classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.arg("-classpath").arg(cp);
        }

        // Output directory
        cmd.arg("-d").arg(output_dir);

        // Optimizations
        if self.config.optimize {
            cmd.arg("-opt:l:inline");
            cmd.arg("-opt-inline-from:**");
        }

        // Additional compiler options
        for option in &self.config.options {
            cmd.arg(option);
        }

        // Common useful flags
        cmd.arg("-deprecation");
        cmd.arg("-feature");
        cmd.arg("-unchecked");

        // Source files
        for source in source_files {
            cmd.arg(source);
        }

        // Execute
        let output = cmd.output()
            .context("Failed to execute Scala compiler")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ScalaCompilationResult {
            success: output.status.success(),
            stdout,
            stderr,
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Compile mixed Java and Scala sources
    pub fn compile_mixed(
        &self,
        scala_sources: &[PathBuf],
        java_sources: &[PathBuf],
        output_dir: &Path,
        classpath: &[PathBuf],
    ) -> Result<ScalaCompilationResult> {
        // First compile Scala sources
        let scala_result = self.compile(scala_sources, output_dir, classpath)?;
        
        if !scala_result.success {
            return Ok(scala_result);
        }

        // Then compile Java sources with Scala classes in classpath
        if !java_sources.is_empty() {
            let mut java_classpath = classpath.to_vec();
            java_classpath.push(output_dir.to_path_buf());

            // Use javac for Java files
            let javac = which::which("javac")
                .context("Java compiler not found")?;

            let mut cmd = Command::new(javac);
            cmd.arg("-d").arg(output_dir);

            let cp = java_classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.arg("-classpath").arg(cp);

            for source in java_sources {
                cmd.arg(source);
            }

            let java_output = cmd.output()
                .context("Failed to compile Java sources")?;

            let java_stdout = String::from_utf8_lossy(&java_output.stdout).to_string();
            let java_stderr = String::from_utf8_lossy(&java_output.stderr).to_string();

            return Ok(ScalaCompilationResult {
                success: java_output.status.success(),
                stdout: format!("{}\n{}", scala_result.stdout, java_stdout),
                stderr: format!("{}\n{}", scala_result.stderr, java_stderr),
                output_dir: output_dir.to_path_buf(),
            });
        }

        Ok(scala_result)
    }
}

/// Result of Scala compilation
#[derive(Debug, Clone)]
pub struct ScalaCompilationResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub output_dir: PathBuf,
}

/// Scala compiler options helper
pub struct ScalaOptions;

impl ScalaOptions {
    /// Enable all warnings
    pub fn all_warnings() -> Vec<String> {
        vec![
            "-Xlint".to_string(),
            "-Ywarn-dead-code".to_string(),
            "-Ywarn-numeric-widen".to_string(),
            "-Ywarn-value-discard".to_string(),
        ]
    }

    /// Strict compilation (treat warnings as errors)
    pub fn strict() -> Vec<String> {
        let mut opts = Self::all_warnings();
        opts.push("-Xfatal-warnings".to_string());
        opts
    }

    /// Enable experimental features
    pub fn experimental() -> Vec<String> {
        vec![
            "-Xexperimental".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ScalaCompilerConfig::default();
        assert_eq!(config.scala_version, "2.13");
        assert_eq!(config.target, "17");
        assert!(!config.optimize);
    }

    #[test]
    fn test_scala_options() {
        let warnings = ScalaOptions::all_warnings();
        assert!(!warnings.is_empty());
        
        let strict = ScalaOptions::strict();
        assert!(strict.contains(&"-Xfatal-warnings".to_string()));
    }
}
