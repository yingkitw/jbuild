use serde::{Deserialize, Serialize};

use crate::settings::mirror::Mirror;
use crate::settings::profile::Profile;
use crate::settings::server::Server;

/// Maven settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Local repository path
    pub local_repository: Option<String>,

    /// Interactive mode
    pub interactive_mode: Option<bool>,

    /// Offline mode
    pub offline: Option<bool>,

    /// Plugin groups
    #[serde(rename = "pluginGroups")]
    pub plugin_groups: Option<Vec<String>>,

    /// Servers
    #[serde(rename = "servers")]
    pub servers: Option<Vec<Server>>,

    /// Mirrors
    #[serde(rename = "mirrors")]
    pub mirrors: Option<Vec<Mirror>>,

    /// Profiles
    #[serde(rename = "profiles")]
    pub profiles: Option<Vec<Profile>>,

    /// Active profiles
    #[serde(rename = "activeProfiles")]
    pub active_profiles: Option<Vec<String>>,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            local_repository: None,
            interactive_mode: Some(true),
            offline: Some(false),
            plugin_groups: None,
            servers: None,
            mirrors: None,
            profiles: None,
            active_profiles: None,
        }
    }
}

