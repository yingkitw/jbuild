use jbuild::build::{BuildSystem, BuildWrapper, WrapperType};
use std::fs;
use tempfile::TempDir;

/// Test Maven build system detection
#[test]
fn test_build_system_detection_maven() {
    let temp_dir = TempDir::new().unwrap();

    // No build system initially
    assert_eq!(BuildSystem::detect(temp_dir.path()), None);

    // Add pom.xml
    fs::write(temp_dir.path().join("pom.xml"), "<project></project>").unwrap();
    assert_eq!(BuildSystem::detect(temp_dir.path()), Some(BuildSystem::Maven));

    // Parent directory detection
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    assert_eq!(BuildSystem::detect(&subdir), Some(BuildSystem::Maven));
}

/// Test Gradle build system detection
#[test]
fn test_build_system_detection_gradle() {
    let temp_dir = TempDir::new().unwrap();

    // Test build.gradle detection
    fs::write(temp_dir.path().join("build.gradle"), "plugins { id 'java' }").unwrap();
    assert_eq!(BuildSystem::detect(temp_dir.path()), Some(BuildSystem::Gradle));

    // Test build.gradle.kts detection
    fs::remove_file(temp_dir.path().join("build.gradle")).unwrap();
    fs::write(temp_dir.path().join("build.gradle.kts"), "plugins { java }").unwrap();
    assert_eq!(BuildSystem::detect(temp_dir.path()), Some(BuildSystem::Gradle));
}

/// Test build system priority (Maven takes precedence)
#[test]
fn test_build_system_detection_priority() {
    let temp_dir = TempDir::new().unwrap();

    // Add both pom.xml and build.gradle
    fs::write(temp_dir.path().join("pom.xml"), "<project></project>").unwrap();
    fs::write(temp_dir.path().join("build.gradle"), "plugins { id 'java' }").unwrap();

    // Maven should be detected first
    assert_eq!(BuildSystem::detect(temp_dir.path()), Some(BuildSystem::Maven));
}

/// Test build file path generation
#[test]
fn test_build_file_path_generation() {
    let temp_dir = TempDir::new().unwrap();

    assert_eq!(
        BuildSystem::Maven.build_file_path(temp_dir.path()),
        temp_dir.path().join("pom.xml")
    );

    assert_eq!(
        BuildSystem::Gradle.build_file_path(temp_dir.path()),
        temp_dir.path().join("build.gradle")
    );
}

/// Test build file name generation
#[test]
fn test_build_file_name_generation() {
    assert_eq!(BuildSystem::Maven.build_file_name(), "pom.xml");
    assert_eq!(BuildSystem::Gradle.build_file_name(), "build.gradle");
}

/// Test Maven wrapper detection
#[test]
fn test_maven_wrapper_detection() {
    let temp_dir = TempDir::new().unwrap();

    // No wrapper initially
    assert!(BuildWrapper::detect(temp_dir.path()).is_none());

    // Add mvnw script
    fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash\necho mvnw").unwrap();
    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.wrapper_type, WrapperType::Maven);
    assert_eq!(wrapper.script_path, temp_dir.path().join("mvnw"));
}

/// Test Gradle wrapper detection
#[test]
fn test_gradle_wrapper_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Add gradlew script
    fs::write(temp_dir.path().join("gradlew"), "#!/bin/bash\necho gradlew").unwrap();
    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.wrapper_type, WrapperType::Gradle);
    assert_eq!(wrapper.script_path, temp_dir.path().join("gradlew"));
}

/// Test wrapper priority (Gradle takes precedence in implementation)
#[test]
fn test_wrapper_detection_priority() {
    let temp_dir = TempDir::new().unwrap();

    // Add both wrappers
    fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash\necho mvnw").unwrap();
    fs::write(temp_dir.path().join("gradlew"), "#!/bin/bash\necho gradlew").unwrap();

    // Gradle wrapper is detected first (as per implementation)
    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.wrapper_type, WrapperType::Gradle);
}

/// Test Maven wrapper properties parsing
#[test]
fn test_maven_wrapper_properties_parsing() {
    let temp_dir = TempDir::new().unwrap();

    // Create mvnw script
    fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash\necho mvnw").unwrap();

    // Create wrapper properties
    fs::create_dir_all(temp_dir.path().join(".mvn/wrapper")).unwrap();
    let properties_content = r#"#Maven Wrapper Properties
distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.4/apache-maven-3.9.4-bin.zip
wrapperUrl=https://repo.maven.apache.org/maven2/org/apache/maven/wrapper/maven-wrapper/3.2.0/maven-wrapper-3.2.0.jar"#;
    fs::write(
        temp_dir.path().join(".mvn/wrapper/maven-wrapper.properties"),
        properties_content,
    ).unwrap();

    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.get_version(), Some("3.9.4".to_string()));
}

