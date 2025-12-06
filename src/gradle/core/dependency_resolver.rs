//! Gradle Dependency Resolution
//!
//! Resolves Gradle dependencies using the shared Maven resolver.

use std::path::PathBuf;
use anyhow::Result;

/// Resolve a dependency to a JAR file path
pub fn resolve_dependency(
    group: &str,
    artifact: &str,
    version: Option<&str>,
) -> Result<Option<PathBuf>> {
    use crate::resolver::resolver::DependencyResolver;
    use crate::artifact::artifact::Artifact;
    use crate::artifact::repository::{LocalRepository, DefaultLocalRepository};
    
    let version = version.ok_or_else(|| {
        anyhow::anyhow!("Dependency {group}:{artifact} has no version specified")
    })?;
    
    let artifact = Artifact::new(group, artifact, version);
    
    // Use default local repository
    let local_repo = DefaultLocalRepository::default();
    
    // Check local repository first
    if local_repo.artifact_exists(&artifact) {
        return Ok(Some(local_repo.artifact_path(&artifact)));
    }
    
    // Try to resolve using dependency resolver
    let local_repo_box: Box<dyn LocalRepository> = Box::new(DefaultLocalRepository::default());
    let resolver = DependencyResolver::new(local_repo_box);
    
    // Convert to Maven Dependency format
    use crate::model::Dependency as MavenDependency;
    let maven_dep = MavenDependency {
        group_id: group.to_string(),
        artifact_id: artifact.to_string(),
        version: Some(version.to_string()),
        type_: Some("jar".to_string()),
        classifier: None,
        scope: Some("compile".to_string()),
        optional: Some(false),
        exclusions: None,
    };
    
    if let Ok(Some(resolved_artifact)) = resolver.resolve_dependency(&maven_dep) {
        if let Some(file_path) = resolved_artifact.file {
            return Ok(Some(file_path));
        }
    }
    
    tracing::warn!("Dependency {group}:{artifact}:{version} could not be resolved");
    
    Ok(None)
}
