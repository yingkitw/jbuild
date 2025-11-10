use serde::{Deserialize, Serialize};

/// Repository mirror
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Mirror {
    pub id: String,
    pub name: Option<String>,
    pub url: String,
    pub mirror_of: String,
}

