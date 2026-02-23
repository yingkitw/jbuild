//! Maven to Gradle migration utility
//!
//! Converts Maven pom.xml to Gradle build.gradle

use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use crate::model::{Model, Dependency};

/// Maven to Gradle migrator
pub struct MavenToGradleMigrator;

impl MavenToGradleMigrator {
    /// Convert pom.xml to build.gradle
    pub fn migrate(pom_path: &Path) -> Result<String> {
        let pom_content = fs::read_to_string(pom_path)
            .context("Failed to read pom.xml")?;
        
        let model = crate::model::parser::parse_pom(&pom_content)
            .context("Failed to parse pom.xml")?;

        Ok(Self::generate_gradle_build(&model))
    }

    /// Generate Gradle build script from Maven model
    fn generate_gradle_build(model: &Model) -> String {
        let mut gradle = String::new();

        // Plugins
        gradle.push_str("plugins {\n");
        gradle.push_str("    id 'java'\n");
        
        // Detect if it's a Spring Boot project
        if Self::is_spring_boot(model) {
            gradle.push_str("    id 'org.springframework.boot' version '3.2.0'\n");
            gradle.push_str("    id 'io.spring.dependency-management' version '1.1.4'\n");
        }
        
        gradle.push_str("}\n\n");

        // Group and version
        gradle.push_str(&format!("group = '{}'\n", model.group_id));
        gradle.push_str(&format!("version = '{}'\n\n", model.version));

        // Java version
        if let Some(props) = &model.properties {
            if let Some(source) = props.get("maven.compiler.source") {
                gradle.push_str("java {\n");
                gradle.push_str(&format!("    sourceCompatibility = '{}'\n", source));
                if let Some(target) = props.get("maven.compiler.target") {
                    gradle.push_str(&format!("    targetCompatibility = '{}'\n", target));
                }
                gradle.push_str("}\n\n");
            }
        }

        // Repositories
        gradle.push_str("repositories {\n");
        gradle.push_str("    mavenCentral()\n");
        
        if let Some(repos) = &model.repositories {
            for repo in &repos.repositories {
                if repo.url != "https://repo1.maven.org/maven2/" 
                    && repo.url != "https://repo.maven.apache.org/maven2/" {
                    gradle.push_str(&format!("    maven {{ url '{}' }}\n", repo.url));
                }
            }
        }
        
        gradle.push_str("}\n\n");

        // Dependencies
        if let Some(deps) = &model.dependencies {
            gradle.push_str("dependencies {\n");
            
            for dep in &deps.dependencies {
                let config = Self::map_scope_to_configuration(&dep.scope);
                let version = dep.version.as_deref().unwrap_or("");
                
                gradle.push_str(&format!(
                    "    {} '{}:{}:{}'\n",
                    config,
                    dep.group_id,
                    dep.artifact_id,
                    version
                ));
            }
            
            gradle.push_str("}\n\n");
        }

        // Test configuration
        gradle.push_str("test {\n");
        gradle.push_str("    useJUnitPlatform()\n");
        gradle.push_str("}\n");

        gradle
    }

    fn map_scope_to_configuration(scope: &Option<String>) -> &str {
        match scope.as_deref() {
            Some("compile") | None => "implementation",
            Some("provided") => "compileOnly",
            Some("runtime") => "runtimeOnly",
            Some("test") => "testImplementation",
            Some("system") => "implementation",
            _ => "implementation",
        }
    }

    fn is_spring_boot(model: &Model) -> bool {
        if let Some(parent) = &model.parent {
            if parent.artifact_id == "spring-boot-starter-parent" {
                return true;
            }
        }
        
        if let Some(deps) = &model.dependencies {
            for dep in &deps.dependencies {
                if dep.group_id == "org.springframework.boot" {
                    return true;
                }
            }
        }
        
        false
    }

    /// Generate settings.gradle
    pub fn generate_settings(model: &Model) -> String {
        format!("rootProject.name = '{}'\n", model.artifact_id)
    }

    /// Generate gradle.properties
    pub fn generate_properties(model: &Model) -> String {
        let mut props = String::new();
        
        if let Some(properties) = &model.properties {
            for (key, value) in properties {
                if !key.starts_with("maven.") {
                    props.push_str(&format!("{}={}\n", key, value));
                }
            }
        }
        
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_mapping() {
        assert_eq!(MavenToGradleMigrator::map_scope_to_configuration(&None), "implementation");
        assert_eq!(MavenToGradleMigrator::map_scope_to_configuration(&Some("test".to_string())), "testImplementation");
        assert_eq!(MavenToGradleMigrator::map_scope_to_configuration(&Some("provided".to_string())), "compileOnly");
    }
}
