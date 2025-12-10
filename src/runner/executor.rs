//! Execute Java applications with proper classpath

use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context};
use which::which;

/// Build classpath for running a Java application
pub fn build_classpath(base_dir: &Path, build_system: &crate::build::BuildSystem) -> Result<String> {
    let mut classpath_parts = Vec::new();

    match build_system {
        crate::build::BuildSystem::Maven => {
            // Maven: target/classes and target/test-classes
            let classes_dir = base_dir.join("target/classes");
            if classes_dir.exists() {
                classpath_parts.push(classes_dir.to_string_lossy().to_string());
            }

            // Add dependency JARs from local repository
            // For now, we'll use a simple approach - in production, resolve from POM
            let lib_dir = base_dir.join("target/lib");
            if lib_dir.exists() {
                add_jars_from_dir(&lib_dir, &mut classpath_parts)?;
            }
        }
        crate::build::BuildSystem::Gradle => {
            // Gradle: build/classes/java/main and build/resources/main
            let classes_dir = base_dir.join("build/classes/java/main");
            if classes_dir.exists() {
                classpath_parts.push(classes_dir.to_string_lossy().to_string());
            }

            let resources_dir = base_dir.join("build/resources/main");
            if resources_dir.exists() {
                classpath_parts.push(resources_dir.to_string_lossy().to_string());
            }

            // Add dependency JARs
            let libs_dir = base_dir.join("build/libs");
            add_jars_from_dir(&libs_dir, &mut classpath_parts)?;
        }
    }

    Ok(classpath_parts.join(if cfg!(windows) { ";" } else { ":" }))
}

/// Add JAR files from a directory to classpath
fn add_jars_from_dir(dir: &Path, classpath_parts: &mut Vec<String>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "jar").unwrap_or(false) {
            classpath_parts.push(path.to_string_lossy().to_string());
        }
    }

    Ok(())
}

/// Run a Java application
pub fn run_java_app(
    base_dir: &Path,
    main_class: &str,
    classpath: &str,
    args: &[String],
) -> Result<()> {
    // Find java executable
    let java = which("java")
        .context("Java not found in PATH. Please ensure JAVA_HOME is set.")?;

    // Build command
    let mut cmd = Command::new(java);
    cmd.arg("-cp").arg(classpath);
    cmd.arg(main_class);
    
    // Add application arguments
    for arg in args {
        cmd.arg(arg);
    }

    cmd.current_dir(base_dir);

    tracing::info!("Running: {} {}", main_class, args.join(" "));
    tracing::debug!("Classpath: {}", classpath);

    let status = cmd.status()
        .context("Failed to run application")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Application exited with code: {:?}",
            status.code()
        ));
    }

    Ok(())
}

