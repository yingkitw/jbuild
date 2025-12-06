//! Dependency Notation Conversion
//!
//! Converts between Maven and Gradle dependency notations.

use crate::artifact::ArtifactCoordinates;

/// Extended dependency coordinates with scope and extension
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyCoordinates {
    /// Core artifact coordinates
    pub coords: ArtifactCoordinates,
    /// Dependency scope (compile, test, etc.)
    pub scope: Option<String>,
    /// File extension (jar, pom, etc.)
    pub extension: Option<String>,
}

impl DependencyCoordinates {
    pub fn new(group_id: impl Into<String>, artifact_id: impl Into<String>) -> Self {
        Self {
            coords: ArtifactCoordinates::new(group_id, artifact_id, ""),
            scope: None,
            extension: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.coords.version = version.into();
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn with_classifier(mut self, classifier: impl Into<String>) -> Self {
        self.coords.classifier = Some(classifier.into());
        self
    }

    // Accessors for compatibility
    pub fn group_id(&self) -> &str { &self.coords.group_id }
    pub fn artifact_id(&self) -> &str { &self.coords.artifact_id }
    pub fn version(&self) -> Option<&str> { 
        if self.coords.version.is_empty() { None } else { Some(&self.coords.version) }
    }
    pub fn classifier(&self) -> Option<&str> { self.coords.classifier.as_deref() }

    /// Parse from Gradle notation: "group:artifact:version" or "group:artifact:version:classifier@ext"
    pub fn from_gradle_notation(notation: &str) -> Option<Self> {
        let (notation, extension) = notation.split_once('@')
            .map(|(n, e)| (n, Some(e.to_string())))
            .unwrap_or((notation, None));

        let parts: Vec<&str> = notation.split(':').collect();
        
        let (group_id, artifact_id, version, classifier) = match parts.len() {
            2 => (parts[0], parts[1], "", None),
            3 => (parts[0], parts[1], parts[2], None),
            4 => (parts[0], parts[1], parts[2], Some(parts[3].to_string())),
            _ => return None,
        };

        Some(Self {
            coords: ArtifactCoordinates {
                group_id: group_id.to_string(),
                artifact_id: artifact_id.to_string(),
                version: version.to_string(),
                packaging: None,
                classifier,
            },
            scope: None,
            extension,
        })
    }

    /// Convert to Gradle notation
    pub fn to_gradle_notation(&self) -> String {
        let mut notation = self.coords.id();
        
        if !self.coords.version.is_empty() {
            notation.push(':');
            notation.push_str(&self.coords.version);
        }
        
        if let Some(ref classifier) = self.coords.classifier {
            notation.push(':');
            notation.push_str(classifier);
        }
        
        if let Some(ref ext) = self.extension {
            notation.push('@');
            notation.push_str(ext);
        }
        
        notation
    }

    /// Convert to Maven XML dependency element
    pub fn to_maven_xml(&self) -> String {
        let mut xml = String::from("<dependency>\n");
        xml.push_str(&format!("    <groupId>{}</groupId>\n", self.coords.group_id));
        xml.push_str(&format!("    <artifactId>{}</artifactId>\n", self.coords.artifact_id));
        
        if !self.coords.version.is_empty() {
            xml.push_str(&format!("    <version>{}</version>\n", self.coords.version));
        }
        if let Some(ref scope) = self.scope {
            xml.push_str(&format!("    <scope>{}</scope>\n", scope));
        }
        if let Some(ref classifier) = self.coords.classifier {
            xml.push_str(&format!("    <classifier>{}</classifier>\n", classifier));
        }
        if let Some(ref ext) = self.extension {
            xml.push_str(&format!("    <type>{}</type>\n", ext));
        }
        
        xml.push_str("</dependency>");
        xml
    }

    /// Get the GAV string
    pub fn gav(&self) -> String {
        if self.coords.version.is_empty() {
            self.coords.id()
        } else {
            self.coords.full_id()
        }
    }
}

/// Scope mapping between Maven and Gradle
pub struct ScopeMapper;

impl ScopeMapper {
    /// Map Maven scope to Gradle configuration
    pub fn maven_to_gradle(maven_scope: &str) -> &'static str {
        match maven_scope {
            "compile" => "implementation",
            "provided" => "compileOnly",
            "runtime" => "runtimeOnly",
            "test" => "testImplementation",
            "system" => "compileOnly",
            "import" => "platform",
            _ => "implementation",
        }
    }

    /// Map Gradle configuration to Maven scope
    pub fn gradle_to_maven(gradle_config: &str) -> &'static str {
        match gradle_config {
            "implementation" => "compile",
            "api" => "compile",
            "compileOnly" => "provided",
            "runtimeOnly" => "runtime",
            "testImplementation" => "test",
            "testCompileOnly" => "test",
            "testRuntimeOnly" => "test",
            "annotationProcessor" => "provided",
            "platform" => "import",
            _ => "compile",
        }
    }

