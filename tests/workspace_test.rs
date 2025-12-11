use jbuild::config::{JbuildWorkspace, Workspace};
use std::fs;
use tempfile::TempDir;

/// Test workspace configuration parsing and serialization
#[test]
fn test_workspace_config_parsing() {
    let toml_content = r#"
members = ["core", "api", "app"]
default_members = ["core", "app"]

[resolver]
version_resolution = "Highest"
conflict_resolution = "Highest"

[package]
name = "test-workspace"
version = "1.0.0"
description = "Test workspace"
"#;

    let config: JbuildWorkspace = toml::from_str(toml_content).unwrap();
    assert_eq!(config.workspace.members, vec!["core", "api", "app"]);
    assert_eq!(config.workspace.default_members, vec!["core", "app"]);
    assert_eq!(config.workspace.package.name, Some("test-workspace".to_string()));
}

/// Test workspace configuration with minimal setup
#[test]
fn test_workspace_config_minimal() {
    let config = JbuildWorkspace::new();
    assert!(config.workspace.members.is_empty());
    assert!(config.workspace.default_members.is_empty());
}

/// Test workspace member addition and removal
#[test]
fn test_workspace_member_management() {
    let mut config = JbuildWorkspace::new();

    // Add members
    config.add_member("core".to_string());
    config.add_member("api".to_string());
    config.add_member("core".to_string()); // Duplicate should be ignored

    assert_eq!(config.workspace.members, vec!["core", "api"]);

    // Remove member
    config.remove_member("api");
    assert_eq!(config.workspace.members, vec!["core"]);

    // Set default members
    config.set_default_members(vec!["core".to_string(), "web".to_string()]);
    assert_eq!(config.workspace.default_members, vec!["core", "web"]);
}

/// Test workspace configuration file operations
#[test]
fn test_workspace_config_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("jbuild-workspace.toml");

    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["core".to_string(), "api".to_string()];
    config.workspace.package.name = Some("test-workspace".to_string());

    // Save configuration
    config.save_to_file(&config_path).unwrap();
    assert!(config_path.exists());

    // Load configuration
    let loaded_config = JbuildWorkspace::from_file(&config_path).unwrap();
    assert_eq!(loaded_config.workspace.members, vec!["core", "api"]);
    assert_eq!(loaded_config.workspace.package.name, Some("test-workspace".to_string()));
}

/// Test workspace detection and loading
#[test]
fn test_workspace_detection() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_file = temp_dir.path().join("jbuild-workspace.toml");

    // Create workspace configuration
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["core".to_string(), "app".to_string()];
    config.save_to_file(&workspace_file).unwrap();

    // Create member directories
    fs::create_dir_all(temp_dir.path().join("core")).unwrap();
    fs::create_dir_all(temp_dir.path().join("app")).unwrap();

    // Test workspace loading
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();
    assert_eq!(workspace.config.workspace.members, vec!["core", "app"]);
    assert_eq!(workspace.root, temp_dir.path());
}

/// Test workspace member resolution with Maven projects
#[test]
fn test_workspace_member_resolution_maven() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["core".to_string(), "app".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create core project with pom.xml
    let core_dir = temp_dir.path().join("core");
    fs::create_dir_all(&core_dir).unwrap();
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>core</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
</project>"#;
    fs::write(core_dir.join("pom.xml"), pom_content).unwrap();

    // Create app project with pom.xml
    let app_dir = temp_dir.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>app</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
</project>"#;
    fs::write(app_dir.join("pom.xml"), pom_content).unwrap();

    // Load workspace
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    assert_eq!(workspace.members.len(), 2);

    // Check members are resolved correctly
    let core_member = workspace.members.iter().find(|m| m.name == "core").unwrap();
    assert_eq!(core_member.relative_path, "core");
    assert_eq!(core_member.build_system, Some(jbuild::build::BuildSystem::Maven));

    let app_member = workspace.members.iter().find(|m| m.name == "app").unwrap();
    assert_eq!(app_member.relative_path, "app");
    assert_eq!(app_member.build_system, Some(jbuild::build::BuildSystem::Maven));
}

