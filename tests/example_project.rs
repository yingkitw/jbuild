use std::path::PathBuf;
use std::fs;
use mvn_rs::model::parse_pom;
use mvn_rs::Model;
use mvn_rs::model::ModelValidator;
use mvn_rs::model::PropertyInterpolator;
use mvn_rs::ArtifactCoordinates;

/// Get the path to the example project
fn example_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("simple-java-project")
}

#[test]
fn test_example_project_pom_exists() {
    let pom_path = example_project_path().join("pom.xml");
    assert!(pom_path.exists(), "pom.xml should exist in example project");
}

#[test]
fn test_parse_example_project_pom() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    assert_eq!(model.group_id, "com.example");
    assert_eq!(model.artifact_id, "simple-java-project");
    assert_eq!(model.version, "1.0.0");
    assert_eq!(model.packaging, "jar");
}

#[test]
fn test_validate_example_project_model() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    let errors = ModelValidator::validate(&model)
        .expect("Validation should succeed");
    
    assert!(errors.is_empty(), "Model should be valid, but got errors: {:?}", errors);
}

#[test]
fn test_example_project_source_files_exist() {
    let main_java = example_project_path()
        .join("src")
        .join("main")
        .join("java")
        .join("com")
        .join("example")
        .join("App.java");
    
    assert!(main_java.exists(), "App.java should exist");
    
    let test_java = example_project_path()
        .join("src")
        .join("test")
        .join("java")
        .join("com")
        .join("example")
        .join("AppTest.java");
    
    assert!(test_java.exists(), "AppTest.java should exist");
}

#[test]
fn test_example_project_dependencies() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    assert!(model.dependencies.is_some(), "Project should have dependencies");
    let dependencies = model.dependencies_vec();
    
    assert!(!dependencies.is_empty(), "Project should have at least one dependency");
    
    // Check for JUnit dependency
    let junit_dep = dependencies.iter()
        .find(|d| d.group_id == "junit" && d.artifact_id == "junit");
    
    assert!(junit_dep.is_some(), "Project should have JUnit dependency");
    
    if let Some(dep) = junit_dep {
        assert_eq!(dep.version, Some("4.13.2".to_string()));
        assert_eq!(dep.scope, Some("test".to_string()));
    }
}

#[test]
fn test_example_project_coordinates() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    let coords = ArtifactCoordinates::new(
        &model.group_id,
        &model.artifact_id,
        &model.version,
    );
    
    assert_eq!(coords.id(), "com.example:simple-java-project");
    assert_eq!(coords.full_id(), "com.example:simple-java-project:1.0.0");
}

#[test]
fn test_example_project_properties() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    if let Some(properties) = &model.properties {
        assert!(properties.contains_key("maven.compiler.source"));
        assert!(properties.contains_key("maven.compiler.target"));
        assert_eq!(properties.get("maven.compiler.source"), Some(&"11".to_string()));
        assert_eq!(properties.get("maven.compiler.target"), Some(&"11".to_string()));
    }
}

#[test]
fn test_example_project_build_configuration() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    assert!(model.build.is_some(), "Project should have build configuration");
    
    if let Some(build) = &model.build {
        assert_eq!(build.source_directory, Some("src/main/java".to_string()));
        assert_eq!(build.test_source_directory, Some("src/test/java".to_string()));
        assert_eq!(build.output_directory, Some("target/classes".to_string()));
        assert_eq!(build.test_output_directory, Some("target/test-classes".to_string()));
    }
}

#[test]
fn test_example_project_resource_files() {
    let resource_file = example_project_path()
        .join("src")
        .join("main")
        .join("resources")
        .join("application.properties");
    
    assert!(resource_file.exists(), "application.properties should exist");
    
    let content = fs::read_to_string(&resource_file)
        .expect("Failed to read application.properties");
    
    assert!(content.contains("app.name"), "Properties file should contain app.name");
    assert!(content.contains("Simple Java Project"), "Properties file should contain project name");
}

#[test]
fn test_example_project_directory_structure() {
    let project_path = example_project_path();
    
    // Check main source directory
    let main_java_dir = project_path.join("src").join("main").join("java");
    assert!(main_java_dir.exists(), "src/main/java should exist");
    assert!(main_java_dir.is_dir(), "src/main/java should be a directory");
    
    // Check test source directory
    let test_java_dir = project_path.join("src").join("test").join("java");
    assert!(test_java_dir.exists(), "src/test/java should exist");
    assert!(test_java_dir.is_dir(), "src/test/java should be a directory");
    
    // Check resources directory
    let resources_dir = project_path.join("src").join("main").join("resources");
    assert!(resources_dir.exists(), "src/main/resources should exist");
    assert!(resources_dir.is_dir(), "src/main/resources should be a directory");
}

#[test]
fn test_property_interpolation_with_example_project() {
    let pom_path = example_project_path().join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)
        .expect("Failed to read pom.xml");
    
    let model = parse_pom(&pom_content)
        .expect("Failed to parse pom.xml");
    
    // Create interpolator with project properties
    let mut props = std::collections::HashMap::new();
    props.insert("project.groupId".to_string(), model.group_id.clone());
    props.insert("project.artifactId".to_string(), model.artifact_id.clone());
    props.insert("project.version".to_string(), model.version.clone());
    
    if let Some(model_props) = &model.properties {
        for (key, value) in model_props {
            props.insert(key.clone(), value.clone());
        }
    }
    
    let interpolator = PropertyInterpolator::new()
        .add_properties(props);
    
    // Test interpolation
    let result = interpolator.interpolate("${project.groupId}:${project.artifactId}:${project.version}")
        .expect("Interpolation should succeed");
    
    assert_eq!(result, "com.example:simple-java-project:1.0.0");
}

