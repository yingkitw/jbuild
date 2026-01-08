use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};

/// External Maven process executor for plugin execution
pub struct ExternalMavenExecutor;

impl ExternalMavenExecutor {
    /// Execute a plugin goal using external Maven process
    pub fn execute_plugin_goal(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        goal: &str,
        project_dir: &PathBuf,
    ) -> Result<()> {
        // Find Maven executable
        let mvn_exe = Self::find_maven()
            .ok_or_else(|| anyhow::anyhow!("Maven not found. Please ensure mvn is in PATH"))?;

        tracing::info!(
            "Executing plugin {}:{}:{}:{} via external Maven process",
            group_id, artifact_id, version, goal
        );

        // Build Maven command: mvn groupId:artifactId:version:goal
        let goal_string = format!("{group_id}:{artifact_id}:{version}:{goal}");
        
        let output = Command::new(&mvn_exe)
            .arg(&goal_string)
            .current_dir(project_dir)
            .output()
            .with_context(|| format!("Failed to execute Maven command: {goal_string}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Maven plugin execution failed: {stderr}"
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            tracing::debug!("Maven output:\n{}", stdout);
        }

        Ok(())
    }

    /// Find Maven executable
    fn find_maven() -> Option<PathBuf> {
        // Try M2_HOME first
        if let Ok(m2_home) = std::env::var("M2_HOME") {
            let mvn_path = PathBuf::from(m2_home)
                .join("bin")
                .join(if cfg!(windows) { "mvn.cmd" } else { "mvn" });
            
            if mvn_path.exists() {
                return Some(mvn_path);
            }
        }

        // Try MAVEN_HOME
        if let Ok(maven_home) = std::env::var("MAVEN_HOME") {
            let mvn_path = PathBuf::from(maven_home)
                .join("bin")
                .join(if cfg!(windows) { "mvn.cmd" } else { "mvn" });
            
            if mvn_path.exists() {
                return Some(mvn_path);
            }
        }

        // Try to find mvn in PATH
        which::which("mvn").ok()
    }

    /// Check if Maven is available
    pub fn is_available() -> bool {
        Self::find_maven().is_some()
    }
}

