//! Gradle Version Catalogs
//!
//! Implements Gradle's version catalog feature for centralized dependency management.
//! Version catalogs allow declaring dependencies in a central location (libs.versions.toml).

use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};

/// A version catalog containing libraries, plugins, and versions
#[derive(Debug, Clone, Default)]
pub struct VersionCatalog {
    /// Catalog name (e.g., "libs")
    pub name: String,
    /// Version declarations
    pub versions: HashMap<String, String>,
    /// Library declarations
    pub libraries: HashMap<String, LibraryDeclaration>,
    /// Plugin declarations
    pub plugins: HashMap<String, PluginDeclaration>,
    /// Bundle declarations (groups of libraries)
    pub bundles: HashMap<String, Vec<String>>,
}

/// A library declaration in the catalog
#[derive(Debug, Clone)]
pub struct LibraryDeclaration {
    /// Library alias (e.g., "junit-jupiter")
    pub alias: String,
    /// Group ID
    pub group: String,
    /// Artifact name
    pub name: String,
    /// Version (can be a reference like "version.ref")
    pub version: VersionSpec,
}

/// A plugin declaration in the catalog
#[derive(Debug, Clone)]
pub struct PluginDeclaration {
    /// Plugin alias
    pub alias: String,
    /// Plugin ID
    pub id: String,
    /// Version
    pub version: VersionSpec,
}

/// Version specification - either a literal or a reference
#[derive(Debug, Clone)]
pub enum VersionSpec {
    /// Literal version string
    Literal(String),
    /// Reference to a version in the versions section
    Reference(String),
}

impl VersionSpec {
    /// Resolve the version using the catalog's version map
    pub fn resolve(&self, versions: &HashMap<String, String>) -> Option<String> {
        match self {
            VersionSpec::Literal(v) => Some(v.clone()),
            VersionSpec::Reference(ref_name) => versions.get(ref_name).cloned(),
        }
    }
}

impl VersionCatalog {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Add a version
    pub fn add_version(&mut self, name: impl Into<String>, version: impl Into<String>) {
        self.versions.insert(name.into(), version.into());
    }

    /// Add a library
    pub fn add_library(&mut self, alias: impl Into<String>, group: impl Into<String>, 
                       name: impl Into<String>, version: VersionSpec) {
        let alias = alias.into();
        self.libraries.insert(alias.clone(), LibraryDeclaration {
            alias: alias.clone(),
            group: group.into(),
            name: name.into(),
            version,
        });
    }

    /// Add a plugin
    pub fn add_plugin(&mut self, alias: impl Into<String>, id: impl Into<String>, version: VersionSpec) {
        let alias = alias.into();
        self.plugins.insert(alias.clone(), PluginDeclaration {
            alias: alias.clone(),
            id: id.into(),
            version,
        });
    }

    /// Add a bundle
    pub fn add_bundle(&mut self, name: impl Into<String>, libraries: Vec<String>) {
        self.bundles.insert(name.into(), libraries);
    }

    /// Get a library by alias
    pub fn get_library(&self, alias: &str) -> Option<&LibraryDeclaration> {
        self.libraries.get(alias)
    }

    /// Get a library's GAV notation
    pub fn get_library_notation(&self, alias: &str) -> Option<String> {
        self.libraries.get(alias).and_then(|lib| {
            lib.version.resolve(&self.versions).map(|v| {
                format!("{}:{}:{}", lib.group, lib.name, v)
            })
        })
    }

    /// Get all libraries in a bundle
    pub fn get_bundle_libraries(&self, bundle_name: &str) -> Vec<&LibraryDeclaration> {
        self.bundles.get(bundle_name)
            .map(|aliases| {
                aliases.iter()
                    .filter_map(|alias| self.libraries.get(alias))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a plugin by alias
    pub fn get_plugin(&self, alias: &str) -> Option<&PluginDeclaration> {
        self.plugins.get(alias)
    }
}

/// Parse a version catalog from TOML content (libs.versions.toml format)
pub fn parse_version_catalog(content: &str, name: &str) -> Result<VersionCatalog> {
    let mut catalog = VersionCatalog::new(name);

    // Simple TOML-like parsing for version catalogs
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }

        // Key-value pair
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos+1..].trim().trim_matches('"');

            match current_section.as_str() {
                "versions" => {
                    catalog.add_version(key, value);
                }
                "libraries" => {
                    // Parse library declaration
                    if let Some(lib) = parse_library_declaration(key, value) {
                        catalog.libraries.insert(key.to_string(), lib);
                    }
                }
                "plugins" => {
                    // Parse plugin declaration
                    if let Some(plugin) = parse_plugin_declaration(key, value) {
                        catalog.plugins.insert(key.to_string(), plugin);
                    }
                }
                "bundles" => {
                    // Parse bundle (array of library aliases)
                    let libs: Vec<String> = value
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    catalog.add_bundle(key, libs);
                }
                _ => {}
            }
        }
    }

    Ok(catalog)
}

