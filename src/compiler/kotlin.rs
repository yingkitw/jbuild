//! Kotlin compiler integration
//!
//! Provides support for compiling Kotlin source files

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

/// Kotlin compiler configuration
#[derive(Debug, Clone)]
pub struct KotlinCompilerConfig {
    /// Kotlin compiler home (kotlinc location)
    pub kotlin_home: Option<PathBuf>,
    /// JVM target version
    pub jvm_target: String,
    /// API version
    pub api_version: Option<String>,
    /// Language version
    pub language_version: Option<String>,
    /// Enable progressive mode
    pub progressive: bool,
    /// Compiler plugins
    pub plugins: Vec<KotlinPlugin>,
}

#[derive(Debug, Clone)]
pub struct KotlinPlugin {
    pub id: String,
    pub path: PathBuf,
    pub options: Vec<(String, String)>,
}

impl Default for KotlinCompilerConfig {
    fn default() -> Self {
        Self {
            kotlin_home: None,
            jvm_target: "17".to_string(),
            api_version: None,
            language_version: None,
            progressive: false,
            plugins: Vec::new(),
        }
    }
}

/// Kotlin compiler
pub struct KotlinCompiler {
    config: KotlinCompilerConfig,
}

impl KotlinCompiler {
    pub fn new(config: KotlinCompilerConfig) -> Self {
        Self { config }
    }

    /// Detect Kotlin compiler from environment
    pub fn detect_kotlinc() -> Result<PathBuf> {
        // Try KOTLIN_HOME environment variable
        if let Ok(kotlin_home) = std::env::var("KOTLIN_HOME") {
            let kotlinc = PathBuf::from(kotlin_home).join("bin/kotlinc");
            if kotlinc.exists() {
                return Ok(kotlinc);
            }
        }

        // Try to find kotlinc in PATH
        which::which("kotlinc")
            .context("Kotlin compiler not found. Set KOTLIN_HOME or add kotlinc to PATH")
    }

    /// Compile Kotlin sources
    pub fn compile(
        &self,
        source_files: &[PathBuf],
        output_dir: &Path,
        classpath: &[PathBuf],
    ) -> Result<KotlinCompilationResult> {
        let kotlinc = if let Some(ref home) = self.config.kotlin_home {
            home.join("bin/kotlinc")
        } else {
            Self::detect_kotlinc()?
        };

        std::fs::create_dir_all(output_dir)?;

        let mut cmd = Command::new(&kotlinc);

        // JVM target
        cmd.arg("-jvm-target").arg(&self.config.jvm_target);

        // API version
        if let Some(ref api_version) = self.config.api_version {
            cmd.arg("-api-version").arg(api_version);
        }

        // Language version
        if let Some(ref lang_version) = self.config.language_version {
            cmd.arg("-language-version").arg(lang_version);
        }

        // Progressive mode
        if self.config.progressive {
            cmd.arg("-progressive");
        }

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

        // Plugins
        for plugin in &self.config.plugins {
            cmd.arg(format!("-Xplugin={}", plugin.path.display()));
            for (key, value) in &plugin.options {
                cmd.arg(format!("-P"));
                cmd.arg(format!("plugin:{}:{}={}", plugin.id, key, value));
            }
        }

        // Source files
        for source in source_files {
            cmd.arg(source);
        }

        // Execute
        let output = cmd.output()
            .context("Failed to execute Kotlin compiler")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(KotlinCompilationResult {
            success: output.status.success(),
            stdout,
            stderr,
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Compile mixed Java and Kotlin sources
    pub fn compile_mixed(
        &self,
        kotlin_sources: &[PathBuf],
        java_sources: &[PathBuf],
        output_dir: &Path,
        classpath: &[PathBuf],
    ) -> Result<KotlinCompilationResult> {
        // Kotlin compiler can handle both Kotlin and Java files
        let mut all_sources = kotlin_sources.to_vec();
        all_sources.extend_from_slice(java_sources);
        
        self.compile(&all_sources, output_dir, classpath)
    }
}

/// Result of Kotlin compilation
#[derive(Debug, Clone)]
pub struct KotlinCompilationResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub output_dir: PathBuf,
}

/// Common Kotlin compiler plugins
pub struct KotlinPlugins;

impl KotlinPlugins {
    /// All-open plugin (makes classes open for frameworks like Spring)
    pub fn all_open(annotations: Vec<String>) -> KotlinPlugin {
        KotlinPlugin {
            id: "org.jetbrains.kotlin.allopen".to_string(),
            path: PathBuf::from("kotlin-allopen-compiler-plugin.jar"),
            options: annotations
                .into_iter()
                .map(|a| ("annotation".to_string(), a))
                .collect(),
        }
    }

    /// No-arg plugin (generates no-arg constructors)
    pub fn no_arg(annotations: Vec<String>) -> KotlinPlugin {
        KotlinPlugin {
            id: "org.jetbrains.kotlin.noarg".to_string(),
            path: PathBuf::from("kotlin-noarg-compiler-plugin.jar"),
            options: annotations
                .into_iter()
                .map(|a| ("annotation".to_string(), a))
                .collect(),
        }
    }

    /// Spring plugin (combines all-open and no-arg for Spring)
    pub fn spring() -> Vec<KotlinPlugin> {
        vec![
            Self::all_open(vec![
                "org.springframework.stereotype.Component".to_string(),
                "org.springframework.stereotype.Service".to_string(),
                "org.springframework.stereotype.Repository".to_string(),
                "org.springframework.boot.autoconfigure.SpringBootApplication".to_string(),
            ]),
            Self::no_arg(vec![
                "javax.persistence.Entity".to_string(),
                "jakarta.persistence.Entity".to_string(),
            ]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = KotlinCompilerConfig::default();
        assert_eq!(config.jvm_target, "17");
        assert!(!config.progressive);
    }

    #[test]
    fn test_spring_plugins() {
        let plugins = KotlinPlugins::spring();
        assert_eq!(plugins.len(), 2);
    }
}
