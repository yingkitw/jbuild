//! Project initialization application service
//! Creates new projects with proper structure and configuration

use crate::domain::build_system::value_objects::BuildSystemType;
use crate::domain::shared::value_objects::JavaVersion;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

/// Project initialization service
/// Creates new projects with appropriate structure
pub struct ProjectInitializationService;

impl ProjectInitializationService {
    /// Create a new project
    pub fn create_project(
        project_dir: &Path,
        name: &str,
        group_id: &str,
        build_system: BuildSystemType,
        java_version: JavaVersion,
    ) -> Result<()> {
        if project_dir.exists() {
            return Err(anyhow!("Directory already exists: {:?}", project_dir));
        }
        
        fs::create_dir_all(project_dir)?;
        
        match build_system {
            BuildSystemType::Maven => {
                Self::create_maven_project(project_dir, name, group_id, java_version)
            }
            BuildSystemType::Gradle => {
                Self::create_gradle_project(project_dir, name, group_id, java_version)
            }
            BuildSystemType::JBuild => {
                Self::create_jbuild_project(project_dir, name, group_id, java_version)
            }
        }
    }
    
    fn create_maven_project(
        project_dir: &Path,
        name: &str,
        group_id: &str,
        java_version: JavaVersion,
    ) -> Result<()> {
        let pom_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>1.0.0</version>
    
    <properties>
        <maven.compiler.source>{}</maven.compiler.source>
        <maven.compiler.target>{}</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>
    
    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>
"#,
            group_id, name, java_version, java_version
        );
        
        fs::write(project_dir.join("pom.xml"), pom_content)?;
        
        Self::create_standard_directories(project_dir)?;
        Self::create_sample_java_file(project_dir, group_id, name)?;
        
        Ok(())
    }
    
    fn create_gradle_project(
        project_dir: &Path,
        name: &str,
        group_id: &str,
        java_version: JavaVersion,
    ) -> Result<()> {
        let build_gradle = format!(
            r#"plugins {{
    id 'java'
}}

group = '{}'
version = '1.0.0'

java {{
    sourceCompatibility = JavaVersion.VERSION_{}
    targetCompatibility = JavaVersion.VERSION_{}
}}

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation 'junit:junit:4.13.2'
}}
"#,
            group_id,
            format!("{}", java_version).replace('.', "_"),
            format!("{}", java_version).replace('.', "_")
        );
        
        fs::write(project_dir.join("build.gradle"), build_gradle)?;
        
        let settings_gradle = format!("rootProject.name = '{}'", name);
        fs::write(project_dir.join("settings.gradle"), settings_gradle)?;
        
        Self::create_standard_directories(project_dir)?;
        Self::create_sample_java_file(project_dir, group_id, name)?;
        
        Ok(())
    }
    
    fn create_jbuild_project(
        project_dir: &Path,
        name: &str,
        group_id: &str,
        java_version: JavaVersion,
    ) -> Result<()> {
        let jbuild_toml = format!(
            r#"[project]
name = "{}"
group = "{}"
version = "1.0.0"

[java]
version = "{}"

[dependencies]
junit = {{ group = "junit", name = "junit", version = "4.13.2", scope = "test" }}
"#,
            name, group_id, java_version
        );
        
        fs::write(project_dir.join("jbuild.toml"), jbuild_toml)?;
        
        Self::create_standard_directories(project_dir)?;
        Self::create_sample_java_file(project_dir, group_id, name)?;
        
        Ok(())
    }
    
    fn create_standard_directories(project_dir: &Path) -> Result<()> {
        fs::create_dir_all(project_dir.join("src/main/java"))?;
        fs::create_dir_all(project_dir.join("src/main/resources"))?;
        fs::create_dir_all(project_dir.join("src/test/java"))?;
        fs::create_dir_all(project_dir.join("src/test/resources"))?;
        Ok(())
    }
    
    fn create_sample_java_file(project_dir: &Path, group_id: &str, _name: &str) -> Result<()> {
        let package_path = group_id.replace('.', "/");
        let java_dir = project_dir.join("src/main/java").join(&package_path);
        fs::create_dir_all(&java_dir)?;
        
        let app_java = format!(
            r#"package {};

public class App {{
    public static void main(String[] args) {{
        System.out.println("Hello, World!");
    }}
}}
"#,
            group_id
        );
        
        fs::write(java_dir.join("App.java"), app_java)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_maven_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        
        let result = ProjectInitializationService::create_project(
            &project_dir,
            "test-project",
            "com.example",
            BuildSystemType::Maven,
            JavaVersion::new(17, 0, 0),
        );
        
        assert!(result.is_ok());
        assert!(project_dir.join("pom.xml").exists());
        assert!(project_dir.join("src/main/java").exists());
        assert!(project_dir.join("src/test/java").exists());
    }

    #[test]
    fn test_create_gradle_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        
        let result = ProjectInitializationService::create_project(
            &project_dir,
            "test-project",
            "com.example",
            BuildSystemType::Gradle,
            JavaVersion::new(17, 0, 0),
        );
        
        assert!(result.is_ok());
        assert!(project_dir.join("build.gradle").exists());
        assert!(project_dir.join("settings.gradle").exists());
        assert!(project_dir.join("src/main/java").exists());
    }

    #[test]
    fn test_create_jbuild_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        
        let result = ProjectInitializationService::create_project(
            &project_dir,
            "test-project",
            "com.example",
            BuildSystemType::JBuild,
            JavaVersion::new(17, 0, 0),
        );
        
        assert!(result.is_ok());
        assert!(project_dir.join("jbuild.toml").exists());
        assert!(project_dir.join("src/main/java").exists());
    }

    #[test]
    fn test_create_project_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = ProjectInitializationService::create_project(
            temp_dir.path(),
            "test",
            "com.example",
            BuildSystemType::Maven,
            JavaVersion::new(17, 0, 0),
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sample_java_file_created() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        
        ProjectInitializationService::create_project(
            &project_dir,
            "test-project",
            "com.example",
            BuildSystemType::Maven,
            JavaVersion::new(17, 0, 0),
        ).unwrap();
        
        let app_java = project_dir.join("src/main/java/com/example/App.java");
        assert!(app_java.exists());
        
        let content = fs::read_to_string(app_java).unwrap();
        assert!(content.contains("package com.example"));
        assert!(content.contains("public class App"));
    }
}
