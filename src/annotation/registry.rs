//! Registry for annotation processors

use std::collections::HashMap;
use std::path::PathBuf;
use super::processor::AnnotationProcessor;

/// Registry of available annotation processors
pub struct ProcessorRegistry {
    processors: HashMap<String, AnnotationProcessor>,
}

impl ProcessorRegistry {
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    /// Register a processor
    pub fn register(&mut self, processor: AnnotationProcessor) {
        self.processors.insert(processor.name.clone(), processor);
    }

    /// Get a processor by name
    pub fn get(&self, name: &str) -> Option<&AnnotationProcessor> {
        self.processors.get(name)
    }

    /// Get all processors
    pub fn all(&self) -> Vec<&AnnotationProcessor> {
        self.processors.values().collect()
    }

    /// Auto-discover processors from class path
    pub fn discover_from_classpath(&mut self, _classpath: &[PathBuf]) {
        // In a real implementation, this would scan JAR files for
        // META-INF/services/javax.annotation.processing.Processor files
        // For now, we add common processors
        for processor in super::processor::common::common_processors() {
            self.register(processor);
        }
    }

    /// Find processors that support given annotations
    pub fn find_for_annotations(&self, annotations: &[String]) -> Vec<&AnnotationProcessor> {
        self.processors
            .values()
            .filter(|p| {
                annotations
                    .iter()
                    .any(|a| p.supported_annotations.contains(a))
            })
            .collect()
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // Register common processors by default
        for processor in super::processor::common::common_processors() {
            registry.register(processor);
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_registry() {
        let mut registry = ProcessorRegistry::new();
        let processor = AnnotationProcessor::new("test.Processor");
        registry.register(processor);

        assert!(registry.get("test.Processor").is_some());
        assert_eq!(registry.all().len(), 1);
    }
}
