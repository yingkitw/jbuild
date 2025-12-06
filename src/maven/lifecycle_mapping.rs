//! Maven Lifecycle Mapping
//!
//! Implements Maven's lifecycle mapping for different packaging types.
//! Based on Maven's LifecycleMappingDelegate and DefaultLifecycleMapping.

use std::collections::HashMap;
use crate::core::lifecycle::LifecyclePhase;

/// Plugin binding for a lifecycle phase
#[derive(Debug, Clone)]
pub struct PluginBinding {
    /// Plugin group ID
    pub group_id: String,
    /// Plugin artifact ID
    pub artifact_id: String,
    /// Plugin version
    pub version: String,
    /// Goal to execute
    pub goal: String,
}

impl PluginBinding {
    pub fn new(
        group_id: impl Into<String>,
        artifact_id: impl Into<String>,
        version: impl Into<String>,
        goal: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
            version: version.into(),
            goal: goal.into(),
        }
    }

    /// Create a binding for maven-compiler-plugin
    pub fn compiler(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-compiler-plugin", "3.11.0", goal)
    }

    /// Create a binding for maven-resources-plugin
    pub fn resources(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-resources-plugin", "3.3.1", goal)
    }

    /// Create a binding for maven-surefire-plugin
    pub fn surefire(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-surefire-plugin", "3.1.2", goal)
    }

    /// Create a binding for maven-jar-plugin
    pub fn jar(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-jar-plugin", "3.3.0", goal)
    }

    /// Create a binding for maven-war-plugin
    pub fn war(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-war-plugin", "3.4.0", goal)
    }

    /// Create a binding for maven-install-plugin
    pub fn install(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-install-plugin", "3.1.1", goal)
    }

    /// Create a binding for maven-deploy-plugin
    pub fn deploy(goal: &str) -> Self {
        Self::new("org.apache.maven.plugins", "maven-deploy-plugin", "3.1.1", goal)
    }

    /// Get the plugin key
    pub fn plugin_key(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }
}

/// Lifecycle mapping for a packaging type
#[derive(Debug, Clone, Default)]
pub struct LifecycleMapping {
    /// Packaging type (e.g., "jar", "war", "pom")
    pub packaging: String,
    /// Phase to plugin bindings
    bindings: HashMap<LifecyclePhase, Vec<PluginBinding>>,
}

impl LifecycleMapping {
    pub fn new(packaging: impl Into<String>) -> Self {
        Self {
            packaging: packaging.into(),
            bindings: HashMap::new(),
        }
    }

    /// Add a binding for a phase
    pub fn bind(&mut self, phase: LifecyclePhase, binding: PluginBinding) {
        self.bindings.entry(phase).or_default().push(binding);
    }