/// Parse a library declaration from TOML value
fn parse_library_declaration(alias: &str, value: &str) -> Option<LibraryDeclaration> {
    // Handle inline table format: { group = "...", name = "...", version.ref = "..." }
    if value.starts_with('{') {
        let inner = value.trim_matches(|c| c == '{' || c == '}');
        let mut group = String::new();
        let mut name = String::new();
        let mut version = VersionSpec::Literal(String::new());

        for part in inner.split(',') {
            let part = part.trim();
            if let Some(eq_pos) = part.find('=') {
                let k = part[..eq_pos].trim();
                let v = part[eq_pos+1..].trim().trim_matches('"');

                match k {
                    "group" => group = v.to_string(),
                    "name" => name = v.to_string(),
                    "version" => version = VersionSpec::Literal(v.to_string()),
                    "version.ref" => version = VersionSpec::Reference(v.to_string()),
                    _ => {}
                }
            }
        }

        if !group.is_empty() && !name.is_empty() {
            return Some(LibraryDeclaration {
                alias: alias.to_string(),
                group,
                name,
                version,
            });
        }
    }

    // Handle simple GAV format: "group:artifact:version"
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() >= 2 {
        return Some(LibraryDeclaration {
            alias: alias.to_string(),
            group: parts[0].to_string(),
            name: parts[1].to_string(),
            version: if parts.len() >= 3 {
                VersionSpec::Literal(parts[2].to_string())
            } else {
                VersionSpec::Literal(String::new())
            },
        });
    }

    None
}

/// Parse a plugin declaration from TOML value
fn parse_plugin_declaration(alias: &str, value: &str) -> Option<PluginDeclaration> {
    // Handle inline table format: { id = "...", version.ref = "..." }
    if value.starts_with('{') {
        let inner = value.trim_matches(|c| c == '{' || c == '}');
        let mut id = String::new();
        let mut version = VersionSpec::Literal(String::new());

        for part in inner.split(',') {
            let part = part.trim();
            if let Some(eq_pos) = part.find('=') {
                let k = part[..eq_pos].trim();
                let v = part[eq_pos+1..].trim().trim_matches('"');

                match k {
                    "id" => id = v.to_string(),
                    "version" => version = VersionSpec::Literal(v.to_string()),
                    "version.ref" => version = VersionSpec::Reference(v.to_string()),
                    _ => {}
                }
            }
        }

        if !id.is_empty() {
            return Some(PluginDeclaration {
                alias: alias.to_string(),
                id,
                version,
            });
        }
    }

    None
}

/// Find and parse the version catalog file
pub fn find_version_catalog(project_dir: &Path) -> Result<Option<VersionCatalog>> {
    let catalog_path = project_dir.join("gradle/libs.versions.toml");
    
    if catalog_path.exists() {
        let content = std::fs::read_to_string(&catalog_path)
            .with_context(|| format!("Failed to read version catalog: {:?}", catalog_path))?;
        let catalog = parse_version_catalog(&content, "libs")?;
        Ok(Some(catalog))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_catalog_creation() {
        let mut catalog = VersionCatalog::new("libs");
        catalog.add_version("junit", "5.9.3");
        catalog.add_library("junit-jupiter", "org.junit.jupiter", "junit-jupiter", 
                           VersionSpec::Reference("junit".to_string()));

        assert_eq!(catalog.versions.get("junit"), Some(&"5.9.3".to_string()));
        assert!(catalog.get_library("junit-jupiter").is_some());
    }

    #[test]
    fn test_version_resolution() {
        let mut catalog = VersionCatalog::new("libs");
        catalog.add_version("junit", "5.9.3");
        catalog.add_library("junit-jupiter", "org.junit.jupiter", "junit-jupiter",
                           VersionSpec::Reference("junit".to_string()));

        let notation = catalog.get_library_notation("junit-jupiter");
        assert_eq!(notation, Some("org.junit.jupiter:junit-jupiter:5.9.3".to_string()));
    }

    #[test]
    fn test_bundle() {
        let mut catalog = VersionCatalog::new("libs");
        catalog.add_library("guava", "com.google.guava", "guava", 
                           VersionSpec::Literal("31.0".to_string()));
        catalog.add_library("commons-lang", "org.apache.commons", "commons-lang3",
                           VersionSpec::Literal("3.12.0".to_string()));
        catalog.add_bundle("common", vec!["guava".to_string(), "commons-lang".to_string()]);

        let bundle_libs = catalog.get_bundle_libraries("common");
        assert_eq!(bundle_libs.len(), 2);
    }

    #[test]
    fn test_parse_version_catalog() {
        let content = r#"
[versions]
junit = "5.9.3"
guava = "31.0"

[libraries]
junit-jupiter = { group = "org.junit.jupiter", name = "junit-jupiter", version.ref = "junit" }
guava = "com.google.guava:guava:31.0"

[bundles]
testing = ["junit-jupiter"]

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version = "1.9.0" }
"#;

        let catalog = parse_version_catalog(content, "libs").unwrap();

        assert_eq!(catalog.versions.get("junit"), Some(&"5.9.3".to_string()));
        assert!(catalog.get_library("junit-jupiter").is_some());
        assert!(catalog.get_library("guava").is_some());
        assert!(catalog.bundles.contains_key("testing"));
        assert!(catalog.get_plugin("kotlin").is_some());
    }

    #[test]
    fn test_literal_version() {
        let version = VersionSpec::Literal("1.0.0".to_string());
        let versions = HashMap::new();
        assert_eq!(version.resolve(&versions), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_reference_version() {
        let version = VersionSpec::Reference("myVersion".to_string());
        let mut versions = HashMap::new();
        versions.insert("myVersion".to_string(), "2.0.0".to_string());
        assert_eq!(version.resolve(&versions), Some("2.0.0".to_string()));
    }
}
