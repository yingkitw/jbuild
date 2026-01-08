use std::collections::HashMap;

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
        ]
    }

    /// Get clean lifecycle phases
    pub fn clean_phases() -> Vec<LifecyclePhase> {
        vec![LifecyclePhase::Clean]
    }

    /// Get the order/index of this phase in the lifecycle
    pub fn order(&self) -> usize {
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
            LifecyclePhase::Clean => 100, // Clean is separate lifecycle
        }
    }

    /// Get all phases up to and including the target phase
    pub fn phases_up_to(target: &LifecyclePhase) -> Vec<LifecyclePhase> {
        let target_order = target.order();
        Self::all().into_iter()
            .filter(|p| p.order() <= target_order)
            .collect()
    }

    /// Parse a lifecycle phase from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
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
            "clean" => Some(LifecyclePhase::Clean),
            _ => None,
        }
    }
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
        };
        write!(f, "{s}")
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

#[derive(Debug, Clone)]
pub struct PluginBinding {
    pub group_id: String,
    pub artifact_id: String,
    pub goal: String,
}

impl Lifecycle {
    /// Get the default Maven lifecycle
    pub fn default() -> Self {
        Self {
            phases: LifecyclePhase::all(),
            bindings: HashMap::new(),
        }
    }
}

