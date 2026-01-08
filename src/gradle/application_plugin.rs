//! Application Plugin
//!
//! Implements Gradle's application plugin for building and running Java applications.

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};

use crate::gradle::model::GradleProject;
use crate::gradle::source_set::SourceSetContainer;

/// Application plugin configuration
#[derive(Debug, Clone)]
pub struct ApplicationExtension {
    /// Main class name
    pub main_class: Option<String>,
    /// Main module name (for modular applications)
    pub main_module: Option<String>,
    /// Application name (used for scripts)
    pub application_name: Option<String>,
    /// Default JVM arguments
    pub application_default_jvm_args: Vec<String>,
    /// Executable directory
    pub executable_dir: String,
}

impl Default for ApplicationExtension {
    fn default() -> Self {
        Self {
            main_class: None,
            main_module: None,
            application_name: None,
            application_default_jvm_args: Vec::new(),
            executable_dir: "bin".to_string(),
        }
    }
}

impl ApplicationExtension {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_main_class(mut self, main_class: impl Into<String>) -> Self {
        self.main_class = Some(main_class.into());
        self
    }

    pub fn with_application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    pub fn add_jvm_arg(&mut self, arg: impl Into<String>) {
        self.application_default_jvm_args.push(arg.into());
    }
}

/// Application plugin implementation
pub struct ApplicationPlugin {
    /// Plugin configuration
    pub extension: ApplicationExtension,
    /// Project base directory
    pub base_dir: PathBuf,
}

impl ApplicationPlugin {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            extension: ApplicationExtension::default(),
            base_dir,
        }
    }

    pub fn with_extension(mut self, extension: ApplicationExtension) -> Self {
        self.extension = extension;
        self
    }

    /// Configure from a GradleProject
    pub fn configure_from_project(&mut self, project: &GradleProject) {
        // Try to detect main class from build script
        if let Some(main_class) = &project.main_class {
            self.extension.main_class = Some(main_class.clone());
        }

        // Set application name from project name
        if self.extension.application_name.is_none() {
            self.extension.application_name = Some(project.name.clone());
        }
    }

    /// Execute the 'run' task
    pub fn run(&self, args: &[String]) -> Result<()> {
        let main_class = self.extension.main_class.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No mainClass configured for application plugin"))?;

        // Build classpath
        let classes_dir = self.base_dir.join("build/classes/java/main");
        let resources_dir = self.base_dir.join("build/resources/main");
        let libs_dir = self.base_dir.join("build/libs");

        let mut classpath_parts = vec![];
        if classes_dir.exists() {
            classpath_parts.push(classes_dir.to_string_lossy().to_string());
        }
        if resources_dir.exists() {
            classpath_parts.push(resources_dir.to_string_lossy().to_string());
        }

        // Add dependency JARs
        if libs_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&libs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "jar").unwrap_or(false) {
                        classpath_parts.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        let classpath = classpath_parts.join(if cfg!(windows) { ";" } else { ":" });

        // Find java executable
        let java = which::which("java")
            .context("Java not found in PATH")?;

        // Build command
        let mut cmd = Command::new(java);
        
        // Add JVM args
        for arg in &self.extension.application_default_jvm_args {
            cmd.arg(arg);
        }

        // Add classpath and main class
        cmd.arg("-cp").arg(&classpath);
        cmd.arg(main_class);

        // Add application args
        for arg in args {
            cmd.arg(arg);
        }

        cmd.current_dir(&self.base_dir);

        tracing::info!("Running: {} {}", main_class, args.join(" "));

        let status = cmd.status()
            .context("Failed to run application")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Application exited with code: {:?}", status.code()));
        }

        Ok(())
    }

    /// Generate start scripts (Unix and Windows)
    pub fn create_start_scripts(&self) -> Result<()> {
        let main_class = self.extension.main_class.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No mainClass configured"))?;

        let app_name = self.extension.application_name.as_ref()
            .unwrap_or(&"app".to_string())
            .clone();

        let scripts_dir = self.base_dir.join("build").join(&self.extension.executable_dir);
        std::fs::create_dir_all(&scripts_dir)?;

        // Generate Unix script
        let unix_script = format!(r#"#!/bin/sh

APP_HOME="$(cd "$(dirname "$0")/.." && pwd)"
CLASSPATH="$APP_HOME/lib/*"

exec java {} -cp "$CLASSPATH" {} "$@"
"#, 
            self.extension.application_default_jvm_args.join(" "),
            main_class
        );

        let unix_path = scripts_dir.join(&app_name);
        std::fs::write(&unix_path, unix_script)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&unix_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&unix_path, perms)?;
        }

        // Generate Windows batch script
        let windows_script = format!(r#"@echo off

set APP_HOME=%~dp0..
set CLASSPATH=%APP_HOME%\lib\*

java {} -cp "%CLASSPATH%" {} %*
"#,
            self.extension.application_default_jvm_args.join(" "),
            main_class
        );

        let windows_path = scripts_dir.join(format!("{app_name}.bat"));
        std::fs::write(&windows_path, windows_script)?;

        tracing::info!("Created start scripts in {:?}", scripts_dir);

        Ok(())
    }

    /// Create a distribution (installDist task)
    pub fn install_dist(&self) -> Result<()> {
        let app_name = self.extension.application_name.as_ref()
            .unwrap_or(&"app".to_string())
            .clone();

        let install_dir = self.base_dir.join("build/install").join(&app_name);
        let lib_dir = install_dir.join("lib");
        let bin_dir = install_dir.join("bin");

        std::fs::create_dir_all(&lib_dir)?;
        std::fs::create_dir_all(&bin_dir)?;

        // Copy application JAR
        let jar_name = format!("{app_name}.jar");
        let source_jar = self.base_dir.join("build/libs").join(&jar_name);
        if source_jar.exists() {
            std::fs::copy(&source_jar, lib_dir.join(&jar_name))?;
        }

        // Create start scripts in the install directory
        let main_class = self.extension.main_class.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No mainClass configured"))?;

        // Unix script
        let unix_script = format!(r#"#!/bin/sh

APP_HOME="$(cd "$(dirname "$0")/.." && pwd)"
CLASSPATH="$APP_HOME/lib/*"

exec java {} -cp "$CLASSPATH" {} "$@"
"#,
            self.extension.application_default_jvm_args.join(" "),
            main_class
        );
        std::fs::write(bin_dir.join(&app_name), unix_script)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(bin_dir.join(&app_name))?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(bin_dir.join(&app_name), perms)?;
        }

        // Windows script
        let windows_script = format!(r#"@echo off
set APP_HOME=%~dp0..
set CLASSPATH=%APP_HOME%\lib\*
java {} -cp "%CLASSPATH%" {} %*
"#,
            self.extension.application_default_jvm_args.join(" "),
            main_class
        );
        std::fs::write(bin_dir.join(format!("{app_name}.bat")), windows_script)?;

        tracing::info!("Installed distribution to {:?}", install_dir);

        Ok(())
    }
}

