use std::path::PathBuf;
use quick_xml::de::from_str;
use thiserror::Error;

use crate::settings::Settings;

#[derive(Debug, Error)]
pub enum SettingsParseError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Parse settings.xml from a string
pub fn parse_settings(settings_content: &str) -> Result<Settings, SettingsParseError> {
    // Normalize XML namespaces similar to POM parsing
    let normalized = normalize_settings_xml(settings_content)?;
    let settings: Settings = from_str(&normalized)?;
    Ok(settings)
}

/// Parse settings.xml from a file path
pub fn parse_settings_file(path: &PathBuf) -> Result<Settings, SettingsParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_settings(&content)
}

/// Load settings from default locations
pub fn load_settings() -> Settings {
    // Try user settings first
    if let Ok(home) = std::env::var("HOME") {
        let user_settings = PathBuf::from(home).join(".m2").join("settings.xml");
        if user_settings.exists() {
            if let Ok(settings) = parse_settings_file(&user_settings) {
                return settings;
            }
        }
    }

    // Try global settings
    if let Ok(maven_home) = std::env::var("M2_HOME") {
        let global_settings = PathBuf::from(maven_home).join("conf").join("settings.xml");
        if global_settings.exists() {
            if let Ok(settings) = parse_settings_file(&global_settings) {
                return settings;
            }
        }
    }

    // Return default settings
    Settings::default()
}

/// Normalize settings XML by removing namespaces
fn normalize_settings_xml(xml: &str) -> Result<String, quick_xml::DeError> {
    // For now, simple approach - in production would need proper namespace handling
    // Similar to POM parser but simplified for settings
    Ok(xml.to_string())
}

