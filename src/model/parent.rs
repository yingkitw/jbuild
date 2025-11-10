use serde::{Deserialize, Serialize};

/// Parent project reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Parent {
    /// The group ID of the parent project.
    pub group_id: String,

    /// The artifact ID of the parent project.
    pub artifact_id: String,

    /// The version of the parent project.
    pub version: String,

    /// The relative path of the parent pom.xml file from the current project.
    pub relative_path: Option<String>,
}

impl Parent {
    /// Get the parent coordinates as a tuple (groupId, artifactId, version)
    pub fn coordinates(&self) -> (&str, &str, &str) {
        (&self.group_id, &self.artifact_id, &self.version)
    }

    /// Get the parent identifier (groupId:artifactId)
    pub fn id(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }

    /// Get the full parent identifier (groupId:artifactId:version)
    pub fn full_id(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

