use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};

use crate::model::{Model, Parent, ModelBuilder};
use crate::model::parser::parse_pom_file;

/// Effective model builder - builds effective model with parent resolution
pub struct EffectiveModelBuilder {
    model_cache: HashMap<String, Model>,
    builder: ModelBuilder,
}

impl EffectiveModelBuilder {
    pub fn new() -> Self {
        Self {
            model_cache: HashMap::new(),
            builder: ModelBuilder::new(),
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
            effective = self.builder.build_effective_model(effective, Some(parent_model));
        }

        // Apply default values
        if effective.packaging.is_empty() {
            effective.packaging = "jar".to_string();
        }

        // Interpolate properties if available
        if let Some(props) = effective.properties.clone() {
            self.builder.interpolate(&mut effective, &props);
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
            let parent_artifact = crate::artifact::Artifact::new(
                &parent.group_id,
                &parent.artifact_id,
                &parent.version,
            );
            
            // We need a resolver to find it in repositories
            // For now, let's assume we can use a default resolver if none is provided
            // In a better design, we should pass the resolver or repository to this builder
            let local_repo = crate::artifact::repository::DefaultLocalRepository::default();
            let resolver = crate::resolver::DependencyResolver::new(Box::new(local_repo));
            
            if let Ok(Some(model)) = resolver.resolve_pom(&parent_artifact) {
                self.model_cache.insert(parent_key, model.clone());
                return Ok(model);
            }

            return Err(anyhow::anyhow!(
                "Parent POM not found in {:?} or repositories: {}:{}:{}",
                parent_pom,
                parent.group_id,
                parent.artifact_id,
                parent.version
            ));
        }

        let parent_model = parse_pom_file(&parent_pom)
            .with_context(|| format!("Failed to parse parent POM: {parent_pom:?}"))?;

        // Cache and return
        self.model_cache.insert(parent_key, parent_model.clone());
        Ok(parent_model)
    }
}

impl Default for EffectiveModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

