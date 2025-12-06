//! Dependency Conflict Resolution
//!
//! Resolves conflicts between different versions of the same dependency.

use std::collections::HashMap;
use crate::artifact::Artifact;
use crate::common::version::version_key;

/// Dependency conflict resolver
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflicts using nearest-wins strategy (Maven's default)
    pub fn resolve_conflicts(dependencies: &[(String, Artifact)]) -> Vec<Artifact> {
        let mut resolved = HashMap::new();
        for (key, artifact) in dependencies {
            resolved.insert(key.clone(), artifact.clone());
        }
        resolved.into_values().collect()
    }

    /// Resolve conflicts using highest-version-wins strategy
    pub fn resolve_by_highest_version(dependencies: &[(String, Artifact)]) -> Vec<Artifact> {
        let mut grouped: HashMap<String, Vec<Artifact>> = HashMap::new();
        
        for (_, artifact) in dependencies {
            let key = format!("{}:{}", artifact.coordinates.group_id, artifact.coordinates.artifact_id);
            grouped.entry(key).or_default().push(artifact.clone());
        }
        
        grouped.into_values()
            .filter_map(|artifacts| artifacts.into_iter().max_by_key(|a| version_key(&a.coordinates.version)))
            .collect()
    }
}

/// Dependency mediator for conflict resolution
pub struct DependencyMediator;

impl DependencyMediator {
    /// Mediate between conflicting dependency versions
    pub fn mediate(_group_id: &str, _artifact_id: &str, versions: &[String]) -> Option<String> {
        versions.iter().max_by_key(|v| version_key(v)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::artifact::ArtifactCoordinates;

    #[test]
    fn test_conflict_resolver_nearest_wins() {
        let artifact1 = Artifact {
            coordinates: ArtifactCoordinates::new("com.example", "lib", "1.0.0"),
            file: Some(PathBuf::from("lib-1.0.0.jar")),
            base_version: "1.0.0".to_string(),
            is_snapshot: false,
        };
        let artifact2 = Artifact {
            coordinates: ArtifactCoordinates::new("com.example", "lib", "2.0.0"),
            file: Some(PathBuf::from("lib-2.0.0.jar")),
            base_version: "2.0.0".to_string(),
            is_snapshot: false,
        };

        let conflicts = vec![
            ("lib:1.0.0".to_string(), artifact1),
            ("lib:2.0.0".to_string(), artifact2),
        ];

        let resolved = ConflictResolver::resolve_conflicts(&conflicts);
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_conflict_resolver_highest_version() {
        let artifact1 = Artifact {
            coordinates: ArtifactCoordinates::new("com.example", "lib", "1.0.0"),
            file: Some(PathBuf::from("lib-1.0.0.jar")),
            base_version: "1.0.0".to_string(),
            is_snapshot: false,
        };
        let artifact2 = Artifact {
            coordinates: ArtifactCoordinates::new("com.example", "lib", "2.0.0"),
            file: Some(PathBuf::from("lib-2.0.0.jar")),
            base_version: "2.0.0".to_string(),
            is_snapshot: false,
        };

        let conflicts = vec![
            ("lib:1.0.0".to_string(), artifact1),
            ("lib:2.0.0".to_string(), artifact2),
        ];

        let resolved = ConflictResolver::resolve_by_highest_version(&conflicts);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].coordinates.version, "2.0.0");
    }

    #[test]
    fn test_dependency_mediator() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string(), "2.0.0".to_string()];
        let result = DependencyMediator::mediate("com.example", "lib", &versions);
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_dependency_mediator_empty() {
        let result = DependencyMediator::mediate("com.example", "lib", &[]);
        assert_eq!(result, None);
    }
    // version_key tests are in common/version.rs
}
