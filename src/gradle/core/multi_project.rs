//! Gradle Multi-Project Support
//!
//! Handles loading and executing multi-project Gradle builds.

use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::gradle::model::GradleProject;
use crate::gradle::parse_gradle_build_script;
use crate::gradle::settings::{GradleSettings, find_settings_file, parse_settings_file};
use crate::build::ExecutionResult;

/// Load settings for a multi-project build
pub fn load_settings(base_dir: &PathBuf) -> Result<Option<GradleSettings>> {
    if let Some(settings_file) = find_settings_file(base_dir) {
        let settings = parse_settings_file(&settings_file, base_dir)
            .with_context(|| format!("Failed to parse settings file: {:?}", settings_file))?;
        Ok(Some(settings))
    } else {
        Ok(None)
    }
}

/// Load a single Gradle project from a directory
pub fn load_project(base_dir: &PathBuf) -> Result<GradleProject> {
    let build_file_gradle = base_dir.join("build.gradle");
    let build_file_kts = base_dir.join("build.gradle.kts");
    
    let build_file = if build_file_gradle.exists() {
        build_file_gradle
    } else if build_file_kts.exists() {
        build_file_kts
    } else {
        return Err(anyhow::anyhow!("No build.gradle or build.gradle.kts found in {:?}", base_dir));
    };

    parse_gradle_build_script(&build_file, base_dir)
        .with_context(|| format!("Failed to parse Gradle build script: {:?}", build_file))
}

/// Load all projects in a multi-project build
pub fn load_multi_project(base_dir: &PathBuf) -> Result<Vec<GradleProject>> {
    let settings = load_settings(base_dir)?;

    match settings {
        Some(settings) if settings.is_multi_project() => {
            let mut projects = Vec::new();

            // Load root project if it has a build file
            if base_dir.join("build.gradle").exists() || base_dir.join("build.gradle.kts").exists() {
                if let Ok(root_project) = load_project(base_dir) {
                    projects.push(root_project);
                }
            }

            // Load subprojects
            for subproject in &settings.subprojects {
                let subproject_dir = subproject.directory(base_dir);
                if subproject_dir.exists() {
                    match load_project(&subproject_dir) {
                        Ok(mut project) => {
                            project.name = subproject.name().to_string();
                            projects.push(project);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load subproject {}: {}",
                                subproject.path,
                                e
                            );
                        }
                    }
                }
            }

            Ok(projects)
        }
        _ => {
            // Single project build
            let project = load_project(base_dir)?;
            Ok(vec![project])
        }
    }
}

/// Execute tasks on all projects in a multi-project build
pub fn execute_multi_project(
    base_dir: &PathBuf,
    tasks: &[String],
    execute_task: impl Fn(&GradleProject, &str) -> Result<()>,
) -> Result<ExecutionResult> {
    let projects = load_multi_project(base_dir)?;

    let mut all_errors = Vec::new();
    let mut all_success = true;

    for project in &projects {
        tracing::info!("Executing tasks for project: {}", project.name);

        for task in tasks {
            match execute_task(project, task) {
                Ok(()) => {
                    tracing::info!(
                        "Successfully executed task '{}' for project '{}'",
                        task,
                        project.name
                    );
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to execute task '{}' for project '{}': {}",
                        task, project.name, e
                    );
                    all_errors.push(error_msg);
                    all_success = false;
                }
            }
        }
    }

    Ok(ExecutionResult {
        success: all_success,
        errors: all_errors,
    })
}

/// Check if a directory is a multi-project build
pub fn is_multi_project(base_dir: &PathBuf) -> bool {
    if let Some(settings_file) = find_settings_file(base_dir) {
        if let Ok(settings) = parse_settings_file(&settings_file, base_dir) {
            return settings.is_multi_project();
        }
    }
    false
}
