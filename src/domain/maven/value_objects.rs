//! Value objects for Maven context

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Maven lifecycle phase value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhase {
    // Clean lifecycle
    PreClean,
    Clean,
    PostClean,

    // Default lifecycle
    Validate,
    Initialize,
    GenerateSources,
    ProcessSources,
    GenerateResources,
    ProcessResources,
    Compile,
    ProcessClasses,
    GenerateTestSources,
    ProcessTestSources,
    GenerateTestResources,
    ProcessTestResources,
    TestCompile,
    ProcessTestClasses,
    Test,
    PreparePackage,
    Package,
    PreIntegrationTest,
    IntegrationTest,
    PostIntegrationTest,
    Verify,
    Install,
    Deploy,

    // Site lifecycle
    PreSite,
    Site,
    PostSite,
    SiteDeploy,
}

crate::impl_data_driven_enum!(LifecyclePhase, {
    // Clean lifecycle
    PreClean => { name: "pre-clean", order: 0 },
    Clean => { name: "clean", order: 1 },
    PostClean => { name: "post-clean", order: 2 },

    // Default lifecycle
    Validate => { name: "validate", order: 100 },
    Initialize => { name: "initialize", order: 101 },
    GenerateSources => { name: "generate-sources", order: 102 },
    ProcessSources => { name: "process-sources", order: 103 },
    GenerateResources => { name: "generate-resources", order: 104 },
    ProcessResources => { name: "process-resources", order: 105 },
    Compile => { name: "compile", order: 106 },
    ProcessClasses => { name: "process-classes", order: 107 },
    GenerateTestSources => { name: "generate-test-sources", order: 108 },
    ProcessTestSources => { name: "process-test-sources", order: 109 },
    GenerateTestResources => { name: "generate-test-resources", order: 110 },
    ProcessTestResources => { name: "process-test-resources", order: 111 },
    TestCompile => { name: "test-compile", order: 112 },
    ProcessTestClasses => { name: "process-test-classes", order: 113 },
    Test => { name: "test", order: 114 },
    PreparePackage => { name: "prepare-package", order: 115 },
    Package => { name: "package", order: 116 },
    PreIntegrationTest => { name: "pre-integration-test", order: 117 },
    IntegrationTest => { name: "integration-test", order: 118 },
    PostIntegrationTest => { name: "post-integration-test", order: 119 },
    Verify => { name: "verify", order: 120 },
    Install => { name: "install", order: 121 },
    Deploy => { name: "deploy", order: 122 },

    // Site lifecycle
    PreSite => { name: "pre-site", order: 200 },
    Site => { name: "site", order: 201 },
    PostSite => { name: "post-site", order: 202 },
    SiteDeploy => { name: "site-deploy", order: 203 },
});

impl FromStr for LifecyclePhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pre-clean" => Ok(LifecyclePhase::PreClean),
            "clean" => Ok(LifecyclePhase::Clean),
            "post-clean" => Ok(LifecyclePhase::PostClean),
            "validate" => Ok(LifecyclePhase::Validate),
            "initialize" => Ok(LifecyclePhase::Initialize),
            "generate-sources" => Ok(LifecyclePhase::GenerateSources),
            "process-sources" => Ok(LifecyclePhase::ProcessSources),
            "generate-resources" => Ok(LifecyclePhase::GenerateResources),
            "process-resources" => Ok(LifecyclePhase::ProcessResources),
            "compile" => Ok(LifecyclePhase::Compile),
            "process-classes" => Ok(LifecyclePhase::ProcessClasses),
            "generate-test-sources" => Ok(LifecyclePhase::GenerateTestSources),
            "process-test-sources" => Ok(LifecyclePhase::ProcessTestSources),
            "generate-test-resources" => Ok(LifecyclePhase::GenerateTestResources),
            "process-test-resources" => Ok(LifecyclePhase::ProcessTestResources),
            "test-compile" => Ok(LifecyclePhase::TestCompile),
            "process-test-classes" => Ok(LifecyclePhase::ProcessTestClasses),
            "test" => Ok(LifecyclePhase::Test),
            "prepare-package" => Ok(LifecyclePhase::PreparePackage),
            "package" => Ok(LifecyclePhase::Package),
            "pre-integration-test" => Ok(LifecyclePhase::PreIntegrationTest),
            "integration-test" => Ok(LifecyclePhase::IntegrationTest),
            "post-integration-test" => Ok(LifecyclePhase::PostIntegrationTest),
            "verify" => Ok(LifecyclePhase::Verify),
            "install" => Ok(LifecyclePhase::Install),
            "deploy" => Ok(LifecyclePhase::Deploy),
            "pre-site" => Ok(LifecyclePhase::PreSite),
            "site" => Ok(LifecyclePhase::Site),
            "post-site" => Ok(LifecyclePhase::PostSite),
            "site-deploy" => Ok(LifecyclePhase::SiteDeploy),
            _ => Err(format!("Invalid lifecycle phase: {}", s)),
        }
    }
}