    /// Get all Maven scopes
    pub fn maven_scopes() -> &'static [&'static str] {
        &["compile", "provided", "runtime", "test", "system", "import"]
    }

    /// Get all Gradle configurations
    pub fn gradle_configurations() -> &'static [&'static str] {
        &[
            "implementation", "api", "compileOnly", "runtimeOnly",
            "testImplementation", "testCompileOnly", "testRuntimeOnly",
            "annotationProcessor", "platform"
        ]
    }
}

/// Convert a Maven POM dependency to Gradle notation
pub fn maven_dep_to_gradle(
    group_id: &str,
    artifact_id: &str,
    version: Option<&str>,
    scope: Option<&str>,
) -> (String, String) {
    let config = scope.map(ScopeMapper::maven_to_gradle).unwrap_or("implementation");
    let notation = match version {
        Some(v) => format!("{}:{}:{}", group_id, artifact_id, v),
        None => format!("{}:{}", group_id, artifact_id),
    };
    (config.to_string(), notation)
}

/// Convert a Gradle dependency to Maven format
pub fn gradle_dep_to_maven(
    notation: &str,
    configuration: &str,
) -> Option<DependencyCoordinates> {
    let mut coords = DependencyCoordinates::from_gradle_notation(notation)?;
    coords.scope = Some(ScopeMapper::gradle_to_maven(configuration).to_string());
    Some(coords)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gradle_notation() {
        let coords = DependencyCoordinates::from_gradle_notation("com.example:lib:1.0.0").unwrap();
        assert_eq!(coords.group_id(), "com.example");
        assert_eq!(coords.artifact_id(), "lib");
        assert_eq!(coords.version(), Some("1.0.0"));
    }

    #[test]
    fn test_parse_gradle_notation_with_classifier() {
        let coords = DependencyCoordinates::from_gradle_notation("com.example:lib:1.0.0:sources").unwrap();
        assert_eq!(coords.classifier(), Some("sources"));
    }

    #[test]
    fn test_parse_gradle_notation_with_extension() {
        let coords = DependencyCoordinates::from_gradle_notation("com.example:lib:1.0.0@pom").unwrap();
        assert_eq!(coords.extension, Some("pom".to_string()));
    }

    #[test]
    fn test_to_gradle_notation() {
        let coords = DependencyCoordinates::new("com.example", "lib").with_version("1.0.0");
        assert_eq!(coords.to_gradle_notation(), "com.example:lib:1.0.0");
    }

    #[test]
    fn test_to_maven_xml() {
        let coords = DependencyCoordinates::new("com.example", "lib")
            .with_version("1.0.0")
            .with_scope("test");
        let xml = coords.to_maven_xml();
        assert!(xml.contains("<groupId>com.example</groupId>"));
        assert!(xml.contains("<scope>test</scope>"));
    }

    #[test]
    fn test_scope_mapping() {
        assert_eq!(ScopeMapper::maven_to_gradle("compile"), "implementation");
        assert_eq!(ScopeMapper::maven_to_gradle("test"), "testImplementation");
        assert_eq!(ScopeMapper::gradle_to_maven("implementation"), "compile");
        assert_eq!(ScopeMapper::gradle_to_maven("testImplementation"), "test");
    }

    #[test]
    fn test_maven_dep_to_gradle() {
        let (config, notation) = maven_dep_to_gradle("junit", "junit", Some("4.13.2"), Some("test"));
        assert_eq!(config, "testImplementation");
        assert_eq!(notation, "junit:junit:4.13.2");
    }

    #[test]
    fn test_gradle_dep_to_maven() {
        let coords = gradle_dep_to_maven("junit:junit:4.13.2", "testImplementation").unwrap();
        assert_eq!(coords.group_id(), "junit");
        assert_eq!(coords.scope, Some("test".to_string()));
    }
}