/// Detect main class from source files
pub fn detect_main_class(source_sets: &SourceSetContainer) -> Option<String> {
    let main_source_set = source_sets.main()?;

    for src_dir in &main_source_set.java_src_dirs {
        if let Some(main_class) = find_main_class_in_dir(src_dir) {
            return Some(main_class);
        }
    }

    None
}

/// Find a class with main method in a directory
fn find_main_class_in_dir(dir: &PathBuf) -> Option<String> {
    if !dir.exists() {
        return None;
    }

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "java").unwrap_or(false))
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            // Simple check for main method
            if content.contains("public static void main") {
                // Extract class name from file path
                let relative = path.strip_prefix(dir).ok()?;
                let class_name = relative.to_string_lossy()
                    .replace(['/', '\\'], ".")
                    .trim_end_matches(".java")
                    .to_string();
                return Some(class_name);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_extension() {
        let ext = ApplicationExtension::new()
            .with_main_class("com.example.Main")
            .with_application_name("my-app");

        assert_eq!(ext.main_class, Some("com.example.Main".to_string()));
        assert_eq!(ext.application_name, Some("my-app".to_string()));
    }

    #[test]
    fn test_add_jvm_args() {
        let mut ext = ApplicationExtension::new();
        ext.add_jvm_arg("-Xmx512m");
        ext.add_jvm_arg("-Denv=prod");

        assert_eq!(ext.application_default_jvm_args.len(), 2);
    }

    #[test]
    fn test_application_plugin_creation() {
        let plugin = ApplicationPlugin::new(PathBuf::from("/project"))
            .with_extension(ApplicationExtension::new().with_main_class("Main"));

        assert_eq!(plugin.extension.main_class, Some("Main".to_string()));
    }
}
