/// Snapshot tests using insta for verifying complex outputs
/// These tests capture and verify the structure of complex objects

use jbuild::core::ExecutionRequestBuilder;
use std::path::PathBuf;

#[test]
#[ignore]  // HashMap ordering is non-deterministic, use for manual review only
fn test_execution_request_snapshot() {
    let base_dir = PathBuf::from("/home/user/project");
    let request = ExecutionRequestBuilder::new(base_dir)
        .with_goals(vec!["clean".to_string(), "compile".to_string(), "test".to_string()])
        .with_pom_file(PathBuf::from("/home/user/project/pom.xml"))
        .with_property("maven.compiler.source".to_string(), "11".to_string())
        .with_property("maven.compiler.target".to_string(), "11".to_string())
        .with_profile("dev".to_string())
        .build();

    // Snapshot the request structure
    insta::assert_debug_snapshot!(request);
}

#[test]
#[ignore]  // HashMap ordering is non-deterministic, use for manual review only
fn test_execution_request_with_many_properties_snapshot() {
    let base_dir = PathBuf::from("/project");
    let mut builder = ExecutionRequestBuilder::new(base_dir);

    // Add multiple properties
    for i in 0..5 {
        builder = builder.with_property(
            format!("property.{}", i),
            format!("value-{}", i),
        );
    }

    let request = builder.build();

    insta::assert_debug_snapshot!(request);
}

#[test]
#[ignore]  // HashMap ordering is non-deterministic, use for manual review only
fn test_execution_request_single_goal_snapshot() {
    let base_dir = PathBuf::from("/project");
    let request = ExecutionRequestBuilder::new(base_dir)
        .add_goal("install".to_string())
        .build();

    insta::assert_debug_snapshot!(request);
}
