use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::build::Build;
use crate::model::dependency::Dependency;
use crate::model::distribution::DistributionManagement;
use crate::model::parent::Parent;
use crate::model::profile::Profile;
use crate::model::repository::Repository;

/// The root element of a Maven POM (Project Object Model)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename = "project")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Declares to which version of project descriptor this POM conforms.
    #[serde(rename = "modelVersion")]
    pub model_version: String,

    /// The location of the parent project, if one exists.
    pub parent: Option<Parent>,

    /// A universally unique identifier for a project.
    #[serde(rename = "groupId")]
    pub group_id: String,

    /// The identifier for this artifact that is unique within the group.
    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    /// The current version of the artifact produced by this project.
    pub version: String,

    /// The type of artifact this project produces (jar, war, pom, etc.)
    #[serde(default = "default_packaging")]
    pub packaging: String,

    /// The full name of the project.
    pub name: Option<String>,

    /// A detailed description of the project.
    pub description: Option<String>,

    /// The URL to the project's homepage.
    pub url: Option<String>,

    /// The year of the project's inception.
    pub inception_year: Option<String>,

    /// Organization information.
    pub organization: Option<Organization>,

    /// License information.
    #[serde(rename = "licenses")]
    pub licenses: Option<Vec<License>>,

    /// Developer information.
    #[serde(rename = "developers")]
    pub developers: Option<Vec<Developer>>,

    /// Contributor information.
    #[serde(rename = "contributors")]
    pub contributors: Option<Vec<Contributor>>,

    /// Project dependencies.
    #[serde(rename = "dependencies", default)]
    pub dependencies: Option<Dependencies>,

    /// Dependency management section.
    #[serde(rename = "dependencyManagement")]
    pub dependency_management: Option<DependencyManagement>,

    /// Build configuration.
    pub build: Option<Build>,

    /// Distribution management.
    #[serde(rename = "distributionManagement")]
    pub distribution_management: Option<DistributionManagement>,

    /// Project properties.
    pub properties: Option<HashMap<String, String>>,

    /// Project modules (for multi-module projects).
    #[serde(rename = "modules")]
    pub modules: Option<Vec<String>>,

    /// Project profiles.
    #[serde(rename = "profiles")]
    pub profiles: Option<Vec<Profile>>,

    /// Repositories for dependency resolution.
    #[serde(rename = "repositories")]
    pub repositories: Option<Vec<Repository>>,

    /// Plugin repositories.
    #[serde(rename = "pluginRepositories")]
    pub plugin_repositories: Option<Vec<Repository>>,

    /// Reporting configuration.
    pub reporting: Option<Reporting>,
}

fn default_packaging() -> String {
    "jar".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct License {
    pub name: Option<String>,
    pub url: Option<String>,
    pub distribution: Option<String>,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Developer {
    pub id: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub organization: Option<String>,
    pub organization_url: Option<String>,
    pub roles: Option<Vec<String>>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contributor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub organization: Option<String>,
    pub organization_url: Option<String>,
    pub roles: Option<Vec<String>>,
    pub timezone: Option<String>,
}

/// Wrapper for dependencies list
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Dependencies {
    #[serde(rename = "dependency", default)]
    pub dependencies: Vec<Dependency>,
}

impl Dependencies {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }
    
    pub fn into_vec(self) -> Vec<Dependency> {
        self.dependencies
    }
}

impl From<Vec<Dependency>> for Dependencies {
    fn from(deps: Vec<Dependency>) -> Self {
        Self { dependencies: deps }
    }
}

impl From<Dependencies> for Vec<Dependency> {
    fn from(deps: Dependencies) -> Self {
        deps.dependencies
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyManagement {
    #[serde(rename = "dependencies", default)]
    pub dependencies: Option<Dependencies>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reporting {
    #[serde(rename = "plugins")]
    pub plugins: Option<Vec<ReportingPlugin>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportingPlugin {
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
    pub version: Option<String>,
}

impl Model {
    /// Get the project coordinates as a tuple (groupId, artifactId, version)
    pub fn coordinates(&self) -> (&str, &str, &str) {
        (&self.group_id, &self.artifact_id, &self.version)
    }

    /// Get the project identifier (groupId:artifactId)
    pub fn id(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }

    /// Get the full project identifier (groupId:artifactId:version)
    pub fn full_id(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
    
    /// Get dependencies as a Vec
    pub fn dependencies_vec(&self) -> Vec<Dependency> {
        self.dependencies.as_ref()
            .map(|deps| deps.dependencies.clone())
            .unwrap_or_default()
    }
}

