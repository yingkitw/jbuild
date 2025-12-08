//! Configuration types for Checkstyle-rs

use crate::checkstyle::api::error::CheckstyleResult;

/// Configuration interface for modules
pub trait Configurable: Send + Sync {
    /// Configure the module with the given configuration
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()>;
}

/// Configuration for a module
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Name of the module
    pub name: String,
    /// Properties/attributes of the module
    pub properties: std::collections::HashMap<String, String>,
    /// Child configurations
    pub children: Vec<Configuration>,
}

impl Configuration {
    /// Create a new configuration
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: std::collections::HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Add a property
    pub fn add_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// Get a property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Add a child configuration
    pub fn add_child(&mut self, child: Configuration) {
        self.children.push(child);
    }

    /// Get child configurations
    pub fn get_children(&self) -> &[Configuration] {
        &self.children
    }
}

/// Context for modules
pub trait Contextualizable: Send + Sync {
    /// Contextualize the module with the given context
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()>;
}

/// Context for module configuration
#[derive(Debug, Clone)]
pub struct Context {
    /// Severity level
    pub severity: crate::checkstyle::api::event::SeverityLevel,
    /// Tab width
    pub tab_width: usize,
    /// Other context data
    pub data: std::collections::HashMap<String, String>,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            severity: crate::checkstyle::api::event::SeverityLevel::Error,
            tab_width: 4,
            data: std::collections::HashMap::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
