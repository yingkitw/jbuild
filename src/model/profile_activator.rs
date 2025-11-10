use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;

use crate::model::profile::{Profile, Activation, ActivationOS, ActivationProperty, ActivationFile};

/// Context for profile activation
#[derive(Debug, Clone)]
pub struct ProfileActivationContext {
    pub active_profile_ids: Vec<String>,
    pub inactive_profile_ids: Vec<String>,
    pub system_properties: HashMap<String, String>,
    pub user_properties: HashMap<String, String>,
    pub project_properties: HashMap<String, String>,
    pub project_directory: Option<PathBuf>,
}

impl ProfileActivationContext {
    pub fn new() -> Self {
        Self {
            active_profile_ids: Vec::new(),
            inactive_profile_ids: Vec::new(),
            system_properties: Self::get_system_properties(),
            user_properties: HashMap::new(),
            project_properties: HashMap::new(),
            project_directory: None,
        }
    }

    pub fn with_active_profiles(mut self, profiles: Vec<String>) -> Self {
        self.active_profile_ids = profiles;
        self
    }

    pub fn with_user_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.user_properties = properties;
        self
    }

    pub fn with_project_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.project_properties = properties;
        self
    }

    pub fn with_project_directory(mut self, directory: PathBuf) -> Self {
        self.project_directory = Some(directory);
        self
    }

    /// Get system properties from environment
    fn get_system_properties() -> HashMap<String, String> {
        let mut props = HashMap::new();
        
        // Add Java system properties
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            props.insert("java.home".to_string(), java_home);
        }
        
        // Add OS properties
        props.insert("os.name".to_string(), std::env::consts::OS.to_string());
        props.insert("os.arch".to_string(), std::env::consts::ARCH.to_string());
        props.insert("os.version".to_string(), Self::get_os_version());
        
        // Add environment variables
        for (key, value) in std::env::vars() {
            props.insert(format!("env.{}", key), value);
        }
        
        props
    }

    fn get_os_version() -> String {
        // Try to get OS version
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    return version.trim().to_string();
                }
            }
        }
        
        // Fallback
        "unknown".to_string()
    }
}

impl Default for ProfileActivationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Profile activator
pub struct ProfileActivator;

impl ProfileActivator {
    /// Check if a profile should be activated
    pub fn is_active(profile: &Profile, context: &ProfileActivationContext) -> bool {
        // Check explicit activation/deactivation
        if context.inactive_profile_ids.contains(&profile.id) {
            return false;
        }
        
        if context.active_profile_ids.contains(&profile.id) {
            return true;
        }

        // Check activation conditions
        if let Some(ref activation) = profile.activation {
            Self::check_activation(activation, context)
        } else {
            false
        }
    }

    /// Check activation conditions
    fn check_activation(activation: &Activation, context: &ProfileActivationContext) -> bool {
        // Check activeByDefault
        if activation.active_by_default == Some(true) {
            return true;
        }

        // Check JDK version
        if let Some(ref jdk) = activation.jdk {
            if !Self::check_jdk_version(jdk) {
                return false;
            }
        }

        // Check OS
        if let Some(ref os) = activation.os {
            if !Self::check_os(os) {
                return false;
            }
        }

        // Check property
        if let Some(ref property) = activation.property {
            if !Self::check_property(property, context) {
                return false;
            }
        }

        // Check file
        if let Some(ref file) = activation.file {
            if !Self::check_file(file, &context.project_directory) {
                return false;
            }
        }

        true
    }

    /// Check JDK version activation
    fn check_jdk_version(jdk_spec: &str) -> bool {
        // Parse JDK version specification (e.g., "1.8", "[1.8,)", "[1.8,1.9)")
        // For now, simple check
        if let Ok(java_version) = std::env::var("JAVA_VERSION") {
            return jdk_spec == java_version || java_version.starts_with(jdk_spec);
        }
        
        // Try to get Java version
        use std::process::Command;
        if let Ok(output) = Command::new("java").arg("-version").output() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Simple heuristic: check if version string contains the spec
            return stderr.contains(jdk_spec);
        }
        
        false
    }

    /// Check OS activation
    fn check_os(os: &ActivationOS) -> bool {
        if let Some(ref name) = os.name {
            if std::env::consts::OS != name {
                return false;
            }
        }

        if let Some(ref family) = os.family {
            let current_family = match std::env::consts::OS {
                "windows" => "windows",
                "macos" => "mac",
                "linux" => "unix",
                _ => "unknown",
            };
            if current_family != family {
                return false;
            }
        }

        if let Some(ref arch) = os.arch {
            if std::env::consts::ARCH != arch {
                return false;
            }
        }

        true
    }

    /// Check property activation
    fn check_property(property: &ActivationProperty, context: &ProfileActivationContext) -> bool {
        if let Some(ref name) = property.name {
            // Check all property sources
            let value = context.user_properties.get(name)
                .or_else(|| context.system_properties.get(name))
                .or_else(|| context.project_properties.get(name))
                .map(|s| s.as_str());

            if let Some(ref expected_value) = property.value {
                return value == Some(expected_value.as_str());
            } else {
                // Property exists check
                return value.is_some();
            }
        }

        false
    }

    /// Check file activation
    fn check_file(file: &ActivationFile, project_dir: &Option<PathBuf>) -> bool {
        if let Some(ref dir) = project_dir {
            if let Some(ref missing) = file.missing {
                let path = dir.join(missing);
                return !path.exists();
            }

            if let Some(ref exists) = file.exists {
                let path = dir.join(exists);
                return path.exists();
            }
        }

        false
    }

    /// Get active profiles from a list
    pub fn get_active_profiles(
        profiles: &[Profile],
        context: &ProfileActivationContext,
    ) -> Vec<Profile> {
        profiles
            .iter()
            .filter(|p| Self::is_active(p, context))
            .cloned()
            .collect()
    }
}

