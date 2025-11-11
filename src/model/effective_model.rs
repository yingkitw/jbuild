use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};

use crate::model::{Model, Parent};
use crate::model::parser::parse_pom_file;

/// Effective model builder - builds effective model with parent resolution
pub struct EffectiveModelBuilder {
    model_cache: HashMap<String, Model>,
}

impl EffectiveModelBuilder {
    pub fn new() -> Self {
        Self {
            model_cache: HashMap::new(),
        }
    }

    /// Build effective model from a base model, resolving parent inheritance
    pub fn build_effective_model(
        &mut self,
        model: Model,
        base_dir: &PathBuf,
    ) -> Result<Model> {
        let mut effective = model.clone();

        // Resolve parent if present
        if let Some(parent_ref) = &effective.parent {
            let parent_model = self.resolve_parent(parent_ref, base_dir)?;
            effective = self.merge_parent(effective, parent_model)?;
        }

        // Apply default values
        if effective.packaging.is_empty() {
            effective.packaging = "jar".to_string();
        }

        Ok(effective)
    }

    /// Resolve parent POM
    fn resolve_parent(
        &mut self,
        parent: &Parent,
        base_dir: &PathBuf,
    ) -> Result<Model> {
        let parent_key = format!("{}:{}:{}", parent.group_id, parent.artifact_id, parent.version);

        // Check cache
        if let Some(cached) = self.model_cache.get(&parent_key) {
            return Ok(cached.clone());
        }

        // Resolve parent POM path
        let parent_pom = if let Some(relative_path) = &parent.relative_path {
            base_dir.join(relative_path)
        } else {
            // Default parent path
            let mut parent_dir = base_dir.clone();
            parent_dir.pop(); // Go up one level
            parent_dir.join("pom.xml")
        };

        if !parent_pom.exists() {
            // Try resolving from local repository
            // TODO: Implement repository-based parent resolution
            return Err(anyhow::anyhow!(
                "Parent POM not found: {:?}",
                parent_pom
            ));
        }

        let parent_model = parse_pom_file(&parent_pom)
            .with_context(|| format!("Failed to parse parent POM: {:?}", parent_pom))?;

        // Cache and return
        self.model_cache.insert(parent_key, parent_model.clone());
        Ok(parent_model)
    }

    /// Merge parent model into child model
    fn merge_parent(&self, mut child: Model, parent: Model) -> Result<Model> {
        // Inherit groupId if not set
        if child.group_id.is_empty() {
            child.group_id = parent.group_id.clone();
        }

        // Inherit version if not set
        if child.version.is_empty() {
            child.version = parent.version.clone();
        }

        // Merge properties
        let mut properties = child.properties.unwrap_or_default();
        if let Some(parent_props) = &parent.properties {
            for (key, value) in parent_props {
                properties.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        child.properties = Some(properties);

        // Merge dependency management
        if let Some(parent_dep_mgmt) = &parent.dependency_management {
            if child.dependency_management.is_none() {
                child.dependency_management = Some(parent_dep_mgmt.clone());
            } else {
                // Merge dependency management entries
                let child_dep_mgmt = child.dependency_management.as_mut().unwrap();
                if let Some(parent_deps) = &parent_dep_mgmt.dependencies {
                    if child_dep_mgmt.dependencies.is_none() {
                        child_dep_mgmt.dependencies = Some(parent_deps.clone());
                    } else {
                        let child_deps = child_dep_mgmt.dependencies.as_mut().unwrap();
                        // Add parent dependencies that aren't in child
                        for parent_dep in &parent_deps.dependencies {
                            let parent_key = format!("{}:{}", parent_dep.group_id, parent_dep.artifact_id);
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
        if let Some(parent_build) = &parent.build {
            if child.build.is_none() {
                child.build = Some(parent_build.clone());
            } else {
                let child_build = child.build.as_mut().unwrap();
                // Inherit source directories if not set
                if child_build.source_directory.is_none() {
                    child_build.source_directory = parent_build.source_directory.clone();
                }
                if child_build.test_source_directory.is_none() {
                    child_build.test_source_directory = parent_build.test_source_directory.clone();
                }
                if child_build.output_directory.is_none() {
                    child_build.output_directory = parent_build.output_directory.clone();
                }
                if child_build.test_output_directory.is_none() {
                    child_build.test_output_directory = parent_build.test_output_directory.clone();
                }
            }
        }

        Ok(child)
    }
}

impl Default for EffectiveModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

