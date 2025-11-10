use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::build::Build;
use crate::model::dependency::Dependency;
use crate::model::repository::Repository;

/// Build profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// Profile identifier.
    pub id: String,

    /// Profile activation configuration.
    pub activation: Option<Activation>,

    /// Profile build configuration.
    pub build: Option<Build>,

    /// Profile dependencies.
    #[serde(rename = "dependencies")]
    pub dependencies: Option<Vec<Dependency>>,

    /// Profile dependency management.
    #[serde(rename = "dependencyManagement")]
    pub dependency_management: Option<ProfileDependencyManagement>,

    /// Profile properties.
    pub properties: Option<HashMap<String, String>>,

    /// Profile repositories.
    #[serde(rename = "repositories")]
    pub repositories: Option<Vec<Repository>>,

    /// Profile plugin repositories.
    #[serde(rename = "pluginRepositories")]
    pub plugin_repositories: Option<Vec<Repository>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Activation {
    /// Activation by default.
    pub active_by_default: Option<bool>,

    /// Activation by JDK version.
    pub jdk: Option<String>,

    /// Activation by OS.
    pub os: Option<ActivationOS>,

    /// Activation by property.
    pub property: Option<ActivationProperty>,

    /// Activation by file.
    pub file: Option<ActivationFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivationOS {
    pub name: Option<String>,
    pub family: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivationProperty {
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivationFile {
    pub missing: Option<String>,
    pub exists: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDependencyManagement {
    #[serde(rename = "dependencies")]
    pub dependencies: Option<Vec<Dependency>>,
}

