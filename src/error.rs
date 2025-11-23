use thiserror::Error;
use std::path::PathBuf;

/// Maven execution errors
#[derive(Error, Debug)]
pub enum MavenError {
    #[error("POM parsing failed: {0}")]
    PomParseError(String),

    #[error("Model validation failed: {0}")]
    ModelValidationError(String),

    #[error("Dependency resolution failed for {group_id}:{artifact_id}: {reason}")]
    DependencyResolutionError {
        group_id: String,
        artifact_id: String,
        reason: String,
    },

    #[error("Artifact download failed: {0}")]
    ArtifactDownloadError(String),

    #[error("Plugin execution failed: {0}")]
    PluginExecutionError(String),

    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Project build failed: {0}")]
    ProjectBuildError(String),

    #[error("Lifecycle execution failed: {0}")]
    LifecycleExecutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    XmlError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid artifact coordinates: {0}")]
    InvalidCoordinates(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Settings error: {0}")]
    SettingsError(String),
}

/// Result type for Maven operations
pub type MavenResult<T> = Result<T, MavenError>;

impl From<quick_xml::de::DeError> for MavenError {
    fn from(err: quick_xml::de::DeError) -> Self {
        MavenError::XmlError(err.to_string())
    }
}

impl From<serde_json::Error> for MavenError {
    fn from(err: serde_json::Error) -> Self {
        MavenError::InvalidConfiguration(err.to_string())
    }
}

impl From<url::ParseError> for MavenError {
    fn from(err: url::ParseError) -> Self {
        MavenError::InvalidConfiguration(format!("Invalid URL: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_resolution_error_display() {
        let err = MavenError::DependencyResolutionError {
            group_id: "com.example".to_string(),
            artifact_id: "lib".to_string(),
            reason: "version not found".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Dependency resolution failed for com.example:lib: version not found"
        );
    }

    #[test]
    fn test_file_not_found_error() {
        let path = PathBuf::from("/path/to/file");
        let err = MavenError::FileNotFound(path.clone());
        assert!(err.to_string().contains("/path/to/file"));
    }
}
