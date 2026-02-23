//! Build configuration converter

use std::path::Path;
use anyhow::Result;

/// Generic build configuration converter
pub struct BuildConverter;

impl BuildConverter {
    /// Detect build system and convert to jbuild
    pub fn convert_to_jbuild(project_dir: &Path) -> Result<ConversionResult> {
        let build_system = Self::detect_build_system(project_dir)?;

        match build_system {
            BuildSystem::Maven => {
                let pom_path = project_dir.join("pom.xml");
                if pom_path.exists() {
                    let result = super::maven_to_jbuild::MavenMigrator::migrate(&pom_path)?;
                    Ok(ConversionResult {
                        source_system: "Maven".to_string(),
                        config: result.config,
                        warnings: result.warnings,
                        notes: result.notes,
                    })
                } else {
                    Err(anyhow::anyhow!("pom.xml not found"))
                }
            }
            BuildSystem::Gradle => {
                let build_gradle = project_dir.join("build.gradle");
                let build_gradle_kts = project_dir.join("build.gradle.kts");

                let gradle_file = if build_gradle.exists() {
                    &build_gradle
                } else if build_gradle_kts.exists() {
                    &build_gradle_kts
                } else {
                    return Err(anyhow::anyhow!("build.gradle file not found"));
                };

                let result = super::gradle_to_jbuild::GradleMigrator::migrate(gradle_file)?;
                Ok(ConversionResult {
                    source_system: "Gradle".to_string(),
                    config: result.config,
                    warnings: result.warnings,
                    notes: result.notes,
                })
            }
            BuildSystem::Unknown => {
                Err(anyhow::anyhow!("Unable to detect build system"))
            }
        }
    }

    fn detect_build_system(project_dir: &Path) -> Result<BuildSystem> {
        if project_dir.join("pom.xml").exists() {
            Ok(BuildSystem::Maven)
        } else if project_dir.join("build.gradle").exists()
            || project_dir.join("build.gradle.kts").exists()
        {
            Ok(BuildSystem::Gradle)
        } else {
            Ok(BuildSystem::Unknown)
        }
    }

    /// Write converted config to file
    pub fn write_config(project_dir: &Path, config: &str) -> Result<()> {
        let config_path = project_dir.join("jbuild.toml");
        std::fs::write(&config_path, config)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildSystem {
    Maven,
    Gradle,
    Unknown,
}

/// Result of build conversion
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub source_system: String,
    pub config: String,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_build_system() {
        // Would need actual file system for testing
    }
}