    /// Get bindings for a phase
    pub fn get_bindings(&self, phase: &LifecyclePhase) -> &[PluginBinding] {
        self.bindings.get(phase).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all phases with bindings
    pub fn phases(&self) -> Vec<LifecyclePhase> {
        let mut phases: Vec<_> = self.bindings.keys().cloned().collect();
        phases.sort_by_key(|p| p.order());
        phases
    }

    /// Create JAR packaging lifecycle mapping
    pub fn jar() -> Self {
        let mut mapping = Self::new("jar");

        mapping.bind(LifecyclePhase::ProcessResources, PluginBinding::resources("resources"));
        mapping.bind(LifecyclePhase::Compile, PluginBinding::compiler("compile"));
        mapping.bind(LifecyclePhase::ProcessTestResources, PluginBinding::resources("testResources"));
        mapping.bind(LifecyclePhase::TestCompile, PluginBinding::compiler("testCompile"));
        mapping.bind(LifecyclePhase::Test, PluginBinding::surefire("test"));
        mapping.bind(LifecyclePhase::Package, PluginBinding::jar("jar"));
        mapping.bind(LifecyclePhase::Install, PluginBinding::install("install"));
        mapping.bind(LifecyclePhase::Deploy, PluginBinding::deploy("deploy"));

        mapping
    }

    /// Create WAR packaging lifecycle mapping
    pub fn war() -> Self {
        let mut mapping = Self::new("war");

        mapping.bind(LifecyclePhase::ProcessResources, PluginBinding::resources("resources"));
        mapping.bind(LifecyclePhase::Compile, PluginBinding::compiler("compile"));
        mapping.bind(LifecyclePhase::ProcessTestResources, PluginBinding::resources("testResources"));
        mapping.bind(LifecyclePhase::TestCompile, PluginBinding::compiler("testCompile"));
        mapping.bind(LifecyclePhase::Test, PluginBinding::surefire("test"));
        mapping.bind(LifecyclePhase::Package, PluginBinding::war("war"));
        mapping.bind(LifecyclePhase::Install, PluginBinding::install("install"));
        mapping.bind(LifecyclePhase::Deploy, PluginBinding::deploy("deploy"));

        mapping
    }

    /// Create POM packaging lifecycle mapping (no bindings except install/deploy)
    pub fn pom() -> Self {
        let mut mapping = Self::new("pom");

        mapping.bind(LifecyclePhase::Install, PluginBinding::install("install"));
        mapping.bind(LifecyclePhase::Deploy, PluginBinding::deploy("deploy"));

        mapping
    }

    /// Create EAR packaging lifecycle mapping
    pub fn ear() -> Self {
        let mut mapping = Self::new("ear");

        mapping.bind(LifecyclePhase::GenerateResources, 
            PluginBinding::new("org.apache.maven.plugins", "maven-ear-plugin", "3.3.0", "generate-application-xml"));
        mapping.bind(LifecyclePhase::ProcessResources, PluginBinding::resources("resources"));
        mapping.bind(LifecyclePhase::Package, 
            PluginBinding::new("org.apache.maven.plugins", "maven-ear-plugin", "3.3.0", "ear"));
        mapping.bind(LifecyclePhase::Install, PluginBinding::install("install"));
        mapping.bind(LifecyclePhase::Deploy, PluginBinding::deploy("deploy"));

        mapping
    }

    /// Create EJB packaging lifecycle mapping
    pub fn ejb() -> Self {
        let mut mapping = Self::new("ejb");

        mapping.bind(LifecyclePhase::ProcessResources, PluginBinding::resources("resources"));
        mapping.bind(LifecyclePhase::Compile, PluginBinding::compiler("compile"));
        mapping.bind(LifecyclePhase::ProcessTestResources, PluginBinding::resources("testResources"));
        mapping.bind(LifecyclePhase::TestCompile, PluginBinding::compiler("testCompile"));
        mapping.bind(LifecyclePhase::Test, PluginBinding::surefire("test"));
        mapping.bind(LifecyclePhase::Package, 
            PluginBinding::new("org.apache.maven.plugins", "maven-ejb-plugin", "3.2.1", "ejb"));
        mapping.bind(LifecyclePhase::Install, PluginBinding::install("install"));
        mapping.bind(LifecyclePhase::Deploy, PluginBinding::deploy("deploy"));

        mapping
    }
}

/// Registry of lifecycle mappings for different packaging types
#[derive(Debug, Default)]
pub struct LifecycleMappingRegistry {
    mappings: HashMap<String, LifecycleMapping>,
}

impl LifecycleMappingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default mappings
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(LifecycleMapping::jar());
        registry.register(LifecycleMapping::war());
        registry.register(LifecycleMapping::pom());
        registry.register(LifecycleMapping::ear());
        registry.register(LifecycleMapping::ejb());
        registry
    }

    /// Register a lifecycle mapping
    pub fn register(&mut self, mapping: LifecycleMapping) {
        self.mappings.insert(mapping.packaging.clone(), mapping);
    }

    /// Get mapping for a packaging type
    pub fn get(&self, packaging: &str) -> Option<&LifecycleMapping> {
        self.mappings.get(packaging)
    }

    /// Get mapping or default to JAR
    pub fn get_or_default(&self, packaging: &str) -> &LifecycleMapping {
        self.mappings.get(packaging)
            .or_else(|| self.mappings.get("jar"))
            .expect("JAR mapping should always exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_binding() {
        let binding = PluginBinding::compiler("compile");
        assert_eq!(binding.artifact_id, "maven-compiler-plugin");
        assert_eq!(binding.goal, "compile");
    }

    #[test]
    fn test_jar_lifecycle_mapping() {
        let mapping = LifecycleMapping::jar();
        
        let compile_bindings = mapping.get_bindings(&LifecyclePhase::Compile);
        assert_eq!(compile_bindings.len(), 1);
        assert_eq!(compile_bindings[0].goal, "compile");

        let package_bindings = mapping.get_bindings(&LifecyclePhase::Package);
        assert_eq!(package_bindings.len(), 1);
        assert_eq!(package_bindings[0].goal, "jar");
    }

    #[test]
    fn test_war_lifecycle_mapping() {
        let mapping = LifecycleMapping::war();
        
        let package_bindings = mapping.get_bindings(&LifecyclePhase::Package);
        assert_eq!(package_bindings.len(), 1);
        assert_eq!(package_bindings[0].goal, "war");
    }

    #[test]
    fn test_pom_lifecycle_mapping() {
        let mapping = LifecycleMapping::pom();
        
        // POM should have no compile phase bindings
        let compile_bindings = mapping.get_bindings(&LifecyclePhase::Compile);
        assert!(compile_bindings.is_empty());

        // But should have install
        let install_bindings = mapping.get_bindings(&LifecyclePhase::Install);
        assert_eq!(install_bindings.len(), 1);
    }

    #[test]
    fn test_lifecycle_mapping_registry() {
        let registry = LifecycleMappingRegistry::with_defaults();
        
        assert!(registry.get("jar").is_some());
        assert!(registry.get("war").is_some());
        assert!(registry.get("pom").is_some());
        
        // Unknown packaging should fall back to jar
        let unknown = registry.get_or_default("unknown");
        assert_eq!(unknown.packaging, "jar");
    }

    #[test]
    fn test_phases_ordering() {
        let mapping = LifecycleMapping::jar();
        let phases = mapping.phases();
        
        // Phases should be in order
        let compile_pos = phases.iter().position(|p| *p == LifecyclePhase::Compile);
        let test_pos = phases.iter().position(|p| *p == LifecyclePhase::Test);
        let package_pos = phases.iter().position(|p| *p == LifecyclePhase::Package);
        
        assert!(compile_pos < test_pos);
        assert!(test_pos < package_pos);
    }
}
