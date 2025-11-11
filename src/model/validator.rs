use anyhow::Result;
use crate::model::Model;

/// Model validation errors
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Model validator
pub struct ModelValidator;

impl ModelValidator {
    /// Validate a model
    pub fn validate(model: &Model) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate required fields
        if model.group_id.is_empty() {
            errors.push(ValidationError {
                field: "groupId".to_string(),
                message: "groupId is required".to_string(),
            });
        }

        if model.artifact_id.is_empty() {
            errors.push(ValidationError {
                field: "artifactId".to_string(),
                message: "artifactId is required".to_string(),
            });
        }

        if model.version.is_empty() {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "version is required".to_string(),
            });
        }

        // Validate model version
        if model.model_version != "4.0.0" {
            errors.push(ValidationError {
                field: "modelVersion".to_string(),
                message: format!("Unsupported model version: {}", model.model_version),
            });
        }

        // Validate packaging
        let valid_packagings = ["pom", "jar", "war", "ear", "ejb", "maven-plugin"];
        if !valid_packagings.contains(&model.packaging.as_str()) {
            // Warning, not error
            tracing::warn!("Unknown packaging type: {}", model.packaging);
        }

        // Validate coordinates format
        if !Self::is_valid_group_id(&model.group_id) {
            errors.push(ValidationError {
                field: "groupId".to_string(),
                message: format!("Invalid groupId format: {}", model.group_id),
            });
        }

        if !Self::is_valid_artifact_id(&model.artifact_id) {
            errors.push(ValidationError {
                field: "artifactId".to_string(),
                message: format!("Invalid artifactId format: {}", model.artifact_id),
            });
        }

        Ok(errors)
    }

    /// Check if groupId is valid
    fn is_valid_group_id(group_id: &str) -> bool {
        // GroupId should be dot-separated identifiers
        !group_id.is_empty() && group_id.split('.').all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        })
    }

    /// Check if artifactId is valid
    fn is_valid_artifact_id(artifact_id: &str) -> bool {
        // ArtifactId should be a valid identifier
        !artifact_id.is_empty() && artifact_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    }

    /// Validate and return errors as Result
    pub fn validate_or_error(model: &Model) -> Result<()> {
        let errors = Self::validate(model)?;
        
        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            
            return Err(anyhow::anyhow!("Model validation failed:\n{}", error_messages.join("\n")));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;

    fn create_valid_model() -> Model {
        Model {
            model_version: "4.0.0".to_string(),
            group_id: "com.example".to_string(),
            artifact_id: "myapp".to_string(),
            version: "1.0.0".to_string(),
            packaging: "jar".to_string(),
            name: None,
            description: None,
            url: None,
            inception_year: None,
            organization: None,
            licenses: None,
            developers: None,
            contributors: None,
            modules: None,
            parent: None,
            dependencies: None,
            dependency_management: None,
            build: None,
            profiles: None,
            properties: None,
            repositories: None,
            plugin_repositories: None,
            distribution_management: None,
            reporting: None,
        }
    }

    #[test]
    fn test_validate_valid_model() {
        let model = create_valid_model();
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_missing_group_id() {
        let mut model = create_valid_model();
        model.group_id = String::new();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "groupId"));
    }

    #[test]
    fn test_validate_missing_artifact_id() {
        let mut model = create_valid_model();
        model.artifact_id = String::new();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "artifactId"));
    }

    #[test]
    fn test_validate_missing_version() {
        let mut model = create_valid_model();
        model.version = String::new();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "version"));
    }

    #[test]
    fn test_validate_invalid_model_version() {
        let mut model = create_valid_model();
        model.model_version = "3.0.0".to_string();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "modelVersion"));
    }

    #[test]
    fn test_validate_invalid_group_id_format() {
        let mut model = create_valid_model();
        model.group_id = "invalid..group".to_string();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "groupId"));
    }

    #[test]
    fn test_validate_invalid_artifact_id_format() {
        let mut model = create_valid_model();
        model.artifact_id = "invalid@artifact".to_string();
        
        let errors = ModelValidator::validate(&model).unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "artifactId"));
    }

    #[test]
    fn test_validate_valid_group_id_format() {
        assert!(ModelValidator::is_valid_group_id("com.example"));
        assert!(ModelValidator::is_valid_group_id("org.apache.maven"));
        assert!(ModelValidator::is_valid_group_id("com.example-test"));
        assert!(ModelValidator::is_valid_group_id("com.example_test"));
    }

    #[test]
    fn test_validate_invalid_group_id_patterns() {
        assert!(!ModelValidator::is_valid_group_id(""));
        assert!(!ModelValidator::is_valid_group_id("com..example"));
        assert!(!ModelValidator::is_valid_group_id("com.example@test"));
    }

    #[test]
    fn test_validate_valid_artifact_id_format() {
        assert!(ModelValidator::is_valid_artifact_id("myapp"));
        assert!(ModelValidator::is_valid_artifact_id("my-app"));
        assert!(ModelValidator::is_valid_artifact_id("my_app"));
        assert!(ModelValidator::is_valid_artifact_id("my.app"));
    }

    #[test]
    fn test_validate_invalid_artifact_id_patterns() {
        assert!(!ModelValidator::is_valid_artifact_id(""));
        assert!(!ModelValidator::is_valid_artifact_id("my@app"));
    }

    #[test]
    fn test_validate_or_error_valid() {
        let model = create_valid_model();
        assert!(ModelValidator::validate_or_error(&model).is_ok());
    }

    #[test]
    fn test_validate_or_error_invalid() {
        let mut model = create_valid_model();
        model.group_id = String::new();
        
        assert!(ModelValidator::validate_or_error(&model).is_err());
    }
}

