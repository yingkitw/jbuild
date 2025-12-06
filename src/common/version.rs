//! Version Utilities
//!
//! Shared version comparison and parsing utilities.

/// Compare two version strings
/// Returns: negative if v1 < v2, positive if v1 > v2, 0 if equal
pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    let v1_parts: Vec<&str> = v1.split('.').collect();
    let v2_parts: Vec<&str> = v2.split('.').collect();
    
    let max_len = v1_parts.len().max(v2_parts.len());
    
    for i in 0..max_len {
        let v1_part = v1_parts.get(i).and_then(|s| parse_version_part(s)).unwrap_or(0);
        let v2_part = v2_parts.get(i).and_then(|s| parse_version_part(s)).unwrap_or(0);
        
        if v1_part != v2_part {
            return v1_part - v2_part;
        }
    }
    
    0
}

/// Parse a version part, handling qualifiers like "1-SNAPSHOT"
fn parse_version_part(s: &str) -> Option<i32> {
    // Handle qualifiers like "1-SNAPSHOT" -> extract "1"
    let numeric = s.split('-').next()?;
    numeric.parse().ok()
}

/// Create a sortable key for version comparison
pub fn version_key(version: &str) -> (i32, i32, i32, String) {
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.first().and_then(|s| parse_version_part(s)).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| parse_version_part(s)).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| parse_version_part(s)).unwrap_or(0);
    let suffix = parts.get(3).map(|s| s.to_string()).unwrap_or_default();
    
    (major, minor, patch, suffix)
}

/// Check if a version is a snapshot
pub fn is_snapshot(version: &str) -> bool {
    version.to_uppercase().contains("SNAPSHOT")
}

/// Get the base version (without snapshot qualifier)
pub fn base_version(version: &str) -> String {
    version.replace("-SNAPSHOT", "").replace("-snapshot", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert!(compare_versions("1.1.0", "1.0.0") > 0);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("2.0.0", "1.9.9") > 0);
    }

    #[test]
    fn test_compare_versions_different_lengths() {
        assert!(compare_versions("1.0", "1.0.1") < 0);
        assert_eq!(compare_versions("1.0", "1.0.0"), 0);
    }

    #[test]
    fn test_version_key() {
        let key1 = version_key("1.0.0");
        let key2 = version_key("2.0.0");
        assert!(key2 > key1);
    }

    #[test]
    fn test_is_snapshot() {
        assert!(is_snapshot("1.0.0-SNAPSHOT"));
        assert!(!is_snapshot("1.0.0"));
    }

    #[test]
    fn test_base_version() {
        assert_eq!(base_version("1.0.0-SNAPSHOT"), "1.0.0");
        assert_eq!(base_version("1.0.0"), "1.0.0");
    }
}
