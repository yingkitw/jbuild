//! Version Range Resolution
//!
//! Parses and resolves Maven version ranges.

use anyhow::Result;
use crate::common::version::compare_versions;

/// Version range parser and resolver
pub struct VersionRangeResolver;

impl VersionRangeResolver {
    /// Parse and resolve a version range
    pub fn resolve_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        if range.starts_with('[') || range.starts_with('(') {
            Self::parse_interval_range(range, available_versions)
        } else if range.contains(',') {
            Self::resolve_multi_range(range, available_versions)
        } else {
            Ok(available_versions.last().cloned())
        }
    }

    fn parse_interval_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        let trimmed = range.trim();
        let inclusive_lower = trimmed.starts_with('[');
        let inclusive_upper = trimmed.ends_with(']');
        
        let content = trimmed
            .trim_start_matches(['[', '('])
            .trim_end_matches([']', ')']);
        
        let parts: Vec<&str> = content.split(',').collect();
        
        if parts.len() == 2 {
            let lower = parts[0].trim();
            let upper = parts[1].trim();
            
            let mut best_version: Option<String> = None;
            
            for version in available_versions {
                let in_lower = lower.is_empty() || 
                    (if inclusive_lower { compare_versions(version, lower) >= 0 } 
                     else { compare_versions(version, lower) > 0 });
                
                let in_upper = upper.is_empty() || 
                    (if inclusive_upper { compare_versions(version, upper) <= 0 } 
                     else { compare_versions(version, upper) < 0 });
                
                if in_lower && in_upper
                    && (best_version.is_none() || compare_versions(version, best_version.as_ref().unwrap()) > 0) {
                        best_version = Some(version.clone());
                    }
            }
            
            Ok(best_version)
        } else if parts.len() == 1 && !parts[0].is_empty() {
            let version = parts[0].trim();
            Ok(available_versions.contains(&version.to_string()).then(|| version.to_string()))
        } else {
            Ok(None)
        }
    }

    fn resolve_multi_range(range: &str, available_versions: &[String]) -> Result<Option<String>> {
        let mut best_version: Option<String> = None;
        
        for sub_range in range.split(',') {
            if let Ok(Some(version)) = Self::parse_interval_range(sub_range.trim(), available_versions) {
                if best_version.is_none() || compare_versions(&version, best_version.as_ref().unwrap()) > 0 {
                    best_version = Some(version);
                }
            }
        }
        
        Ok(best_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_range_inclusive() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string(), "2.0.0".to_string()];
        let result = VersionRangeResolver::resolve_range("[1.0.0,2.0.0]", &versions).unwrap();
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_range_exclusive_upper() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string(), "2.0.0".to_string()];
        let result = VersionRangeResolver::resolve_range("[1.0.0,2.0.0)", &versions).unwrap();
        assert_eq!(result, Some("1.5.0".to_string()));
    }

    #[test]
    fn test_version_range_open_lower() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string(), "2.0.0".to_string()];
        let result = VersionRangeResolver::resolve_range("(,2.0.0]", &versions).unwrap();
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_range_open_upper() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string(), "2.0.0".to_string()];
        let result = VersionRangeResolver::resolve_range("[1.0.0,)", &versions).unwrap();
        assert_eq!(result, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_range_single() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string()];
        let result = VersionRangeResolver::resolve_range("[1.0.0]", &versions).unwrap();
        assert_eq!(result, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_version_range_not_found() {
        let versions = vec!["1.0.0".to_string(), "1.5.0".to_string()];
        let result = VersionRangeResolver::resolve_range("[2.0.0,3.0.0]", &versions).unwrap();
        assert_eq!(result, None);
    }
    // compare_versions tests are in common/version.rs
}
