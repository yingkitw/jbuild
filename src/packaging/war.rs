use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use crate::packaging::{JarBuilder, Manifest, ResourceFilter};

/// WAR file builder (WAR is essentially a JAR with web.xml)
pub struct WarBuilder {
    jar_builder: JarBuilder,
    web_xml: Option<PathBuf>,
    webapp_dir: Option<PathBuf>,
}

impl WarBuilder {
    pub fn new() -> Self {
        Self {
            jar_builder: JarBuilder::new(),
            web_xml: None,
            webapp_dir: None,
        }
    }

    /// Set the web.xml descriptor
    pub fn with_web_xml(mut self, web_xml: PathBuf) -> Self {
        self.web_xml = Some(web_xml);
        self
    }

    /// Set the webapp directory (WEB-INF contents)
    pub fn with_webapp_dir(mut self, webapp_dir: PathBuf) -> Self {
        self.webapp_dir = Some(webapp_dir);
        self
    }

    /// Set the classes directory
    pub fn with_classes_dir(mut self, classes_dir: PathBuf) -> Self {
        self.jar_builder = self.jar_builder.with_classes_dir(classes_dir);
        self
    }

    /// Set the manifest
    pub fn with_manifest(mut self, manifest: Manifest) -> Self {
        self.jar_builder = self.jar_builder.with_manifest(manifest);
        self
    }

    /// Add a resource
    pub fn add_resource(mut self, source: PathBuf, target: PathBuf) -> Self {
        self.jar_builder = self.jar_builder.add_resource(source, target);
        self
    }

    /// Build the WAR file
    pub fn build(mut self, output_path: &Path) -> Result<()> {
        tracing::info!("Creating WAR file: {:?}", output_path);

        // WAR structure:
        // WEB-INF/
        //   web.xml
        //   classes/
        //   lib/
        // META-INF/
        //   MANIFEST.MF
        // (other web resources)

        // Add web.xml if provided
        if let Some(ref web_xml) = self.web_xml {
            if web_xml.exists() {
                self.jar_builder = self.jar_builder.add_resource(
                    web_xml.clone(),
                    PathBuf::from("WEB-INF/web.xml"),
                );
            }
        }

        // Add webapp directory contents
        if let Some(ref webapp_dir) = self.webapp_dir {
            if webapp_dir.exists() {
                let filter = ResourceFilter::default();
                self.jar_builder = self.jar_builder.add_resources_from_dir(
                    webapp_dir.clone(),
                    None,
                    filter,
                )?;
            }
        }

        // Build as JAR (WAR is just a JAR with specific structure)
        self.jar_builder.build(output_path)
    }
}

impl Default for WarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

