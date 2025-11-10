use std::collections::{HashMap, HashSet};
use anyhow::Result;
use crate::artifact::Artifact;
use crate::model::Dependency;
use crate::resolver::resolver::DependencyResolver;

/// Version range parser and resolver
pub struct VersionRangeResolver;

impl VersionRangeResolver {
    /// Parse and resolve a version range
    pub fn resolve_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        // Maven version ranges: [1.0,2.0), (1.0,2.0], [1.0,2.0], (1.0,2.0)
        // Also supports: [1.0,), (,2.0], [1.0]
        
        if range.starts_with('[') || range.starts_with('(') {
            Self::parse_interval_range(range, available_versions)
        } else if range.contains(',') {
            // Multiple ranges: [1.0,2.0),[2.0,3.0)
            Self::resolve_multi_range(range, available_versions)
        } else {
            // Single version or simple range
            Ok(available_versions.last().cloned())
        }
    }

    fn parse_interval_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        // Parse [1.0,2.0) format
        let trimmed = range.trim();
        let inclusive_lower = trimmed.starts_with('[');
        let inclusive_upper = trimmed.ends_with(']');
        
        let content = trimmed
            .trim_start_matches('[')
            .trim_start_matches('(')
            .trim_end_matches(']')
            .trim_end_matches(')');
        
        let parts: Vec<&str> = content.split(',').collect();
        
        if parts.len() == 2 {
            let lower = parts[0].trim();
            let upper = parts[1].trim();
            
            // Find highest version in range
            let mut best_version: Option<String> = None;
            
            for version in available_versions {
                let in_lower = if lower.is_empty() {
                    true
                } else if inclusive_lower {
                    Self::compare_versions(version, lower) >= 0
                } else {
                    Self::compare_versions(version, lower) > 0
                };
                
                let in_upper = if upper.is_empty() {
                    true
                } else if inclusive_upper {
                    Self::compare_versions(version, upper) <= 0
                } else {
                    Self::compare_versions(version, upper) < 0
                };
                
                if in_lower && in_upper {
                    if best_version.is_none() || Self::compare_versions(version, &best_version.as_ref().unwrap()) > 0 {
                        best_version = Some(version.clone());
                    }
                }
            }
            
            Ok(best_version)
        } else if parts.len() == 1 && !parts[0].is_empty() {
            // Single version: [1.0]
            let version = parts[0].trim();
            if available_versions.contains(&version.to_string()) {
                Ok(Some(version.to_string()))
            } else {
                Ok(None)
            }
        } else {
            // Invalid range
            Ok(None)
        }
    }

    fn resolve_multi_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        // Split by comma and try each range
        let ranges: Vec<&str> = range.split(',').collect();
        let mut best_version: Option<String> = None;
        
        for sub_range in ranges {
            if let Ok(Some(version)) = Self::parse_interval_range(sub_range.trim(), available_versions) {
                if best_version.is_none() || Self::compare_versions(&version, &best_version.as_ref().unwrap()) > 0 {
                    best_version = Some(version);
                }
            }
        }
        
        Ok(best_version)
    }

    /// Compare two version strings (simplified)
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        // Simple version comparison - in production, use proper semantic versioning
        // This is a simplified implementation
        let v1_parts: Vec<&str> = v1.split('.').collect();
        let v2_parts: Vec<&str> = v2.split('.').collect();
        
        let max_len = v1_parts.len().max(v2_parts.len());
        
        for i in 0..max_len {
            let v1_part = v1_parts.get(i).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let v2_part = v2_parts.get(i).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            
            if v1_part != v2_part {
                return v1_part - v2_part;
            }
        }
        
        0
    }
}

/// Dependency conflict resolver
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflicts using nearest-wins strategy (Maven's default)
    pub fn resolve_conflicts(
        dependencies: &[(String, Artifact)], // (dependency_key, artifact)
    ) -> Vec<Artifact> {
        let mut resolved = HashMap::new();
        
        // Nearest wins: later dependencies override earlier ones
        for (key, artifact) in dependencies {
            resolved.insert(key.clone(), artifact.clone());
        }
        
        resolved.into_values().collect()
    }

    /// Resolve conflicts using highest-version-wins strategy
    pub fn resolve_by_highest_version(
        dependencies: &[(String, Artifact)],
    ) -> Vec<Artifact> {
        let mut grouped: HashMap<String, Vec<Artifact>> = HashMap::new();
        
        // Group by groupId:artifactId
        for (_, artifact) in dependencies {
            let key = format!("{}:{}", 
                artifact.coordinates.group_id, 
                artifact.coordinates.artifact_id
            );
            grouped.entry(key).or_insert_with(Vec::new).push(artifact.clone());
        }
        
        // For each group, pick highest version
        let mut resolved = Vec::new();
        for (_, artifacts) in grouped {
            if let Some(best) = artifacts.iter().max_by_key(|a| &a.coordinates.version) {
                resolved.push(best.clone());
            }
        }
        
        resolved
    }
}

