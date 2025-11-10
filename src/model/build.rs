use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::dependency::Dependency;

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    /// The source directory.
    pub source_directory: Option<String>,

    /// The script source directory.
    pub script_source_directory: Option<String>,

    /// The test source directory.
    pub test_source_directory: Option<String>,

    /// The output directory.
    pub output_directory: Option<String>,

    /// The test output directory.
    pub test_output_directory: Option<String>,

    /// Build extensions.
    #[serde(rename = "extensions")]
    pub extensions: Option<Vec<Extension>>,

    /// Default goal to execute when none is specified.
    pub default_goal: Option<String>,

    /// Build resources.
    #[serde(rename = "resources")]
    pub resources: Option<Vec<Resource>>,

    /// Test resources.
    #[serde(rename = "testResources")]
    pub test_resources: Option<Vec<Resource>>,

    /// Build plugins.
    #[serde(rename = "plugins")]
    pub plugins: Option<Vec<Plugin>>,

    /// Plugin management.
    #[serde(rename = "pluginManagement")]
    pub plugin_management: Option<PluginManagement>,

    /// Build directory.
    pub directory: Option<String>,

    /// Final name of the artifact.
    pub final_name: Option<String>,

    /// Filters for resources.
    #[serde(rename = "filters")]
    pub filters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub target_path: Option<String>,
    pub filtering: Option<bool>,
    pub directory: Option<String>,
    #[serde(rename = "includes")]
    pub includes: Option<Vec<String>>,
    #[serde(rename = "excludes")]
    pub excludes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub group_id: Option<String>,
    pub artifact_id: String,
    pub version: Option<String>,
    #[serde(rename = "extensions")]
    pub extensions: Option<bool>,
    pub executions: Option<Vec<Execution>>,
    pub dependencies: Option<Vec<Dependency>>,
    pub goals: Option<HashMap<String, String>>,
    pub inherited: Option<bool>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub id: Option<String>,
    pub phase: Option<String>,
    #[serde(rename = "goals")]
    pub goals: Option<Vec<String>>,
    pub configuration: Option<serde_json::Value>,
    pub inherited: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManagement {
    #[serde(rename = "plugins")]
    pub plugins: Option<Vec<Plugin>>,
}

impl Build {
    /// Get the default source directory
    pub fn source_directory(&self) -> &str {
        self.source_directory
            .as_deref()
            .unwrap_or("src/main/java")
    }

    /// Get the default test source directory
    pub fn test_source_directory(&self) -> &str {
        self.test_source_directory
            .as_deref()
            .unwrap_or("src/test/java")
    }

    /// Get the default output directory
    pub fn output_directory(&self) -> &str {
        self.output_directory
            .as_deref()
            .unwrap_or("target/classes")
    }

    /// Get the default test output directory
    pub fn test_output_directory(&self) -> &str {
        self.test_output_directory
            .as_deref()
            .unwrap_or("target/test-classes")
    }
}

