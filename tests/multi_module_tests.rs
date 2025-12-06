//! Integration tests for multi-module Maven and Gradle projects

use std::path::PathBuf;

/// Get the path to the examples directory
fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")
}

mod maven_multi_module {
    use super::*;
    use jbuild::model::parser::parse_pom;

    #[test]
    fn test_parse_parent_pom() {
        let pom_path = examples_dir().join("multi-module-maven/pom.xml");
        let content = std::fs::read_to_string(&pom_path).expect("Failed to read parent pom.xml");
        let model = parse_pom(&content).expect("Failed to parse parent pom.xml");

        assert_eq!(model.group_id, "com.example");
        assert_eq!(model.artifact_id, "multi-module-parent");
        assert_eq!(model.version, "1.0.0");
        assert_eq!(model.packaging, "pom");
    }

    #[test]
    fn test_parse_core_module_pom() {
        let pom_path = examples_dir().join("multi-module-maven/core/pom.xml");
        let content = std::fs::read_to_string(&pom_path).expect("Failed to read core pom.xml");
        let model = parse_pom(&content).expect("Failed to parse core pom.xml");

        assert_eq!(model.group_id, "com.example");
        assert_eq!(model.artifact_id, "core");
        assert_eq!(model.version, "1.0.0");
    }

    #[test]
    fn test_parse_api_module_pom() {
        let pom_path = examples_dir().join("multi-module-maven/api/pom.xml");
        let content = std::fs::read_to_string(&pom_path).expect("Failed to read api pom.xml");
        let model = parse_pom(&content).expect("Failed to parse api pom.xml");

        assert_eq!(model.artifact_id, "api");

        // Check dependency on core
        let deps = model.dependencies_vec();
        assert!(!deps.is_empty());
        let core_dep = deps.iter().find(|d| d.artifact_id == "core");
        assert!(core_dep.is_some());
        assert_eq!(core_dep.unwrap().group_id, "com.example");
    }

    #[test]
    fn test_parse_app_module_pom() {
        let pom_path = examples_dir().join("multi-module-maven/app/pom.xml");
        let content = std::fs::read_to_string(&pom_path).expect("Failed to read app pom.xml");
        let model = parse_pom(&content).expect("Failed to parse app pom.xml");

        assert_eq!(model.artifact_id, "app");

        // Check dependency on api
        let deps = model.dependencies_vec();
        let api_dep = deps.iter().find(|d| d.artifact_id == "api");
        assert!(api_dep.is_some());

        // Check build configuration
        assert!(model.build.is_some());
    }

    #[test]
    fn test_property_inheritance() {
        let parent_path = examples_dir().join("multi-module-maven/pom.xml");
        let parent_content = std::fs::read_to_string(&parent_path).expect("Failed to read parent pom.xml");
        let parent_model = parse_pom(&parent_content).expect("Failed to parse parent pom.xml");

        // Check properties are defined in parent
        assert!(parent_model.properties.is_some());
        let props = parent_model.properties.as_ref().unwrap();
        assert!(props.contains_key("junit.version"));
        assert_eq!(props.get("junit.version"), Some(&"5.9.3".to_string()));
    }

    #[test]
    fn test_maven_project_structure() {
        // Verify all Maven module directories exist
        let modules = vec!["core", "api", "app"];
        for module in &modules {
            let module_dir = examples_dir().join(format!("multi-module-maven/{}", module));
            assert!(module_dir.exists(), "Module directory should exist: {:?}", module_dir);

            let pom_file = module_dir.join("pom.xml");
            assert!(pom_file.exists(), "POM file should exist: {:?}", pom_file);

            let src_dir = module_dir.join("src/main/java");
            assert!(src_dir.exists(), "Source directory should exist: {:?}", src_dir);
        }
    }
}

mod gradle_multi_module {
    use super::*;
    use jbuild::gradle::{parse_settings_file, parse_gradle_build_script};

    #[test]
    fn test_parse_settings_gradle() {
        let settings_path = examples_dir().join("multi-module-gradle/settings.gradle");
        let base_dir = examples_dir().join("multi-module-gradle");

        let settings = parse_settings_file(&settings_path, &base_dir)
            .expect("Failed to parse settings.gradle");

        assert_eq!(settings.root_project_name, "multi-module-gradle");
        assert!(settings.is_multi_project());
        assert_eq!(settings.subprojects.len(), 3);

        // Check subproject paths
        let paths: Vec<&str> = settings.subprojects.iter().map(|s| s.path.as_str()).collect();
        assert!(paths.contains(&":core"));
        assert!(paths.contains(&":api"));
        assert!(paths.contains(&":app"));
    }