/// Dependency mediator for conflict resolution
pub struct DependencyMediator;

impl DependencyMediator {
    /// Mediate between conflicting dependency versions
    pub fn mediate(
        group_id: &str,
        artifact_id: &str,
        versions: &[String],
    ) -> Option<String> {
        if versions.is_empty() {
            return None;
        }
        
        // Use highest version as default strategy
        versions.iter().max_by_key(|v| Self::version_key(v)).cloned()
    }

    /// Create a sortable key for version comparison
    fn version_key(version: &str) -> (i32, i32, i32, String) {
        // Parse version into major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();
        let major = parts.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let suffix = parts.get(3).map(|s| s.to_string()).unwrap_or_default();
        
        (major, minor, patch, suffix)
    }
}

/// Enhanced dependency resolver with advanced features
pub struct AdvancedDependencyResolver {
    base_resolver: DependencyResolver,
    exclusions: HashSet<String>, // groupId:artifactId patterns
}

impl AdvancedDependencyResolver {
    pub fn new(resolver: DependencyResolver) -> Self {
        Self {
            base_resolver: resolver,
            exclusions: HashSet::new(),
        }
    }

    /// Add an exclusion pattern
    pub fn add_exclusion(mut self, group_id: &str, artifact_id: &str) -> Self {
        self.exclusions.insert(format!("{}:{}", group_id, artifact_id));
        self
    }

    /// Check if a dependency is excluded
    pub fn is_excluded(&self, dependency: &Dependency) -> bool {
        let key = dependency.id();
        self.exclusions.contains(&key)
    }

    /// Resolve dependency with exclusions handling
    pub fn resolve_with_exclusions(
        &self,
        dependency: &Dependency,
    ) -> Result<Option<Artifact>> {
        // Check if this dependency itself is excluded
        if self.is_excluded(dependency) {
            return Ok(None);
        }

        // Check exclusions from dependency itself
        if let Some(ref exclusions) = dependency.exclusions {
            for exclusion in exclusions {
                let exclusion_key = format!("{}:{}", exclusion.group_id, exclusion.artifact_id);
                if self.exclusions.contains(&exclusion_key) {
                    // This transitive dependency would be excluded
                    // For now, we still resolve the direct dependency
                }
            }
        }

        // Handle optional dependencies
        if dependency.optional == Some(true) {
            // Optional dependencies are not required
            // Resolve if available, but don't fail if missing
            return self.base_resolver.resolve_dependency(dependency);
        }

        // Resolve version range if present
        if let Some(ref version) = dependency.version {
            if version.contains('[') || version.contains('(') || version.contains(',') {
                // Version range - would need to fetch available versions
                // For now, treat as regular version
                return self.base_resolver.resolve_dependency(dependency);
            }
        }

        self.base_resolver.resolve_dependency(dependency)
    }

    /// Resolve dependencies with conflict resolution
    pub fn resolve_with_conflict_resolution(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Artifact>> {
        let mut dependency_map: HashMap<String, (String, Artifact)> = HashMap::new();

        for dependency in dependencies {
            if let Some(artifact) = self.resolve_with_exclusions(dependency)? {
                let key = format!("{}:{}", 
                    artifact.coordinates.group_id, 
                    artifact.coordinates.artifact_id
                );
                dependency_map.insert(key, (dependency.full_id(), artifact));
            }
        }

        // Resolve conflicts
        let conflicts: Vec<(String, Artifact)> = dependency_map
            .into_iter()
            .map(|(_, (_, artifact))| (artifact.coordinates.full_id(), artifact))
            .collect();

        let resolved_artifacts = ConflictResolver::resolve_conflicts(&conflicts);

        Ok(resolved_artifacts)
    }
}

