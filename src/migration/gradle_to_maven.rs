//! Gradle to Maven migration utility
//!
//! Converts Gradle build.gradle to Maven pom.xml

use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use crate::gradle::GradleProject;

/// Gradle to Maven migrator
pub struct GradleToMavenMigrator;

impl GradleToMavenMigrator {
    /// Convert build.gradle to pom.xml
    pub fn migrate(gradle_path: &Path) -> Result<String> {
        let base_dir = gradle_path.parent().unwrap_or_else(|| Path::new("."));
        let project = crate::gradle::parse_gradle_build_script(gradle_path, base_dir)
            .context("Failed to parse build.gradle")?;

        Ok(Self::generate_pom(&project))
    }

    /// Generate pom.xml from Gradle project
    fn generate_pom(project: &GradleProject) -> String {
        let mut pom = String::new();

        // XML declaration
        pom.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        pom.push_str("<project xmlns=\"http://maven.apache.org/POM/4.0.0\"\n");
        pom.push_str("         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
        pom.push_str("         xsi:schemaLocation=\"http://maven.apache.org/POM/4.0.0\n");
        pom.push_str("         http://maven.apache.org/xsd/maven-4.0.0.xsd\">\n");
        pom.push_str("    <modelVersion>4.0.0</modelVersion>\n\n");

        // Coordinates
        pom.push_str(&format!("    <groupId>{}</groupId>\n", 
            project.group.as_deref().unwrap_or("com.example")));
        pom.push_str(&format!("    <artifactId>{}</artifactId>\n", project.name));
        pom.push_str(&format!("    <version>{}</version>\n", 
            project.version.as_deref().unwrap_or("1.0.0")));
        pom.push_str("    <packaging>jar</packaging>\n\n");

        // Properties
        pom.push_str("    <properties>\n");
        
        // Java version from sourceCompatibility
        if let Some(ref java_config) = project.java {
            if let Some(ref source) = java_config.source_compatibility {
                pom.push_str(&format!("        <maven.compiler.source>{}</maven.compiler.source>\n", source));
            }
            if let Some(ref target) = java_config.target_compatibility {
                pom.push_str(&format!("        <maven.compiler.target>{}</maven.compiler.target>\n", target));
            }
        }
        
        pom.push_str("        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>\n");
        pom.push_str("    </properties>\n\n");

        // Repositories
        if !project.repositories.is_empty() {
            pom.push_str("    <repositories>\n");
            for (idx, repo) in project.repositories.iter().enumerate() {
                if repo.url != "https://repo1.maven.org/maven2/" {
                    pom.push_str("        <repository>\n");
                    pom.push_str(&format!("            <id>repo-{}</id>\n", idx));
                    pom.push_str(&format!("            <url>{}</url>\n", repo.url));
                    pom.push_str("        </repository>\n");
                }
            }
            pom.push_str("    </repositories>\n\n");
        }

        // Dependencies
        if !project.dependencies.is_empty() {
            pom.push_str("    <dependencies>\n");
            
            for dep in &project.dependencies {
                if let (Some(group), Some(artifact), Some(version)) = 
                    (&dep.group, &dep.artifact, &dep.version) {
                    pom.push_str("        <dependency>\n");
                    pom.push_str(&format!("            <groupId>{}</groupId>\n", group));
                    pom.push_str(&format!("            <artifactId>{}</artifactId>\n", artifact));
                    pom.push_str(&format!("            <version>{}</version>\n", version));
                    
                    let scope = Self::map_configuration_to_scope(&dep.configuration);
                    if scope != "compile" {
                        pom.push_str(&format!("            <scope>{}</scope>\n", scope));
                    }
                    
                    pom.push_str("        </dependency>\n");
                }
            }
            
            pom.push_str("    </dependencies>\n\n");
        }

        // Build section
        pom.push_str("    <build>\n");
        pom.push_str("        <plugins>\n");
        pom.push_str("            <plugin>\n");
        pom.push_str("                <groupId>org.apache.maven.plugins</groupId>\n");
        pom.push_str("                <artifactId>maven-compiler-plugin</artifactId>\n");
        pom.push_str("                <version>3.11.0</version>\n");
        pom.push_str("            </plugin>\n");
        pom.push_str("            <plugin>\n");
        pom.push_str("                <groupId>org.apache.maven.plugins</groupId>\n");
        pom.push_str("                <artifactId>maven-surefire-plugin</artifactId>\n");
        pom.push_str("                <version>3.1.2</version>\n");
        pom.push_str("            </plugin>\n");
        pom.push_str("        </plugins>\n");
        pom.push_str("    </build>\n");

        pom.push_str("</project>\n");

        pom
    }

    fn map_configuration_to_scope(configuration: &str) -> &str {
        match configuration {
            "implementation" | "api" => "compile",
            "compileOnly" => "provided",
            "runtimeOnly" => "runtime",
            "testImplementation" | "testCompileOnly" => "test",
            _ => "compile",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_mapping() {
        assert_eq!(GradleToMavenMigrator::map_configuration_to_scope("implementation"), "compile");
        assert_eq!(GradleToMavenMigrator::map_configuration_to_scope("testImplementation"), "test");
        assert_eq!(GradleToMavenMigrator::map_configuration_to_scope("compileOnly"), "provided");
    }
}
