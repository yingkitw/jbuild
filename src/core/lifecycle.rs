use std::collections::HashMap;
use std::str::FromStr;

/// Maven lifecycle phase
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
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
    Clean,
}

impl LifecyclePhase {
    /// Get all lifecycle phases in order
    pub fn all() -> Vec<LifecyclePhase> {
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
            LifecyclePhase::Clean,
        ]
    }

    /// Get phase order for sorting
    pub fn order(&self) -> u32 {
        match self {
            LifecyclePhase::Validate => 0,
            LifecyclePhase::Initialize => 1,
            LifecyclePhase::GenerateSources => 2,
            LifecyclePhase::ProcessSources => 3,
            LifecyclePhase::GenerateResources => 4,
            LifecyclePhase::ProcessResources => 5,
            LifecyclePhase::Compile => 6,
            LifecyclePhase::ProcessClasses => 7,
            LifecyclePhase::GenerateTestSources => 8,
            LifecyclePhase::ProcessTestSources => 9,
            LifecyclePhase::GenerateTestResources => 10,
            LifecyclePhase::ProcessTestResources => 11,
            LifecyclePhase::TestCompile => 12,
            LifecyclePhase::ProcessTestClasses => 13,
            LifecyclePhase::Test => 14,
            LifecyclePhase::PreparePackage => 15,
            LifecyclePhase::Package => 16,
            LifecyclePhase::PreIntegrationTest => 17,
            LifecyclePhase::IntegrationTest => 18,
            LifecyclePhase::PostIntegrationTest => 19,
            LifecyclePhase::Verify => 20,
            LifecyclePhase::Install => 21,
            LifecyclePhase::Deploy => 22,
            LifecyclePhase::Clean => 100,
        }
    }

    /// Returns all phases up to this phase
    pub fn phases_up_to(&self) -> Vec<LifecyclePhase> {
        let target_order = self.order();
        Self::all()
            .into_iter()
            .filter(|p| p.order() <= target_order)
            .collect()
    }

    /// Get clean lifecycle phases
    pub fn clean_phases() -> Vec<LifecyclePhase> {
        vec![LifecyclePhase::Clean]
    }
}

impl FromStr for LifecyclePhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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
            "clean" => Ok(LifecyclePhase::Clean),
            _ => Err(format!("Invalid lifecycle phase: {}", s)),
        }
    }
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl LifecyclePhase {
    /// Returns the phase name as a string
    pub fn as_str(&self) -> &str {
        match self {
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
            LifecyclePhase::Clean => "clean",
        }
    }
}

/// Maven lifecycle
#[derive(Debug, Clone)]
pub struct Lifecycle {
    /// Lifecycle phases in order
    pub phases: Vec<LifecyclePhase>,

    /// Plugin bindings for each phase
    pub bindings: HashMap<LifecyclePhase, Vec<PluginBinding>>,
}

/// Plugin binding for lifecycle phase
#[derive(Debug, Clone)]
pub struct PluginBinding {
    pub group_id: String,
    pub artifact_id: String,
    pub goal: String,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            phases: LifecyclePhase::all(),
            bindings: HashMap::new(),
        }
    }
}