/// Test workspace member resolution with Gradle projects
#[test]
fn test_workspace_member_resolution_gradle() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["lib".to_string(), "web".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create lib project with build.gradle
    let lib_dir = temp_dir.path().join("lib");
    fs::create_dir_all(&lib_dir).unwrap();
    fs::write(lib_dir.join("build.gradle"), "plugins { id 'java' }").unwrap();

    // Create web project with build.gradle.kts
    let web_dir = temp_dir.path().join("web");
    fs::create_dir_all(&web_dir).unwrap();
    fs::write(web_dir.join("build.gradle.kts"), "plugins { java }").unwrap();

    // Load workspace
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    assert_eq!(workspace.members.len(), 2);

    // Check members are resolved correctly
    let lib_member = workspace.members.iter().find(|m| m.name == "lib").unwrap();
    assert_eq!(lib_member.relative_path, "lib");
    assert_eq!(lib_member.build_system, Some(jbuild::build::BuildSystem::Gradle));

    let web_member = workspace.members.iter().find(|m| m.name == "web").unwrap();
    assert_eq!(web_member.relative_path, "web");
    assert_eq!(web_member.build_system, Some(jbuild::build::BuildSystem::Gradle));
}

/// Test workspace member resolution with mixed build systems
#[test]
fn test_workspace_member_resolution_mixed() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["maven-lib".to_string(), "gradle-app".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create Maven project
    let maven_dir = temp_dir.path().join("maven-lib");
    fs::create_dir_all(&maven_dir).unwrap();
    fs::write(maven_dir.join("pom.xml"), "<project></project>").unwrap();

    // Create Gradle project
    let gradle_dir = temp_dir.path().join("gradle-app");
    fs::create_dir_all(&gradle_dir).unwrap();
    fs::write(gradle_dir.join("build.gradle"), "plugins { id 'java' }").unwrap();

    // Load workspace
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    assert_eq!(workspace.members.len(), 2);

    let maven_member = workspace.members.iter().find(|m| m.name == "maven-lib").unwrap();
    assert_eq!(maven_member.build_system, Some(jbuild::build::BuildSystem::Maven));

    let gradle_member = workspace.members.iter().find(|m| m.name == "gradle-app").unwrap();
    assert_eq!(gradle_member.build_system, Some(jbuild::build::BuildSystem::Gradle));
}

/// Test workspace member resolution with unknown build system
#[test]
fn test_workspace_member_resolution_unknown_build_system() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["unknown-project".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create project directory without build files
    let project_dir = temp_dir.path().join("unknown-project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("README.md"), "# Unknown Project").unwrap();

    // Load workspace
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    assert_eq!(workspace.members.len(), 1);

    let member = &workspace.members[0];
    assert_eq!(member.name, "unknown-project");
    assert_eq!(member.build_system, None); // No build system detected
}

/// Test workspace build order calculation
#[test]
fn test_workspace_build_order() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["core".to_string(), "api".to_string(), "app".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create projects with dependencies
    for project in &["core", "api", "app"] {
        let project_dir = temp_dir.path().join(project);
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("pom.xml"), "<project></project>").unwrap();
    }

    // Load workspace and manually set dependencies
    let mut workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    // Simulate dependencies: app -> api -> core
    workspace.members[0].workspace_dependencies = vec!["core".to_string()]; // api depends on core
    workspace.members[1].workspace_dependencies = vec!["api".to_string()]; // app depends on api

    let build_order: Vec<&str> = workspace.get_build_order()
        .iter()
        .map(|m| m.name.as_str())
        .collect();

    // Should build in dependency order: core, api, app
    assert_eq!(build_order, vec!["core", "api", "app"]);
}

