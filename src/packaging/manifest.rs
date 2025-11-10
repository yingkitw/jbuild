use std::collections::HashMap;
use std::io::Write;
use anyhow::{Context, Result};

/// JAR manifest representation
#[derive(Debug, Clone)]
pub struct Manifest {
    main_attributes: HashMap<String, String>,
    sections: HashMap<String, HashMap<String, String>>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            main_attributes: HashMap::new(),
            sections: HashMap::new(),
        }
    }

    /// Set a main attribute
    pub fn set_main_attribute(&mut self, key: String, value: String) {
        self.main_attributes.insert(key, value);
    }

    /// Get a main attribute
    pub fn get_main_attribute(&self, key: &str) -> Option<&String> {
        self.main_attributes.get(key)
    }

    /// Set a section attribute
    pub fn set_section_attribute(&mut self, section: String, key: String, value: String) {
        self.sections
            .entry(section)
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    /// Create a default manifest for a JAR
    pub fn default_jar_manifest(_group_id: &str, _artifact_id: &str, _version: &str) -> Self {
        let mut manifest = Self::new();
        manifest.set_main_attribute("Manifest-Version".to_string(), "1.0".to_string());
        manifest.set_main_attribute(
            "Created-By".to_string(),
            format!("mvn-rs {}", env!("CARGO_PKG_VERSION")),
        );
        
        // Set main class if it's an executable JAR (would come from POM configuration)
        // manifest.set_main_attribute("Main-Class".to_string(), "com.example.Main".to_string());
        
        manifest
    }

    /// Write manifest to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write main attributes
        for (key, value) in &self.main_attributes {
            writeln!(writer, "{}: {}", key, value)
                .context("Failed to write manifest attribute")?;
        }

        // Write sections
        for (section_name, attributes) in &self.sections {
            writeln!(writer, "").context("Failed to write manifest newline")?;
            writeln!(writer, "Name: {}", section_name)
                .context("Failed to write manifest section name")?;
            
            for (key, value) in attributes {
                writeln!(writer, "{}: {}", key, value)
                    .context("Failed to write manifest section attribute")?;
            }
        }

        // Manifest must end with newline
        writeln!(writer, "").context("Failed to write manifest final newline")?;

        Ok(())
    }

    /// Write manifest to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