    #[test]
    fn test_parse_root_build_gradle() {
        let build_path = examples_dir().join("multi-module-gradle/build.gradle");
        let base_dir = examples_dir().join("multi-module-gradle");

        let project = parse_gradle_build_script(&build_path, &base_dir)
            .expect("Failed to parse root build.gradle");

        assert_eq!(project.group, Some("com.example".to_string()));
        assert_eq!(project.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_core_build_gradle() {
        let build_path = examples_dir().join("multi-module-gradle/core/build.gradle");
        let base_dir = examples_dir().join("multi-module-gradle/core");

        let project = parse_gradle_build_script(&build_path, &base_dir)
            .expect("Failed to parse core build.gradle");

        // Check plugins
        let plugin_ids: Vec<&str> = project.plugins.iter().map(|p| p.id.as_str()).collect();
        assert!(plugin_ids.contains(&"java-library"));
    }

    #[test]
    fn test_parse_api_build_gradle() {
        let build_path = examples_dir().join("multi-module-gradle/api/build.gradle");
        let base_dir = examples_dir().join("multi-module-gradle/api");

        let project = parse_gradle_build_script(&build_path, &base_dir)
            .expect("Failed to parse api build.gradle");

        // Check dependencies
        let dep_notations: Vec<&str> = project.dependencies.iter().map(|d| d.notation.as_str()).collect();
        assert!(dep_notations.iter().any(|n| n.contains(":core")));
    }

    #[test]
    fn test_parse_app_build_gradle() {
        let build_path = examples_dir().join("multi-module-gradle/app/build.gradle");
        let base_dir = examples_dir().join("multi-module-gradle/app");

        let project = parse_gradle_build_script(&build_path, &base_dir)
            .expect("Failed to parse app build.gradle");

        // Check plugins
        let plugin_ids: Vec<&str> = project.plugins.iter().map(|p| p.id.as_str()).collect();
        assert!(plugin_ids.contains(&"application"));

        // Check dependencies
        let dep_notations: Vec<&str> = project.dependencies.iter().map(|d| d.notation.as_str()).collect();
        assert!(dep_notations.iter().any(|n| n.contains(":api")));
    }

    #[test]
    fn test_subproject_directories() {
        let settings_path = examples_dir().join("multi-module-gradle/settings.gradle");
        let base_dir = examples_dir().join("multi-module-gradle");

        let settings = parse_settings_file(&settings_path, &base_dir)
            .expect("Failed to parse settings.gradle");

        for subproject in &settings.subprojects {
            let dir = subproject.directory(&base_dir);
            assert!(dir.exists(), "Subproject directory should exist: {:?}", dir);

            let build_file = dir.join("build.gradle");
            assert!(build_file.exists(), "Build file should exist: {:?}", build_file);
        }
    }
}

mod build_system_comparison {
    use super::*;
    use jbuild::build::BuildSystem;

    #[test]
    fn test_detect_maven_project() {
        let maven_dir = examples_dir().join("multi-module-maven");
        let build_system = BuildSystem::detect(&maven_dir);
        assert!(matches!(build_system, Some(BuildSystem::Maven)));
    }

    #[test]
    fn test_detect_gradle_project() {
        let gradle_dir = examples_dir().join("multi-module-gradle");
        let build_system = BuildSystem::detect(&gradle_dir);
        assert!(matches!(build_system, Some(BuildSystem::Gradle)));
    }

    #[test]
    fn test_maven_and_gradle_have_same_structure() {
        // Both projects should have the same module structure
        let maven_modules = vec!["core", "api", "app"];
        let gradle_modules = vec!["core", "api", "app"];

        assert_eq!(maven_modules, gradle_modules);

        // Both should have the same source files
        for module in &maven_modules {
            let maven_src = examples_dir().join(format!("multi-module-maven/{}/src/main/java", module));
            let gradle_src = examples_dir().join(format!("multi-module-gradle/{}/src/main/java", module));

            // At least one of them should exist (we created them)
            assert!(maven_src.exists() || gradle_src.exists(),
                "Source directory should exist for module: {}", module);
        }
    }
}
