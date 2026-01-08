/// Tests for Gradle functionality

use jbuild::gradle::parse_gradle_build_script;
use std::path::PathBuf;

#[test]
fn test_parse_simple_gradle_build() {
    let build_file = PathBuf::from("examples/simple-gradle-project/build.gradle");
    let base_dir = PathBuf::from("examples/simple-gradle-project");
    
    if !build_file.exists() {
        // Skip test if example project doesn't exist
        return;
    }
    
    let project = parse_gradle_build_script(&build_file, &base_dir).unwrap();
    
    assert_eq!(project.plugins.len(), 1);
    assert_eq!(project.plugins[0].id, "java");
    assert_eq!(project.group, Some("com.example".to_string()));
    assert_eq!(project.version, Some("1.0.0".to_string()));
    assert!(!project.repositories.is_empty());
    assert!(project.dependencies.len() >= 2);
}

#[test]
fn test_gradle_project_find_task() {
    use jbuild::gradle::{GradleProject, Task};
    
    let mut project = GradleProject::new(
        PathBuf::from("/test"),
        PathBuf::from("/test/build.gradle"),
    );
    
    project.tasks.push(Task {
        name: "build".to_string(),
        task_type: Some("Standard".to_string()),
        description: None,
        group: Some("build".to_string()),
        depends_on: Vec::new(),
        actions: Vec::new(),
    });
    
    assert!(project.find_task("build").is_some());
    assert!(project.find_task("nonexistent").is_none());
}

#[test]
fn test_gradle_task_names() {
    use jbuild::gradle::{GradleProject, Task};
    
    let mut project = GradleProject::new(
        PathBuf::from("/test"),
        PathBuf::from("/test/build.gradle"),
    );
    
    project.tasks.push(Task {
        name: "clean".to_string(),
        task_type: None,
        description: None,
        group: None,
        depends_on: Vec::new(),
        actions: Vec::new(),
    });
    
    project.tasks.push(Task {
        name: "build".to_string(),
        task_type: None,
        description: None,
        group: None,
        depends_on: Vec::new(),
        actions: Vec::new(),
    });
    
    let task_names = project.task_names();
    assert!(task_names.contains(&"clean".to_string()));
    assert!(task_names.contains(&"build".to_string()));
    assert_eq!(task_names.len(), 2);
}