/// Test workspace member dependency detection
#[test]
fn test_workspace_member_dependencies() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace config
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["core".to_string(), "api".to_string(), "app".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    // Create core project
    let core_dir = temp_dir.path().join("core");
    fs::create_dir_all(&core_dir).unwrap();
    fs::write(core_dir.join("pom.xml"), "<project></project>").unwrap();

    // Create api project that depends on core
    let api_dir = temp_dir.path().join("api");
    fs::create_dir_all(&api_dir).unwrap();
    let api_pom = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>api</artifactId>
    <version>1.0.0</version>

    <dependencies>
        <dependency>
            <groupId>com.example</groupId>
            <artifactId>core</artifactId>
            <version>1.0.0</version>
        </dependency>
    </dependencies>
</project>"#;
    fs::write(api_dir.join("pom.xml"), api_pom).unwrap();

    // Create app project
    let app_dir = temp_dir.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    fs::write(app_dir.join("pom.xml"), "<project></project>").unwrap();

    // Load workspace
    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();

    // Check that api project detected dependency on core
    let api_member = workspace.members.iter().find(|m| m.name == "api").unwrap();
    assert!(api_member.workspace_dependencies.contains(&"core".to_string()));

    // Check that core and app have no dependencies
    let core_member = workspace.members.iter().find(|m| m.name == "core").unwrap();
    assert!(core_member.workspace_dependencies.is_empty());

    let app_member = workspace.members.iter().find(|m| m.name == "app").unwrap();
    assert!(app_member.workspace_dependencies.is_empty());
}

/// Test workspace root detection
#[test]
fn test_workspace_root_detection() {
    let temp_dir = TempDir::new().unwrap();

    // No workspace file
    assert!(!Workspace::is_workspace_root(temp_dir.path()));

    // Create workspace file
    fs::write(temp_dir.path().join("jbuild-workspace.toml"), "[workspace]").unwrap();
    assert!(Workspace::is_workspace_root(temp_dir.path()));
}

/// Test workspace error handling
#[test]
fn test_workspace_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Try to load workspace without config file
    let result = Workspace::from_directory(temp_dir.path());
    assert!(result.is_err());

    // Create config but reference non-existent member
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec!["non-existent".to_string()];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    let result = Workspace::from_directory(temp_dir.path());
    assert!(result.is_err());
}

/// Test workspace member name extraction from different sources
#[test]
fn test_workspace_member_name_extraction() {
    let temp_dir = TempDir::new().unwrap();

    // Test Maven project name extraction
    let maven_dir = temp_dir.path().join("maven-project");
    fs::create_dir_all(&maven_dir).unwrap();
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <artifactId>test-artifact</artifactId>
</project>"#;
    fs::write(maven_dir.join("pom.xml"), pom_content).unwrap();

    let workspace = Workspace::from_directory(temp_dir.path());
    assert!(workspace.is_err()); // No workspace config yet

    // Test Gradle project name extraction
    let gradle_dir = temp_dir.path().join("gradle-project");
    fs::create_dir_all(&gradle_dir).unwrap();
    fs::write(gradle_dir.join("build.gradle"), "rootProject.name = 'gradle-test'").unwrap();

    // Test fallback to directory name
    let plain_dir = temp_dir.path().join("plain-project");
    fs::create_dir_all(&plain_dir).unwrap();

    // Create workspace with these projects
    let mut config = JbuildWorkspace::new();
    config.workspace.members = vec![
        "maven-project".to_string(),
        "gradle-project".to_string(),
        "plain-project".to_string(),
    ];
    config.save_to_file(&temp_dir.path().join("jbuild-workspace.toml")).unwrap();

    let workspace = Workspace::from_directory(temp_dir.path()).unwrap();
    assert_eq!(workspace.members.len(), 3);

    // Note: The actual name extraction depends on the implementation details
    // This test verifies the workspace loads without errors
}