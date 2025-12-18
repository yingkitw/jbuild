use std::collections::HashMap;
use jbuild::model::{Model, Dependencies, Dependency};
use jbuild::model::model::DependencyManagement;
use jbuild::model::model_builder::ModelBuilder;

#[test]
fn test_inherit_basic_info() {
    let parent = Model {
        model_version: "4.0.0".to_string(),
        group_id: "com.example".to_string(),
        artifact_id: "parent".to_string(),
        version: "1.0.0".to_string(),
        packaging: "pom".to_string(),
        ..Default::default()
    };
    
    let child = Model {
        model_version: "4.0.0".to_string(),
        group_id: "".to_string(),
        artifact_id: "child".to_string(),
        version: "".to_string(),
        packaging: "".to_string(),
        ..Default::default()
    };
    
    let builder = ModelBuilder::new();
    let effective = builder.build_effective_model(child, Some(parent));
    
    assert_eq!(effective.group_id, "com.example");
    assert_eq!(effective.version, "1.0.0");
    assert_eq!(effective.packaging, "pom");
}

#[test]
fn test_merge_properties() {
    let mut parent_props = HashMap::new();
    parent_props.insert("prop1".to_string(), "val1".to_string());
    parent_props.insert("prop2".to_string(), "val2".to_string());
    
    let parent = Model {
        properties: Some(parent_props),
        ..Default::default()
    };
    
    let mut child_props = HashMap::new();
    child_props.insert("prop2".to_string(), "child-val2".to_string());
    child_props.insert("prop3".to_string(), "val3".to_string());
    
    let child = Model {
        properties: Some(child_props),
        ..Default::default()
    };
    
    let builder = ModelBuilder::new();
    let effective = builder.build_effective_model(child, Some(parent));
    
    let props = effective.properties.unwrap();
    assert_eq!(props.get("prop1").unwrap(), "val1");
    assert_eq!(props.get("prop2").unwrap(), "child-val2");
    assert_eq!(props.get("prop3").unwrap(), "val3");
}

#[test]
fn test_dependency_management_inheritance() {
    let mut parent_deps = Vec::new();
    parent_deps.push(Dependency {
        group_id: "org.slf4j".to_string(),
        artifact_id: "slf4j-api".to_string(),
        version: Some("1.7.36".to_string()),
        ..Default::default()
    });
    
    let parent = Model {
        dependency_management: Some(DependencyManagement {
            dependencies: Some(Dependencies { dependencies: parent_deps }),
        }),
        ..Default::default()
    };
    
    let child = Model {
        ..Default::default()
    };
    
    let builder = ModelBuilder::new();
    let effective = builder.build_effective_model(child, Some(parent));
    
    let dep_mgmt = effective.dependency_management.unwrap();
    let deps = dep_mgmt.dependencies.unwrap().dependencies;
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].group_id, "org.slf4j");
    assert_eq!(deps[0].version.as_ref().unwrap(), "1.7.36");
}

#[test]
fn test_interpolation() {
    let mut props = HashMap::new();
    props.insert("my.version".to_string(), "1.2.3".to_string());
    
    let mut model = Model {
        group_id: "com.example".to_string(),
        artifact_id: "test".to_string(),
        version: "${my.version}".to_string(),
        properties: Some(props.clone()),
        ..Default::default()
    };
    
    let builder = ModelBuilder::new();
    builder.interpolate(&mut model, &props);
    
    assert_eq!(model.version, "1.2.3");
}
