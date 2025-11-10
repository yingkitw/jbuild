use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Settings profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub properties: Option<HashMap<String, String>>,
}

