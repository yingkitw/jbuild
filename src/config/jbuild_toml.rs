//! jbuild.toml configuration parsing and conversion helpers.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use crate::config::write_lock_file;

/// Top-level jbuild.toml structure
#[derive(Debug, Deserialize)]
pub struct JbuildConfig {
    pub package: PackageSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: BTreeMap<String, String>,
}

/// Package section
#[derive(Debug, Deserialize)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub java: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl JbuildConfig {
    /// Parse a jbuild.toml file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let cfg: JbuildConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        // Write lock file next to jbuild.toml if absent
        let lock_path = path.with_file_name("jbuild.lock");
        if !lock_path.exists() {
            write_lock_file(&lock_path, &cfg)
                .with_context(|| format!("Failed to generate {}", lock_path.display()))?;
        }

        Ok(cfg)
    }

    /// Convert to a simple pom.xml string so we can reuse the Maven pipeline.
    pub fn to_pom_xml(&self) -> String {
        let java_version = self.package.java.as_deref().unwrap_or("17");
        let description = self
            .package
            .description
            .as_deref()
            .unwrap_or("Project generated from jbuild.toml");

        let mut deps_xml = String::new();
        for (ga, ver) in &self.dependencies {
            let parts: Vec<&str> = ga.split(':').collect();
            if parts.len() >= 2 {
                deps_xml.push_str(&format!(
                    "        <dependency>\n            <groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>\n        </dependency>\n",
                    parts[0], parts[1], ver
                ));
            }
        }
        for (ga, ver) in &self.dev_dependencies {
            let parts: Vec<&str> = ga.split(':').collect();
            if parts.len() >= 2 {
                deps_xml.push_str(&format!(
                    "        <dependency>\n            <groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>\n            <scope>test</scope>\n        </dependency>\n",
                    parts[0], parts[1], ver
                ));
            }
        }

        format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>{name}</groupId>
    <artifactId>{name}</artifactId>
    <version>{version}</version>
    <packaging>jar</packaging>
    <name>{name}</name>
    <description>{description}</description>
    <properties>
        <maven.compiler.source>{java_version}</maven.compiler.source>
        <maven.compiler.target>{java_version}</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>
    <dependencies>
{deps}    </dependencies>
</project>
"#,
            name = self.package.name,
            version = self.package.version,
            description = description,
            java_version = java_version,
            deps = deps_xml
        )
    }
}

