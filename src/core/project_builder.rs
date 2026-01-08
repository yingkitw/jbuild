use std::path::PathBuf;
use anyhow::{Context, Result};

use crate::model::{Model, parse_pom_file, EffectiveModelBuilder};
use crate::core::project::MavenProject;

/// Project builder - builds MavenProject from POM files
pub struct ProjectBuilder;

impl ProjectBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build a project from a POM file
    pub fn build(&self, pom_file: &PathBuf) -> Result<MavenProject> {
        let base_model = parse_pom_file(pom_file)
            .with_context(|| format!("Failed to parse POM file: {pom_file:?}"))?;
        
        let basedir = pom_file.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        // Build effective model with parent resolution
        let mut effective_builder = EffectiveModelBuilder::new();
        let effective_model = effective_builder.build_effective_model(base_model, &basedir)
            .with_context(|| "Failed to build effective model")?;

        Ok(MavenProject::new(effective_model, basedir))
    }

    /// Build a project from a model and base directory
    pub fn build_from_model(&self, model: Model, basedir: PathBuf) -> MavenProject {
        MavenProject::new(model, basedir)
    }

    /// Build projects for a multi-module reactor
    pub fn build_reactor(&self, root_pom: &PathBuf) -> Result<Vec<MavenProject>> {
        let root_project = self.build(root_pom)?;
        let mut projects = vec![root_project.clone()];

        // Build child modules
        if let Some(modules) = &root_project.model.modules {
            let root_dir = root_project.basedir.clone();
            for module in modules {
                let module_pom = root_dir.join(module).join("pom.xml");
                if module_pom.exists() {
                    if let Ok(module_project) = self.build(&module_pom) {
                        projects.push(module_project);
                    }
                }
            }
        }

        Ok(projects)
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