/// Test Gradle wrapper properties parsing
#[test]
fn test_gradle_wrapper_properties_parsing() {
    let temp_dir = TempDir::new().unwrap();

    // Create gradlew script
    fs::write(temp_dir.path().join("gradlew"), "#!/bin/bash\necho gradlew").unwrap();

    // Create wrapper properties
    fs::create_dir_all(temp_dir.path().join("gradle/wrapper")).unwrap();
    let properties_content = r#"#Gradle Wrapper Properties
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.4-bin.zip
networkTimeout=10000
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    fs::write(
        temp_dir.path().join("gradle/wrapper/gradle-wrapper.properties"),
        properties_content,
    ).unwrap();

    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.get_version(), Some("8.4".to_string()));
}

/// Test wrapper version parsing with different formats
#[test]
fn test_wrapper_version_parsing_formats() {
    let test_cases = vec![
        ("https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.4/apache-maven-3.9.4-bin.zip", Some("3.9.4".to_string())),
        ("https://services.gradle.org/distributions/gradle-8.4-bin.zip", Some("8.4".to_string())),
        ("https://services.gradle.org/distributions/gradle-7.6.1-all.zip", Some("7.6.1".to_string())),
        ("invalid-url", None),
        ("", None),
    ];

    for (url, expected_version) in test_cases {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash").unwrap();
        fs::create_dir_all(temp_dir.path().join(".mvn/wrapper")).unwrap();

        let properties = format!("distributionUrl={}", url);
        fs::write(
            temp_dir.path().join(".mvn/wrapper/maven-wrapper.properties"),
            properties,
        ).unwrap();

        let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
        assert_eq!(wrapper.get_version(), expected_version, "Failed for URL: {}", url);
    }
}

/// Test build system default goals
#[test]
fn test_build_system_default_goals() {
    assert_eq!(BuildSystem::Maven.default_goals(), vec!["compile".to_string()]);
    assert_eq!(BuildSystem::Gradle.default_goals(), vec!["build".to_string()]);
}

/// Test wrapper script path resolution
#[test]
fn test_wrapper_script_path_resolution() {
    let temp_dir = TempDir::new().unwrap();

    // Test Maven wrapper
    fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash").unwrap();
    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.script_path.file_name().unwrap(), "mvnw");

    // Test Gradle wrapper
    fs::remove_file(temp_dir.path().join("mvnw")).unwrap();
    fs::write(temp_dir.path().join("gradlew"), "#!/bin/bash").unwrap();
    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.script_path.file_name().unwrap(), "gradlew");
}

/// Test wrapper detection with missing properties file
#[test]
fn test_wrapper_detection_missing_properties() {
    let temp_dir = TempDir::new().unwrap();

    // Create wrapper script but no properties file
    fs::write(temp_dir.path().join("mvnw"), "#!/bin/bash").unwrap();

    let wrapper = BuildWrapper::detect(temp_dir.path()).unwrap();
    assert_eq!(wrapper.get_version(), None);
}

/// Test build system detection in subdirectories
#[test]
fn test_build_system_detection_subdirectory() {
    let temp_dir = TempDir::new().unwrap();

    // Create build file in root
    fs::write(temp_dir.path().join("pom.xml"), "<project></project>").unwrap();

    // Test detection from subdirectory
    let subdir = temp_dir.path().join("src/main/java/com/example");
    fs::create_dir_all(&subdir).unwrap();

    assert_eq!(BuildSystem::detect(&subdir), Some(BuildSystem::Maven));
}

/// Test build system detection with symlinks
#[test]
fn test_build_system_detection_with_symlinks() {
    let temp_dir = TempDir::new().unwrap();
    let linked_dir = temp_dir.path().join("linked");

    // Create actual project directory
    let actual_dir = temp_dir.path().join("actual");
    fs::create_dir_all(&actual_dir).unwrap();
    fs::write(actual_dir.join("pom.xml"), "<project></project>").unwrap();

    // Create symlink (if supported)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&actual_dir, &linked_dir).unwrap();

        // Should detect build system through symlink
        assert_eq!(BuildSystem::detect(&linked_dir), Some(BuildSystem::Maven));
    }
}

/// Test wrapper type enum values
#[test]
fn test_wrapper_type_enum() {
    assert_eq!(WrapperType::Maven as u8, 0);
    assert_eq!(WrapperType::Gradle as u8, 1);

    // Test Debug formatting
    assert_eq!(format!("{:?}", WrapperType::Maven), "Maven");
    assert_eq!(format!("{:?}", WrapperType::Gradle), "Gradle");
}

/// Test build system enum values
#[test]
fn test_build_system_enum() {
    assert_eq!(BuildSystem::Maven as u8, 0);
    assert_eq!(BuildSystem::Gradle as u8, 1);

    // Test Debug formatting
    assert_eq!(format!("{:?}", BuildSystem::Maven), "Maven");
    assert_eq!(format!("{:?}", BuildSystem::Gradle), "Gradle");
}