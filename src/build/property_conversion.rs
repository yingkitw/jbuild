//! Property Conversion
//!
//! Converts properties between Maven and Gradle formats.

use std::collections::HashMap;

/// Property converter between Maven and Gradle
pub struct PropertyConverter;

impl PropertyConverter {
    /// Convert Maven properties to Gradle extra properties
    pub fn maven_to_gradle(maven_props: &HashMap<String, String>) -> HashMap<String, String> {
        let mut gradle_props = HashMap::new();

        for (key, value) in maven_props {
            let gradle_key = Self::convert_maven_key_to_gradle(key);
            gradle_props.insert(gradle_key, value.clone());
        }

        gradle_props
    }

    /// Convert Gradle properties to Maven properties
    pub fn gradle_to_maven(gradle_props: &HashMap<String, String>) -> HashMap<String, String> {
        let mut maven_props = HashMap::new();

        for (key, value) in gradle_props {
            let maven_key = Self::convert_gradle_key_to_maven(key);
            maven_props.insert(maven_key, value.clone());
        }

        maven_props
    }

    /// Convert a Maven property key to Gradle format
    fn convert_maven_key_to_gradle(key: &str) -> String {
        // Maven uses dots, Gradle uses camelCase for some properties
        match key {
            // Project properties
            "project.version" => "version".to_string(),
            "project.groupId" => "group".to_string(),
            "project.artifactId" => "name".to_string(),
            "project.name" => "projectName".to_string(),
            "project.description" => "description".to_string(),
            
            // Build properties
            "project.build.directory" => "buildDir".to_string(),
            "project.build.sourceDirectory" => "sourceSets.main.java.srcDirs".to_string(),
            "project.build.testSourceDirectory" => "sourceSets.test.java.srcDirs".to_string(),
            "project.build.outputDirectory" => "sourceSets.main.output.classesDirs".to_string(),
            
            // Compiler properties
            "maven.compiler.source" => "sourceCompatibility".to_string(),
            "maven.compiler.target" => "targetCompatibility".to_string(),
            "maven.compiler.release" => "javaLanguageVersion".to_string(),
            "project.build.sourceEncoding" => "compileJava.options.encoding".to_string(),
            
            // Test properties
            "maven.test.skip" => "test.enabled".to_string(),
            "skipTests" => "test.enabled".to_string(),
            
            // Default: keep as-is but convert dots to underscores for ext properties
            _ => {
                if key.contains('.') {
                    format!("ext.{}", key.replace('.', "_"))
                } else {
                    key.to_string()
                }
            }
        }
    }

    /// Convert a Gradle property key to Maven format
    fn convert_gradle_key_to_maven(key: &str) -> String {
        match key {
            "version" => "project.version".to_string(),
            "group" => "project.groupId".to_string(),
            "name" => "project.artifactId".to_string(),
            "description" => "project.description".to_string(),
            "buildDir" => "project.build.directory".to_string(),
            "sourceCompatibility" => "maven.compiler.source".to_string(),
            "targetCompatibility" => "maven.compiler.target".to_string(),
            
            // Default: keep as-is
            _ => {
                if key.starts_with("ext.") {
                    key[4..].replace('_', ".")
                } else {
                    key.to_string()
                }
            }
        }
    }

    /// Get standard Maven properties from a project
    pub fn standard_maven_properties() -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert("project.build.sourceEncoding".to_string(), "UTF-8".to_string());
        props.insert("maven.compiler.source".to_string(), "17".to_string());
        props.insert("maven.compiler.target".to_string(), "17".to_string());
        props
    }

    /// Get standard Gradle properties
    pub fn standard_gradle_properties() -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert("sourceCompatibility".to_string(), "17".to_string());
        props.insert("targetCompatibility".to_string(), "17".to_string());
        props
    }
}

/// Maven property interpolation pattern: ${property.name}
pub fn interpolate_maven_properties(text: &str, properties: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    
    for (key, value) in properties {
        let pattern = format!("${{{}}}", key);
        result = result.replace(&pattern, value);
    }
    
    result
}

/// Gradle property interpolation pattern: $propertyName or ${propertyName}
pub fn interpolate_gradle_properties(text: &str, properties: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    
    for (key, value) in properties {
        // ${property} format
        let pattern1 = format!("${{{}}}", key);
        result = result.replace(&pattern1, value);
        
        // $property format (only for simple identifiers)
        if key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let pattern2 = format!("${}", key);
            // Be careful not to replace partial matches
            result = result.replace(&pattern2, value);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maven_to_gradle_conversion() {
        let mut maven_props = HashMap::new();
        maven_props.insert("maven.compiler.source".to_string(), "17".to_string());
        maven_props.insert("project.version".to_string(), "1.0.0".to_string());

        let gradle_props = PropertyConverter::maven_to_gradle(&maven_props);

        assert_eq!(gradle_props.get("sourceCompatibility"), Some(&"17".to_string()));
        assert_eq!(gradle_props.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_gradle_to_maven_conversion() {
        let mut gradle_props = HashMap::new();
        gradle_props.insert("sourceCompatibility".to_string(), "17".to_string());
        gradle_props.insert("version".to_string(), "1.0.0".to_string());

        let maven_props = PropertyConverter::gradle_to_maven(&gradle_props);

        assert_eq!(maven_props.get("maven.compiler.source"), Some(&"17".to_string()));
        assert_eq!(maven_props.get("project.version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_maven_interpolation() {
        let mut props = HashMap::new();
        props.insert("version".to_string(), "1.0.0".to_string());
        props.insert("name".to_string(), "my-app".to_string());

        let text = "Building ${name} version ${version}";
        let result = interpolate_maven_properties(text, &props);

        assert_eq!(result, "Building my-app version 1.0.0");
    }

    #[test]
    fn test_gradle_interpolation() {
        let mut props = HashMap::new();
        props.insert("version".to_string(), "1.0.0".to_string());

        let text = "version = ${version}";
        let result = interpolate_gradle_properties(text, &props);

        assert_eq!(result, "version = 1.0.0");
    }
}
