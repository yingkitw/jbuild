//! Value objects for Maven context

use serde::{Deserialize, Serialize};
use std::fmt;

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

impl LifecyclePhase {
    /// Returns the phase name as a string
    pub fn as_str(&self) -> &str {
        match self {
            // Clean lifecycle
            LifecyclePhase::PreClean => "pre-clean",
            LifecyclePhase::Clean => "clean",
            LifecyclePhase::PostClean => "post-clean",
            
            // Default lifecycle
            LifecyclePhase::Validate => "validate",
            LifecyclePhase::Initialize => "initialize",
            LifecyclePhase::GenerateSources => "generate-sources",
            LifecyclePhase::ProcessSources => "process-sources",
            LifecyclePhase::GenerateResources => "generate-resources",
            LifecyclePhase::ProcessResources => "process-resources",
            LifecyclePhase::Compile => "compile",
            LifecyclePhase::ProcessClasses => "process-classes",
            LifecyclePhase::GenerateTestSources => "generate-test-sources",
            LifecyclePhase::ProcessTestSources => "process-test-sources",
            LifecyclePhase::GenerateTestResources => "generate-test-resources",
            LifecyclePhase::ProcessTestResources => "process-test-resources",
            LifecyclePhase::TestCompile => "test-compile",
            LifecyclePhase::ProcessTestClasses => "process-test-classes",
            LifecyclePhase::Test => "test",
            LifecyclePhase::PreparePackage => "prepare-package",
            LifecyclePhase::Package => "package",
            LifecyclePhase::PreIntegrationTest => "pre-integration-test",
            LifecyclePhase::IntegrationTest => "integration-test",
            LifecyclePhase::PostIntegrationTest => "post-integration-test",
            LifecyclePhase::Verify => "verify",
            LifecyclePhase::Install => "install",
            LifecyclePhase::Deploy => "deploy",
            
            // Site lifecycle
            LifecyclePhase::PreSite => "pre-site",
            LifecyclePhase::Site => "site",
            LifecyclePhase::PostSite => "post-site",
            LifecyclePhase::SiteDeploy => "site-deploy",
        }
    }

    /// Parses a phase from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Clean lifecycle
            "pre-clean" => Some(LifecyclePhase::PreClean),
            "clean" => Some(LifecyclePhase::Clean),
            "post-clean" => Some(LifecyclePhase::PostClean),
            
            // Default lifecycle
            "validate" => Some(LifecyclePhase::Validate),
            "initialize" => Some(LifecyclePhase::Initialize),
            "generate-sources" => Some(LifecyclePhase::GenerateSources),
            "process-sources" => Some(LifecyclePhase::ProcessSources),
            "generate-resources" => Some(LifecyclePhase::GenerateResources),
            "process-resources" => Some(LifecyclePhase::ProcessResources),
            "compile" => Some(LifecyclePhase::Compile),
            "process-classes" => Some(LifecyclePhase::ProcessClasses),
            "generate-test-sources" => Some(LifecyclePhase::GenerateTestSources),
            "process-test-sources" => Some(LifecyclePhase::ProcessTestSources),
            "generate-test-resources" => Some(LifecyclePhase::GenerateTestResources),
            "process-test-resources" => Some(LifecyclePhase::ProcessTestResources),
            "test-compile" => Some(LifecyclePhase::TestCompile),
            "process-test-classes" => Some(LifecyclePhase::ProcessTestClasses),
            "test" => Some(LifecyclePhase::Test),
            "prepare-package" => Some(LifecyclePhase::PreparePackage),
            "package" => Some(LifecyclePhase::Package),
            "pre-integration-test" => Some(LifecyclePhase::PreIntegrationTest),
            "integration-test" => Some(LifecyclePhase::IntegrationTest),
            "post-integration-test" => Some(LifecyclePhase::PostIntegrationTest),
            "verify" => Some(LifecyclePhase::Verify),
            "install" => Some(LifecyclePhase::Install),
            "deploy" => Some(LifecyclePhase::Deploy),
            
            // Site lifecycle
            "pre-site" => Some(LifecyclePhase::PreSite),
            "site" => Some(LifecyclePhase::Site),
            "post-site" => Some(LifecyclePhase::PostSite),
            "site-deploy" => Some(LifecyclePhase::SiteDeploy),
            
            _ => None,
        }
    }

    /// Returns the order of this phase in the lifecycle
    pub fn order(&self) -> u32 {
        match self {
            // Clean lifecycle (0-99)
            LifecyclePhase::PreClean => 0,
            LifecyclePhase::Clean => 1,
            LifecyclePhase::PostClean => 2,
            
            // Default lifecycle (100-199)
            LifecyclePhase::Validate => 100,
            LifecyclePhase::Initialize => 101,
            LifecyclePhase::GenerateSources => 102,
            LifecyclePhase::ProcessSources => 103,
            LifecyclePhase::GenerateResources => 104,
            LifecyclePhase::ProcessResources => 105,
            LifecyclePhase::Compile => 106,
            LifecyclePhase::ProcessClasses => 107,
            LifecyclePhase::GenerateTestSources => 108,
            LifecyclePhase::ProcessTestSources => 109,
            LifecyclePhase::GenerateTestResources => 110,
            LifecyclePhase::ProcessTestResources => 111,
            LifecyclePhase::TestCompile => 112,
            LifecyclePhase::ProcessTestClasses => 113,
            LifecyclePhase::Test => 114,
            LifecyclePhase::PreparePackage => 115,
            LifecyclePhase::Package => 116,
            LifecyclePhase::PreIntegrationTest => 117,
            LifecyclePhase::IntegrationTest => 118,
            LifecyclePhase::PostIntegrationTest => 119,
            LifecyclePhase::Verify => 120,
            LifecyclePhase::Install => 121,
            LifecyclePhase::Deploy => 122,
            
            // Site lifecycle (200-299)
            LifecyclePhase::PreSite => 200,
            LifecyclePhase::Site => 201,
            LifecyclePhase::PostSite => 202,
            LifecyclePhase::SiteDeploy => 203,
        }
    }

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

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecyclePhase::Validate => write!(f, "validate"),
            LifecyclePhase::Compile => write!(f, "compile"),
            LifecyclePhase::Test => write!(f, "test"),
            LifecyclePhase::Package => write!(f, "package"),
            LifecyclePhase::Install => write!(f, "install"),
            LifecyclePhase::Deploy => write!(f, "deploy"),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}


impl PartialOrd for LifecyclePhase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.order().cmp(&other.order()))
    }
}

impl Ord for LifecyclePhase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_phase_from_str() {
        assert_eq!(LifecyclePhase::from_str("compile"), Some(LifecyclePhase::Compile));
        assert_eq!(LifecyclePhase::from_str("test"), Some(LifecyclePhase::Test));
        assert_eq!(LifecyclePhase::from_str("package"), Some(LifecyclePhase::Package));
        assert_eq!(LifecyclePhase::from_str("invalid"), None);
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
