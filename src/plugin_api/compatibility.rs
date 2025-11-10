use std::collections::HashMap;
use anyhow::Result;
use crate::plugin_api::{Plugin, PluginDescriptor};
use crate::model::build::Plugin as ModelPlugin;

/// Plugin configuration inheritance handler
pub struct PluginCompatibility;

impl PluginCompatibility {
    /// Merge plugin configurations from parent and child
    pub fn merge_configurations(
        parent_config: Option<&serde_json::Value>,
        child_config: Option<&serde_json::Value>,
    ) -> Option<serde_json::Value> {
        match (parent_config, child_config) {
            (None, None) => None,
            (Some(p), None) => Some(p.clone()),
            (None, Some(c)) => Some(c.clone()),
            (Some(p), Some(c)) => {
                // Merge JSON objects (child overrides parent)
                if let (Some(p_obj), Some(c_obj)) = (p.as_object(), c.as_object()) {
                    let mut merged = p_obj.clone();
                    for (key, value) in c_obj {
                        merged.insert(key.clone(), value.clone());
                    }
                    Some(serde_json::Value::Object(merged))
                } else {
                    // Child takes precedence for non-objects
                    Some(c.clone())
                }
            }
        }
    }

    /// Apply plugin configuration inheritance
    pub fn apply_inheritance(
        plugin: &ModelPlugin,
        parent_plugin: Option<&ModelPlugin>,
    ) -> ModelPlugin {
        let mut merged = plugin.clone();

        if let Some(parent) = parent_plugin {
            // Inherit version if not specified
            if merged.version.is_none() {
                merged.version = parent.version.clone();
            }

            // Inherit groupId if not specified
            if merged.group_id.is_none() {
                merged.group_id = parent.group_id.clone();
            }

            // Merge configurations
            merged.configuration = Self::merge_configurations(
                parent.configuration.as_ref(),
                merged.configuration.as_ref(),
            );

            // Merge dependencies
            if merged.dependencies.is_none() && parent.dependencies.is_some() {
                merged.dependencies = parent.dependencies.clone();
            } else if let (Some(ref mut child_deps), Some(ref parent_deps)) = 
                (merged.dependencies.as_mut(), parent.dependencies.as_ref()) {
                // Merge dependency lists (child first)
                let mut all_deps = child_deps.clone();
                for parent_dep in parent_deps.iter() {
                    if !all_deps.iter().any(|d| 
                        d.group_id == parent_dep.group_id && 
                        d.artifact_id == parent_dep.artifact_id
                    ) {
                        all_deps.push(parent_dep.clone());
                    }
                }
                merged.dependencies = Some(all_deps);
            }

            // Merge executions
            if merged.executions.is_none() && parent.executions.is_some() {
                merged.executions = parent.executions.clone();
            } else if let (Some(ref mut child_execs), Some(ref parent_execs)) = 
                (merged.executions.as_mut(), parent.executions.as_ref()) {
                // Merge executions (child first)
                let mut all_execs = child_execs.clone();
                for parent_exec in parent_execs.iter() {
                    if !all_execs.iter().any(|e| e.id == parent_exec.id) {
                        all_execs.push(parent_exec.clone());
                    }
                }
                merged.executions = Some(all_execs);
            }
        }

        merged
    }

    /// Create a compatibility layer for legacy plugins
    pub fn create_compatibility_layer(
        descriptor: &PluginDescriptor,
    ) -> Result<HashMap<String, String>> {
        let mut compatibility = HashMap::new();

        // Map plugin descriptor to compatibility properties
        compatibility.insert("plugin.groupId".to_string(), descriptor.group_id.clone());
        compatibility.insert("plugin.artifactId".to_string(), descriptor.artifact_id.clone());
        compatibility.insert("plugin.version".to_string(), descriptor.version.clone());
        
        if let Some(ref name) = descriptor.name {
            compatibility.insert("plugin.name".to_string(), name.clone());
        }
        
        if let Some(ref description) = descriptor.description {
            compatibility.insert("plugin.description".to_string(), description.clone());
        }

        Ok(compatibility)
    }
}

