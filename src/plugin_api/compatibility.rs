use std::collections::HashMap;
use anyhow::Result;
use crate::plugin_api::PluginDescriptor;
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
            } else if let (Some(ref mut child_deps), Some(parent_deps)) = 
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
            } else if let (Some(ref mut child_execs), Some(parent_execs)) = 
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::build::Plugin as ModelPlugin;
    use crate::model::Dependency;

    fn create_test_plugin(group_id: &str, artifact_id: &str, version: &str) -> ModelPlugin {
        ModelPlugin {
            group_id: Some(group_id.to_string()),
            artifact_id: artifact_id.to_string(),
            version: Some(version.to_string()),
            extensions: None,
            inherited: None,
            configuration: None,
            dependencies: None,
            executions: None,
            goals: None,
        }
    }

    #[test]
    fn test_merge_configurations_none() {
        let result = PluginCompatibility::merge_configurations(None, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_configurations_parent_only() {
        let parent = serde_json::json!({"key": "value"});
        let result = PluginCompatibility::merge_configurations(Some(&parent), None);
        assert_eq!(result, Some(parent));
    }

    #[test]
    fn test_merge_configurations_child_only() {
        let child = serde_json::json!({"key": "value"});
        let result = PluginCompatibility::merge_configurations(None, Some(&child));
        assert_eq!(result, Some(child));
    }

    #[test]
    fn test_merge_configurations_merge() {
        let parent = serde_json::json!({"parent": "value", "shared": "parent"});
        let child = serde_json::json!({"child": "value", "shared": "child"});
        let result = PluginCompatibility::merge_configurations(Some(&parent), Some(&child));
        
        assert!(result.is_some());
        let merged = result.unwrap();
        assert_eq!(merged["parent"], "value");
        assert_eq!(merged["child"], "value");
        assert_eq!(merged["shared"], "child"); // Child overrides
    }

    #[test]
    fn test_apply_inheritance_version() {
        let parent = create_test_plugin("com.example", "parent-plugin", "1.0.0");
        let mut child = create_test_plugin("com.example", "child-plugin", "");
        child.version = None;

        let merged = PluginCompatibility::apply_inheritance(&child, Some(&parent));
        assert_eq!(merged.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_apply_inheritance_group_id() {
        let parent = create_test_plugin("com.example", "parent-plugin", "1.0.0");
        let mut child = create_test_plugin("", "child-plugin", "1.0.0");
        child.group_id = None;

        let merged = PluginCompatibility::apply_inheritance(&child, Some(&parent));
        assert_eq!(merged.group_id, Some("com.example".to_string()));
    }

    #[test]
    fn test_apply_inheritance_dependencies() {
        let mut parent = create_test_plugin("com.example", "parent-plugin", "1.0.0");
        parent.dependencies = Some(vec![
            Dependency {
                group_id: "com.example".to_string(),
                artifact_id: "dep1".to_string(),
                version: Some("1.0.0".to_string()),
                scope: None,
                optional: None,
                type_: None,
                classifier: None,
                exclusions: None,
            }
        ]);

        let child = create_test_plugin("com.example", "child-plugin", "1.0.0");
        let merged = PluginCompatibility::apply_inheritance(&child, Some(&parent));
        
        assert!(merged.dependencies.is_some());
        assert_eq!(merged.dependencies.unwrap().len(), 1);
    }

    #[test]
    fn test_create_compatibility_layer() {
        let descriptor = PluginDescriptor {
            group_id: "com.example".to_string(),
            artifact_id: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            name: Some("Test Plugin".to_string()),
            description: Some("A test plugin".to_string()),
            goal_prefix: Some("test".to_string()),
            goals: vec![],
            dependencies: vec![],
        };

        let compatibility = PluginCompatibility::create_compatibility_layer(&descriptor).unwrap();
        
        assert_eq!(compatibility.get("plugin.groupId"), Some(&"com.example".to_string()));
        assert_eq!(compatibility.get("plugin.artifactId"), Some(&"test-plugin".to_string()));
        assert_eq!(compatibility.get("plugin.version"), Some(&"1.0.0".to_string()));
        assert_eq!(compatibility.get("plugin.name"), Some(&"Test Plugin".to_string()));
        assert_eq!(compatibility.get("plugin.description"), Some(&"A test plugin".to_string()));
    }
}

