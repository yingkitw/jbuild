//! Properties file loader for Checkstyle-rs

use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load properties from a Java properties file
pub struct PropertiesLoader;

impl PropertiesLoader {
    /// Load properties from a file path
    pub fn load_properties<P: AsRef<Path>>(path: P) -> CheckstyleResult<HashMap<String, String>> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| {
            CheckstyleError::Configuration(format!(
                "Could not open properties file {}: {}",
                path.display(),
                e
            ))
        })?;

        let reader = BufReader::new(file);
        Self::parse_properties(reader)
    }

    /// Parse properties from a reader
    fn parse_properties<R: BufRead>(reader: R) -> CheckstyleResult<HashMap<String, String>> {
        let mut properties = HashMap::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| {
                CheckstyleError::Configuration(format!(
                    "Error reading properties file at line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }

            // Find the key-value separator
            if let Some(sep_pos) = line.find('=') {
                let key = line[..sep_pos].trim().to_string();
                let value = line[sep_pos + 1..].trim().to_string();

                if !key.is_empty() {
                    properties.insert(key, value);
                }
            }
        }

        Ok(properties)
    }
}
