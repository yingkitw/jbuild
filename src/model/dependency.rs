use serde::{Deserialize, Serialize};

/// Project dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    /// The group ID of the dependency.
    #[serde(rename = "groupId")]
    pub group_id: String,

    /// The artifact ID of the dependency.
    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    /// The version of the dependency.
    pub version: Option<String>,

    /// The type of dependency (jar, war, pom, etc.)
    #[serde(rename = "type")]
    pub type_: Option<String>,

    /// The classifier of the dependency.
    pub classifier: Option<String>,

    /// The scope of the dependency (compile, provided, runtime, test, system).
    pub scope: Option<String>,

    /// Whether this dependency is optional.
    pub optional: Option<bool>,

    /// Exclusions for transitive dependencies.
    #[serde(rename = "exclusions")]
    pub exclusions: Option<Vec<Exclusion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exclusion {
    /// The group ID to exclude.
    #[serde(rename = "groupId")]
    pub group_id: String,

    /// The artifact ID to exclude.
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
}

impl Dependency {
    /// Get the dependency coordinates as a tuple (groupId, artifactId, version)
    pub fn coordinates(&self) -> (&str, &str, Option<&str>) {
        (&self.group_id, &self.artifact_id, self.version.as_deref())
    }

    /// Get the dependency identifier (groupId:artifactId)
    pub fn id(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }

    /// Get the full dependency identifier (groupId:artifactId:version)
    pub fn full_id(&self) -> String {
        if let Some(version) = &self.version {
            format!("{}:{}:{}", self.group_id, self.artifact_id, version)
        } else {
            format!("{}:{}", self.group_id, self.artifact_id)
        }
    }

    /// Check if this is a test-scoped dependency
    pub fn is_test_scope(&self) -> bool {
        self.scope.as_deref() == Some("test")
    }

    /// Check if this is a compile-scoped dependency
    pub fn is_compile_scope(&self) -> bool {
        self.scope.as_deref().unwrap_or("compile") == "compile"
    }
}

