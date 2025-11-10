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

