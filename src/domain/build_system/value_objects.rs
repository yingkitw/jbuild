//! Value objects for Build System context

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Build system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildSystemType {
    Maven,
    Gradle,
    JBuild,
}

impl BuildSystemType {
    pub fn as_str(&self) -> &str {
        match self {
            BuildSystemType::Maven => "Maven",
            BuildSystemType::Gradle => "Gradle",
            BuildSystemType::JBuild => "JBuild",
        }
    }
}

/// Build file value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildFile {
    path: PathBuf,
    build_type: BuildSystemType,
}

impl BuildFile {
    pub fn new(path: PathBuf, build_type: BuildSystemType) -> Self {
        Self { path, build_type }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn build_type(&self) -> BuildSystemType {
        self.build_type
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Goal or task name value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalOrTask(String);

impl GoalOrTask {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for GoalOrTask {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for GoalOrTask {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
