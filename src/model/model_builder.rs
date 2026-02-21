use std::collections::HashMap;

use crate::model::{Model, PropertyInterpolator};

/// Model builder - builds effective model from POM with inheritance and interpolation
pub struct ModelBuilder;

impl ModelBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build effective model from base model and parent
    pub fn build_effective_model(&self, model: Model, parent: Option<Model>) -> Model {
        let mut effective = model.clone();

        if let Some(parent_model) = parent {
            // Inherit from parent
            if effective.group_id.is_empty() {
                effective.group_id = parent_model.group_id.clone();
            }
            if effective.version.is_empty() {
                effective.version = parent_model.version.clone();
            }
            if effective.packaging.is_empty() {
                effective.packaging = parent_model.packaging.clone();
            }

            // Merge properties
            let mut properties = effective.properties.unwrap_or_default();
            if let Some(parent_props) = &parent_model.properties {
                for (key, value) in parent_props {
                    properties
                        .entry(key.clone())
                        .or_insert_with(|| value.clone());
                }
            }
            effective.properties = Some(properties);

            // Merge dependency management
            if let Some(parent_dep_mgmt) = &parent_model.dependency_management {
                if effective.dependency_management.is_none() {
                    effective.dependency_management = Some(parent_dep_mgmt.clone());
                } else if let Some(child_dep_mgmt) = &mut effective.dependency_management {
                    if let Some(parent_deps) = &parent_dep_mgmt.dependencies {
                        if child_dep_mgmt.dependencies.is_none() {
                            child_dep_mgmt.dependencies = Some(parent_deps.clone());
                        } else if let Some(child_deps) = &mut child_dep_mgmt.dependencies {
                            for parent_dep in &parent_deps.dependencies {
                                let parent_key =
                                    format!("{}:{}", parent_dep.group_id, parent_dep.artifact_id);
                                if !child_deps.dependencies.iter().any(|d| {
                                    format!("{}:{}", d.group_id, d.artifact_id) == parent_key
                                }) {
                                    child_deps.dependencies.push(parent_dep.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Merge build configuration
            if let Some(parent_build) = &parent_model.build {
                if effective.build.is_none() {
                    effective.build = Some(parent_build.clone());
                } else if let Some(child_build) = &mut effective.build {
                    if child_build.source_directory.is_none() {
                        child_build.source_directory = parent_build.source_directory.clone();
                    }
                    if child_build.test_source_directory.is_none() {
                        child_build.test_source_directory =
                            parent_build.test_source_directory.clone();
                    }
                    if child_build.output_directory.is_none() {
                        child_build.output_directory = parent_build.output_directory.clone();
                    }
                    if child_build.test_output_directory.is_none() {
                        child_build.test_output_directory =
                            parent_build.test_output_directory.clone();
                    }

                    // Merge plugin management
                    if let Some(parent_plugin_mgmt) = &parent_build.plugin_management {
                        if child_build.plugin_management.is_none() {
                            child_build.plugin_management = Some(parent_plugin_mgmt.clone());
                        } else if let Some(child_plugin_mgmt) = &mut child_build.plugin_management {
                            if let Some(parent_plugins) = &parent_plugin_mgmt.plugins {
                                if child_plugin_mgmt.plugins.is_none() {
                                    child_plugin_mgmt.plugins = Some(parent_plugins.clone());
                                } else if let Some(child_plugins) = &mut child_plugin_mgmt.plugins {
                                    for parent_plugin in parent_plugins {
                                        if !child_plugins.iter().any(|p| {
                                            p.group_id == parent_plugin.group_id
                                                && p.artifact_id == parent_plugin.artifact_id
                                        }) {
                                            child_plugins.push(parent_plugin.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        effective
    }

    /// Interpolate model properties
    pub fn interpolate(&self, model: &mut Model, properties: &HashMap<String, String>) {
        let interpolator = PropertyInterpolator::new().add_properties(properties.clone());

        // Interpolate basic info
        if let Ok(val) = interpolator.interpolate(&model.group_id) {
            model.group_id = val;
        }
        if let Ok(val) = interpolator.interpolate(&model.artifact_id) {
            model.artifact_id = val;
        }
        if let Ok(val) = interpolator.interpolate(&model.version) {
            model.version = val;
        }

        // Interpolate properties
        if let Some(props) = &mut model.properties {
            let _ = interpolator.interpolate_map(props);
        }

        // Interpolate dependencies
        if let Some(deps) = &mut model.dependencies {
            for dep in &mut deps.dependencies {
                if let Ok(val) = interpolator.interpolate(&dep.group_id) {
                    dep.group_id = val;
                }
                if let Ok(val) = interpolator.interpolate(&dep.artifact_id) {
                    dep.artifact_id = val;
                }
                if let Some(version) = &dep.version {
                    if let Ok(val) = interpolator.interpolate(version) {
                        dep.version = Some(val);
                    }
                }
            }
        }
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
