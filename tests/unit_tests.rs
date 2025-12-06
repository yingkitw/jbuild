/// Comprehensive unit tests for jbuild
/// These tests use mock implementations to verify component behavior in isolation

use jbuild::core::{ExecutionRequestBuilder, ArtifactRepository, DependencyResolutionStrategy};
use jbuild::{MockArtifactRepository, MockDependencyResolver, TestProjectBuilder};
use std::path::PathBuf;

#[test]
fn test_execution_request_builder_creates_valid_request() {
    let base_dir = PathBuf::from("/test/project");
    let request = ExecutionRequestBuilder::new(base_dir.clone())
        .with_goals(vec!["clean".to_string(), "compile".to_string()])
        .with_pom_file(PathBuf::from("/test/project/pom.xml"))
        .build();

    assert_eq!(request.base_directory, base_dir);
    assert_eq!(request.goals.len(), 2);
    assert_eq!(request.goals[0], "clean");
    assert_eq!(request.goals[1], "compile");
}

#[test]
fn test_execution_request_builder_with_system_properties() {
    let base_dir = PathBuf::from("/test/project");
    let request = ExecutionRequestBuilder::new(base_dir)
        .with_property("java.version".to_string(), "11".to_string())
        .with_property("project.build.sourceEncoding".to_string(), "UTF-8".to_string())
        .build();

    assert_eq!(request.system_properties.len(), 2);
    assert_eq!(
        request.system_properties.get("java.version"),
        Some(&"11".to_string())
    );
}

#[test]
fn test_mock_artifact_repository_stores_and_retrieves() {
    let repo = MockArtifactRepository::new();

    // Store artifacts
    repo.add_artifact("com.example", "lib-a", "1.0.0", "/repo/lib-a-1.0.0.jar".to_string());
    repo.add_artifact("com.example", "lib-b", "2.0.0", "/repo/lib-b-2.0.0.jar".to_string());

    // Verify existence
    assert!(repo.exists("com.example", "lib-a", "1.0.0"));
    assert!(repo.exists("com.example", "lib-b", "2.0.0"));
    assert!(!repo.exists("com.example", "lib-c", "1.0.0"));

    // Retrieve paths
    let path_a = repo.get_path("com.example", "lib-a", "1.0.0").unwrap();
    assert_eq!(path_a, "/repo/lib-a-1.0.0.jar");
}

#[test]
fn test_mock_artifact_repository_missing_artifact_error() {
    let repo = MockArtifactRepository::new();

    let result = repo.get_path("com.example", "missing", "1.0.0");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Artifact not found"));
}

#[test]
fn test_mock_dependency_resolver_successful_resolution() {
    let resolver = MockDependencyResolver::new();

    resolver.register_resolution(
        "com.example",
        "lib",
        "1.0.0",
        "/repo/lib-1.0.0.jar".to_string(),
    );

    let result = resolver
        .resolve_dependency("com.example", "lib", "1.0.0")
        .unwrap();

    assert_eq!(result, Some("/repo/lib-1.0.0.jar".to_string()));
}

#[test]
fn test_mock_dependency_resolver_missing_dependency() {
    let resolver = MockDependencyResolver::new();

    resolver.register_missing("com.example", "missing", "1.0.0");

    let result = resolver
        .resolve_dependency("com.example", "missing", "1.0.0")
        .unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_mock_dependency_resolver_multiple_dependencies() {
    let resolver = MockDependencyResolver::new();

    resolver.register_resolution(
        "com.example",
        "lib-a",
        "1.0.0",
        "/repo/lib-a.jar".to_string(),
    );
    resolver.register_resolution(
        "com.example",
        "lib-b",
        "2.0.0",
        "/repo/lib-b.jar".to_string(),
    );

    let deps = vec![
        ("com.example".to_string(), "lib-a".to_string(), "1.0.0".to_string()),
        ("com.example".to_string(), "lib-b".to_string(), "2.0.0".to_string()),
    ];

    let resolved = resolver.resolve_dependencies(&deps).unwrap();
    assert_eq!(resolved.len(), 2);
}

#[test]
fn test_test_project_builder_defaults() {
    let project = TestProjectBuilder::new();

    assert_eq!(project.group_id(), "com.example");
    assert_eq!(project.artifact_id(), "test-project");
    assert_eq!(project.version(), "1.0.0");
    assert_eq!(project.packaging(), "jar");
}

#[test]
fn test_test_project_builder_customization() {
    let project = TestProjectBuilder::new()
        .with_group_id("org.mycompany".to_string())
        .with_artifact_id("my-app".to_string())
        .with_version("3.0.0".to_string())
        .with_packaging("war".to_string());

    assert_eq!(project.group_id(), "org.mycompany");
    assert_eq!(project.artifact_id(), "my-app");
    assert_eq!(project.version(), "3.0.0");
    assert_eq!(project.packaging(), "war");
}

#[test]
fn test_execution_request_builder_reactor_mode() {
    let base_dir = PathBuf::from("/test/project");
    let request = ExecutionRequestBuilder::new(base_dir)
        .reactor_active(false)
        .build();

    assert!(!request.reactor_active);
}

#[test]
fn test_execution_request_builder_with_profiles() {
    let base_dir = PathBuf::from("/test/project");
    let request = ExecutionRequestBuilder::new(base_dir)
        .with_profile("dev".to_string())
        .with_profile("test".to_string())
        .build();

    assert_eq!(request.active_profiles.len(), 2);
    assert!(request.active_profiles.contains(&"dev".to_string()));
    assert!(request.active_profiles.contains(&"test".to_string()));
}

#[test]
fn test_mock_artifact_repository_multiple_versions() {
    let repo = MockArtifactRepository::new();

    repo.add_artifact("com.example", "lib", "1.0.0", "/repo/lib-1.0.0.jar".to_string());
    repo.add_artifact("com.example", "lib", "1.1.0", "/repo/lib-1.1.0.jar".to_string());
    repo.add_artifact("com.example", "lib", "2.0.0", "/repo/lib-2.0.0.jar".to_string());

    assert!(repo.exists("com.example", "lib", "1.0.0"));
    assert!(repo.exists("com.example", "lib", "1.1.0"));
    assert!(repo.exists("com.example", "lib", "2.0.0"));

    let artifacts = repo.get_all_artifacts();
    assert_eq!(artifacts.len(), 3);
}
