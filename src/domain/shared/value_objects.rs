//! Shared value objects used across bounded contexts

use std::cmp::Ordering;
use std::fmt;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Version value object with semantic comparison
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Version(String);

impl Version {
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_snapshot(&self) -> bool {
        self.0.to_uppercase().contains("SNAPSHOT")
    }

    pub fn base_version(&self) -> &str {
        if let Some(idx) = self.0.find('-') {
            &self.0[..idx]
        } else {
            &self.0
        }
    }

    /// Parse version into numeric components for comparison
    fn parse_components(&self) -> Vec<VersionComponent> {
        let base = self.base_version();
        base.split('.')
            .map(|part| {
                if let Ok(num) = part.parse::<u64>() {
                    VersionComponent::Numeric(num)
                } else {
                    VersionComponent::String(part.to_string())
                }
            })
            .collect()
    }

    /// Compare versions semantically
    pub fn compare(&self, other: &Version) -> Ordering {
        let self_components = self.parse_components();
        let other_components = other.parse_components();

        // Compare component by component
        for i in 0..self_components.len().max(other_components.len()) {
            let self_comp = self_components.get(i).unwrap_or(&VersionComponent::Numeric(0));
            let other_comp = other_components.get(i).unwrap_or(&VersionComponent::Numeric(0));

            match self_comp.cmp(other_comp) {
                Ordering::Equal => continue,
                other => return other,
            }
        }

        // If base versions are equal, check for qualifiers
        let self_has_qualifier = self.0.contains('-');
        let other_has_qualifier = other.0.contains('-');
        
        match (self_has_qualifier, other_has_qualifier) {
            (true, false) => Ordering::Less,  // Qualified versions are less than release
            (false, true) => Ordering::Greater, // Release is greater than qualified
            _ => Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionComponent {
    Numeric(u64),
    String(String),
}

impl PartialOrd for VersionComponent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionComponent {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (VersionComponent::Numeric(a), VersionComponent::Numeric(b)) => a.cmp(b),
            (VersionComponent::String(a), VersionComponent::String(b)) => a.cmp(b),
            (VersionComponent::Numeric(_), VersionComponent::String(_)) => Ordering::Less,
            (VersionComponent::String(_), VersionComponent::Numeric(_)) => Ordering::Greater,
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Version {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// File path value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilePath(PathBuf);

impl FilePath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
}

impl From<PathBuf> for FilePath {
    fn from(p: PathBuf) -> Self {
        Self(p)
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        Self(PathBuf::from(s))
    }
}

/// Java version value object
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JavaVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl JavaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn from_string(version: &str) -> Option<Self> {
        let version = version.trim();
        
        // Handle old format: 1.8.0
        if version.starts_with("1.") {
            let parts: Vec<&str> = version.split('.').collect();
            if parts.len() >= 2 {
                let major = parts[1].parse().ok()?;
                let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
                return Some(Self::new(major, 0, patch));
            }
        }
        
        // Handle new format: 17.0.1
        let parts: Vec<&str> = version.split('.').collect();
        if !parts.is_empty() {
            let major = parts[0].parse().ok()?;
            let minor = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
            let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
            return Some(Self::new(major, minor, patch));
        }
        
        None
    }

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_snapshot() {
        let v = Version::new("1.0.0-SNAPSHOT");
        assert!(v.is_snapshot());
        assert_eq!(v.base_version(), "1.0.0");
    }

    #[test]
    fn test_version_comparison_numeric() {
        let v1 = Version::new("1.0.0");
        let v2 = Version::new("1.0.1");
        let v3 = Version::new("1.1.0");
        let v4 = Version::new("2.0.0");

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
        assert_eq!(v1, Version::new("1.0.0"));
    }

    #[test]
    fn test_version_comparison_snapshot() {
        let v1 = Version::new("1.0.0-SNAPSHOT");
        let v2 = Version::new("1.0.0");

        assert!(v1 < v2); // Snapshot is less than release
    }

    #[test]
    fn test_version_comparison_different_lengths() {
        let v1 = Version::new("1.0");
        let v2 = Version::new("1.0.0");
        let v3 = Version::new("1.0.1");

        assert_eq!(v1, v2); // 1.0 == 1.0.0
        assert!(v2 < v3);
    }

    #[test]
    fn test_version_comparison_mixed() {
        let v1 = Version::new("1.0.0");
        let v2 = Version::new("1.0.0-alpha");
        let v3 = Version::new("1.0.0-beta");
        let v4 = Version::new("1.0.0");

        assert!(v2 < v1); // alpha < release
        assert!(v3 < v1); // beta < release
        assert_eq!(v1, v4);
    }

    #[test]
    fn test_version_ordering() {
        let mut versions = [Version::new("2.0.0"),
            Version::new("1.0.0-SNAPSHOT"),
            Version::new("1.0.0"),
            Version::new("1.5.0"),
            Version::new("1.0.1")];

        versions.sort();

        assert_eq!(versions[0].as_str(), "1.0.0-SNAPSHOT");
        assert_eq!(versions[1].as_str(), "1.0.0");
        assert_eq!(versions[2].as_str(), "1.0.1");
        assert_eq!(versions[3].as_str(), "1.5.0");
        assert_eq!(versions[4].as_str(), "2.0.0");
    }

    #[test]
    fn test_java_version_parsing() {
        assert_eq!(JavaVersion::from_string("1.8.0"), Some(JavaVersion::new(8, 0, 0)));
        assert_eq!(JavaVersion::from_string("17.0.1"), Some(JavaVersion::new(17, 0, 1)));
        assert_eq!(JavaVersion::from_string("11"), Some(JavaVersion::new(11, 0, 0)));
    }

    #[test]
    fn test_java_version_comparison() {
        let v8 = JavaVersion::new(8, 0, 0);
        let v11 = JavaVersion::new(11, 0, 0);
        let v17 = JavaVersion::new(17, 0, 1);
        
        assert!(v8 < v11);
        assert!(v11 < v17);
    }

    #[test]
    fn test_file_path() {
        let path = FilePath::new("/tmp/test.txt");
        assert_eq!(path.as_path().to_str().unwrap(), "/tmp/test.txt");
    }
}