impl LifecyclePhase {
    /// Returns all phases up to and including this phase
    pub fn phases_up_to(&self) -> Vec<LifecyclePhase> {
        let target_order = self.order();

        // Determine which lifecycle this phase belongs to
        let lifecycle_phases = if target_order < 100 {
            // Clean lifecycle
            vec![
                LifecyclePhase::PreClean,
                LifecyclePhase::Clean,
                LifecyclePhase::PostClean,
            ]
        } else if target_order < 200 {
            // Default lifecycle
            vec![
                LifecyclePhase::Validate,
                LifecyclePhase::Initialize,
                LifecyclePhase::GenerateSources,
                LifecyclePhase::ProcessSources,
                LifecyclePhase::GenerateResources,
                LifecyclePhase::ProcessResources,
                LifecyclePhase::Compile,
                LifecyclePhase::ProcessClasses,
                LifecyclePhase::GenerateTestSources,
                LifecyclePhase::ProcessTestSources,
                LifecyclePhase::GenerateTestResources,
                LifecyclePhase::ProcessTestResources,
                LifecyclePhase::TestCompile,
                LifecyclePhase::ProcessTestClasses,
                LifecyclePhase::Test,
                LifecyclePhase::PreparePackage,
                LifecyclePhase::Package,
                LifecyclePhase::PreIntegrationTest,
                LifecyclePhase::IntegrationTest,
                LifecyclePhase::PostIntegrationTest,
                LifecyclePhase::Verify,
                LifecyclePhase::Install,
                LifecyclePhase::Deploy,
            ]
        } else {
            // Site lifecycle
            vec![
                LifecyclePhase::PreSite,
                LifecyclePhase::Site,
                LifecyclePhase::PostSite,
                LifecyclePhase::SiteDeploy,
            ]
        };

        lifecycle_phases
            .into_iter()
            .filter(|phase| phase.order() <= target_order)
            .collect()
    }
}

impl PartialOrd for LifecyclePhase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LifecyclePhase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order().cmp(&other.order())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_phase_from_str() {
        assert_eq!(
            "compile".parse::<LifecyclePhase>(),
            Ok(LifecyclePhase::Compile)
        );
        assert_eq!("test".parse::<LifecyclePhase>(), Ok(LifecyclePhase::Test));
        assert_eq!(
            "package".parse::<LifecyclePhase>(),
            Ok(LifecyclePhase::Package)
        );
        assert!("invalid".parse::<LifecyclePhase>().is_err());
    }

    #[test]
    fn test_lifecycle_phase_order() {
        assert!(LifecyclePhase::Compile < LifecyclePhase::Test);
        assert!(LifecyclePhase::Test < LifecyclePhase::Package);
        assert!(LifecyclePhase::Package < LifecyclePhase::Install);
    }

    #[test]
    fn test_lifecycle_phase_phases_up_to() {
        let phases = LifecyclePhase::Test.phases_up_to();
        assert!(phases.contains(&LifecyclePhase::Compile));
        assert!(phases.contains(&LifecyclePhase::TestCompile));
        assert!(phases.contains(&LifecyclePhase::Test));
        assert!(!phases.contains(&LifecyclePhase::Package));
    }

    #[test]
    fn test_lifecycle_phase_display() {
        assert_eq!(LifecyclePhase::Compile.to_string(), "compile");
        assert_eq!(LifecyclePhase::TestCompile.to_string(), "test-compile");
    }
}
