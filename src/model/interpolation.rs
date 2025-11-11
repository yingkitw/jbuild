use std::collections::HashMap;
use regex::Regex;
use anyhow::{Context, Result};

/// Property interpolator for POM files
pub struct PropertyInterpolator {
    properties: HashMap<String, String>,
}

impl PropertyInterpolator {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    /// Add properties from a source
    pub fn add_properties(mut self, properties: HashMap<String, String>) -> Self {
        for (key, value) in properties {
            self.properties.insert(key, value);
        }
        self
    }

    /// Add a single property
    pub fn add_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }

    /// Interpolate a string with properties
    pub fn interpolate(&self, input: &str) -> Result<String> {
        // Pattern: ${property.name} or ${project.property}
        let pattern = Regex::new(r"\$\{([^}]+)\}")
            .context("Failed to create interpolation regex")?;

        let mut result = input.to_string();
        let mut max_iterations = 100; // Prevent infinite loops
        let mut changed = true;

        while changed && max_iterations > 0 {
            changed = false;
            max_iterations -= 1;

            result = pattern.replace_all(&result, |caps: &regex::Captures| {
                let prop_name = caps.get(1).unwrap().as_str();
                
                if let Some(value) = self.properties.get(prop_name) {
                    changed = true;
                    value.clone()
                } else {
                    // Keep original if property not found
                    caps.get(0).unwrap().as_str().to_string()
                }
            }).to_string();
        }

        if max_iterations == 0 {
            tracing::warn!("Property interpolation reached max iterations, possible circular reference");
        }

        Ok(result)
    }

    /// Interpolate all properties in a HashMap
    pub fn interpolate_map(&self, map: &mut HashMap<String, String>) -> Result<()> {
        let mut interpolated = HashMap::new();
        
        for (key, value) in map.iter() {
            let new_value = self.interpolate(value)?;
            interpolated.insert(key.clone(), new_value);
        }
        
        *map = interpolated;
        Ok(())
    }

    /// Interpolate a string with default properties
    pub fn interpolate_with_defaults(input: &str, additional_properties: &HashMap<String, String>) -> Result<String> {
        let mut interpolator = Self::new();
        
        // Add Maven built-in properties
        interpolator = interpolator.add_property("project.groupId".to_string(), "".to_string());
        interpolator = interpolator.add_property("project.artifactId".to_string(), "".to_string());
        interpolator = interpolator.add_property("project.version".to_string(), "".to_string());
        interpolator = interpolator.add_property("project.basedir".to_string(), ".".to_string());
        interpolator = interpolator.add_property("project.build.directory".to_string(), "target".to_string());
        interpolator = interpolator.add_property("project.build.outputDirectory".to_string(), "target/classes".to_string());
        interpolator = interpolator.add_property("project.build.testOutputDirectory".to_string(), "target/test-classes".to_string());
        
        // Add additional properties
        for (key, value) in additional_properties {
            interpolator = interpolator.add_property(key.clone(), value.clone());
        }
        
        interpolator.interpolate(input)
    }
}

impl Default for PropertyInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_simple() {
        let interpolator = PropertyInterpolator::new()
            .add_property("test.property".to_string(), "test-value".to_string());
        
        let result = interpolator.interpolate("Hello ${test.property}").unwrap();
        assert_eq!(result, "Hello test-value");
    }

    #[test]
    fn test_interpolate_multiple() {
        let interpolator = PropertyInterpolator::new()
            .add_property("prop1".to_string(), "value1".to_string())
            .add_property("prop2".to_string(), "value2".to_string());
        
        let result = interpolator.interpolate("${prop1} and ${prop2}").unwrap();
        assert_eq!(result, "value1 and value2");
    }

    #[test]
    fn test_interpolate_nested() {
        let interpolator = PropertyInterpolator::new()
            .add_property("base".to_string(), "target".to_string())
            .add_property("target".to_string(), "${base}/classes".to_string());
        
        let result = interpolator.interpolate("${target}").unwrap();
        assert_eq!(result, "target/classes");
    }

    #[test]
    fn test_interpolate_missing_property() {
        let interpolator = PropertyInterpolator::new();
        
        let result = interpolator.interpolate("Hello ${missing.property}").unwrap();
        assert_eq!(result, "Hello ${missing.property}");
    }

    #[test]
    fn test_interpolate_no_placeholders() {
        let interpolator = PropertyInterpolator::new();
        
        let result = interpolator.interpolate("Hello World").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_interpolate_map() {
        let interpolator = PropertyInterpolator::new()
            .add_property("prop1".to_string(), "value1".to_string());
        
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "Hello ${prop1}".to_string());
        map.insert("key2".to_string(), "World".to_string());
        
        interpolator.interpolate_map(&mut map).unwrap();
        
        assert_eq!(map.get("key1"), Some(&"Hello value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"World".to_string()));
    }

    #[test]
    fn test_interpolate_with_defaults() {
        let mut props = HashMap::new();
        props.insert("project.groupId".to_string(), "com.example".to_string());
        props.insert("project.artifactId".to_string(), "myapp".to_string());
        
        let result = PropertyInterpolator::interpolate_with_defaults(
            "${project.groupId}:${project.artifactId}:${project.version}",
            &props
        ).unwrap();
        
        assert_eq!(result, "com.example:myapp:");
    }
}
