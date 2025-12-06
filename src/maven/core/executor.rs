//! Maven executor implementation of BuildExecutor trait

use anyhow::Result;
use crate::build::{BuildExecutor, BuildSystem, ExecutionRequest, ExecutionResult};
use crate::core::default_maven::DefaultMaven;
use crate::core::execution::MavenExecutionRequest;

/// Maven executor that implements BuildExecutor trait
pub struct MavenBuildExecutor {
    maven: DefaultMaven,
}

impl MavenBuildExecutor {
    pub fn new() -> Self {
        Self {
            maven: DefaultMaven::new(),
        }
    }
}

impl BuildExecutor for MavenBuildExecutor {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        // Convert generic ExecutionRequest to MavenExecutionRequest
        let mut maven_request = MavenExecutionRequest::new(request.base_directory);
        maven_request.goals = request.goals;
        maven_request.system_properties = request.system_properties;
        maven_request.show_errors = request.show_errors;
        // Note: offline mode would need to be handled via settings

        // Execute Maven build
        let maven_result = self.maven.execute(maven_request)?;

        // Convert MavenExecutionResult to ExecutionResult
        Ok(ExecutionResult {
            success: maven_result.success,
            errors: maven_result.exceptions.iter().map(|e| e.to_string()).collect(),
        })
    }

    fn build_system(&self) -> BuildSystem {
        BuildSystem::Maven
    }
}

