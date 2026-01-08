//! Configuration loader for Checkstyle-rs

use crate::checkstyle::api::config::Configuration;
use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Load configuration from XML file
pub struct ConfigurationLoader;

impl ConfigurationLoader {
    /// Load configuration from a file path or resource name
    pub fn load_configuration<P: AsRef<Path>>(path: P) -> CheckstyleResult<Configuration> {
        let path = path.as_ref();

        // Try to open as file first
        let file = File::open(path).map_err(|e| {
            CheckstyleError::Configuration(format!(
                "Could not open configuration file {}: {}",
                path.display(),
                e
            ))
        })?;

        let reader = BufReader::new(file);
        Self::parse_xml(reader)
    }

    /// Parse XML configuration
    fn parse_xml<R: std::io::BufRead>(reader: R) -> CheckstyleResult<Configuration> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.trim_text(true);

        let mut config_stack: Vec<Configuration> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = e.name();
                    let name_str = String::from_utf8_lossy(name.as_ref());

                    let mut config = Configuration::new(name_str.to_string());

                    // Read attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key_bytes = attr.key.as_ref();
                            let key = String::from_utf8_lossy(key_bytes);
                            let value = attr
                                .decode_and_unescape_value(&xml_reader)
                                .unwrap_or_default();
                            config.add_property(key.to_string(), value.to_string());
                        }
                    }

                    config_stack.push(config);
                }
                Ok(Event::End(_)) => {
                    if config_stack.len() > 1 {
                        let child = config_stack.pop().unwrap();
                        if let Some(parent) = config_stack.last_mut() {
                            parent.add_child(child);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CheckstyleError::Configuration(format!(
                        "XML parsing error: {e}"
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        config_stack
            .pop()
            .ok_or_else(|| CheckstyleError::Configuration("Empty configuration file".to_string()))
    }

    /// Create a default configuration with common checks
    pub fn create_default_configuration() -> Configuration {
        let mut checker = Configuration::new("Checker".to_string());
        
        // Add TreeWalker with common checks
        let mut tree_walker = Configuration::new("TreeWalker".to_string());
        
        // Add common checks (names match module factory)
        tree_walker.add_child(Configuration::new("EmptyCatchBlock".to_string()));
        tree_walker.add_child(Configuration::new("EmptyStatement".to_string()));
        tree_walker.add_child(Configuration::new("MissingSwitchDefault".to_string()));
        tree_walker.add_child(Configuration::new("MultipleVariableDeclarations".to_string()));
        tree_walker.add_child(Configuration::new("SimplifyBooleanReturn".to_string()));
        tree_walker.add_child(Configuration::new("PackageName".to_string()));
        tree_walker.add_child(Configuration::new("TypeName".to_string()));
        tree_walker.add_child(Configuration::new("RedundantImport".to_string()));
        
        // Add LineLength check with default max of 120
        let mut line_length = Configuration::new("LineLength".to_string());
        line_length.add_property("max".to_string(), "120".to_string());
        checker.add_child(line_length);
        
        checker.add_child(tree_walker);
        checker
    }
}
