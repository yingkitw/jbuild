//! Java Toolchain Support
//!
//! Implements Gradle's Java toolchain feature for managing JDK versions.

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};

/// Java toolchain specification
#[derive(Debug, Clone)]
pub struct JavaToolchain {
    /// Java language version (e.g., 11, 17, 21, 24)
    pub language_version: u32,
    /// Vendor (optional)
    pub vendor: Option<JavaVendor>,
    /// Implementation (optional)
    pub implementation: Option<JvmImplementation>,
}

/// Java vendor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JavaVendor {
    AdoptOpenJdk,
    Amazon,
    Azul,
    BellSoft,
    Graal,
    Ibm,
    Microsoft,
    Oracle,
    Sap,
    Other(String),
}

/// JVM implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JvmImplementation {
    VendorSpecific,
    J9,
    Hotspot,
}

impl JavaToolchain {
    pub fn new(language_version: u32) -> Self {
        Self {
            language_version,
            vendor: None,
            implementation: None,
        }
    }

    pub fn with_vendor(mut self, vendor: JavaVendor) -> Self {
        self.vendor = Some(vendor);
        self
    }

    pub fn with_implementation(mut self, implementation: JvmImplementation) -> Self {
        self.implementation = Some(implementation);
        self
    }
}

/// Resolved Java installation
#[derive(Debug, Clone)]
pub struct JavaInstallation {
    /// Path to Java home
    pub java_home: PathBuf,
    /// Java version
    pub version: String,
    /// Major version number
    pub major_version: u32,
    /// Vendor name
    pub vendor: Option<String>,
    /// Path to java executable
    pub java_executable: PathBuf,
    /// Path to javac executable
    pub javac_executable: PathBuf,
}

impl JavaInstallation {
    /// Detect the current Java installation from JAVA_HOME or PATH
    pub fn detect() -> Result<Self> {
        // Try JAVA_HOME first
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let java_home = PathBuf::from(java_home);
            if java_home.exists() {
                return Self::from_java_home(&java_home);
            }
        }

        // Try to find java in PATH
        if let Ok(output) = Command::new("java").arg("-version").output() {
            let version_output = String::from_utf8_lossy(&output.stderr);
            if let Some(installation) = Self::parse_from_path(&version_output) {
                return Ok(installation);
            }
        }

        Err(anyhow!("No Java installation found. Set JAVA_HOME or add java to PATH."))
    }

    /// Create from JAVA_HOME path
    pub fn from_java_home(java_home: &PathBuf) -> Result<Self> {
        let java_executable = if cfg!(windows) {
            java_home.join("bin/java.exe")
        } else {
            java_home.join("bin/java")
        };

        let javac_executable = if cfg!(windows) {
            java_home.join("bin/javac.exe")
        } else {
            java_home.join("bin/javac")
        };

        if !java_executable.exists() {
            return Err(anyhow!("Java executable not found at {java_executable:?}"));
        }

        // Get version info
        let output = Command::new(&java_executable)
            .arg("-version")
            .output()
            .map_err(|e| anyhow!("Failed to run java -version: {e}"))?;

        let version_output = String::from_utf8_lossy(&output.stderr);
        let (version, major_version, vendor) = Self::parse_version_output(&version_output)?;

        Ok(Self {
            java_home: java_home.clone(),
            version,
            major_version,
            vendor,
            java_executable,
            javac_executable,
        })
    }

    /// Parse version from java -version output
    fn parse_version_output(output: &str) -> Result<(String, u32, Option<String>)> {
        let mut version = String::new();
        let mut major_version = 0u32;
        let mut vendor = None;

        for line in output.lines() {
            // Parse version line like: openjdk version "17.0.1" or java version "1.8.0_301"
            if line.contains("version") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start+1..].find('"') {
                        version = line[start+1..start+1+end].to_string();
                        
                        // Parse major version
                        let version_parts: Vec<&str> = version.split('.').collect();
                        if !version_parts.is_empty() {
                            let first = version_parts[0];
                            if first == "1" && version_parts.len() > 1 {
                                // Old format: 1.8.0 -> major is 8
                                major_version = version_parts[1].parse().unwrap_or(0);
                            } else {
                                // New format: 17.0.1 -> major is 17
                                major_version = first.parse().unwrap_or(0);
                            }
                        }
                    }
                }

                // Detect vendor
                let line_lower = line.to_lowercase();
                if line_lower.contains("openjdk") {
                    vendor = Some("OpenJDK".to_string());
                } else if line_lower.contains("oracle") {
                    vendor = Some("Oracle".to_string());
                } else if line_lower.contains("azul") || line_lower.contains("zulu") {
                    vendor = Some("Azul".to_string());
                } else if line_lower.contains("amazon") || line_lower.contains("corretto") {
                    vendor = Some("Amazon".to_string());
                } else if line_lower.contains("ibm") || line_lower.contains("semeru") {
                    vendor = Some("IBM".to_string());
                } else if line_lower.contains("microsoft") {
                    vendor = Some("Microsoft".to_string());
                } else if line_lower.contains("graal") {
                    vendor = Some("GraalVM".to_string());
                }
            }
        }

        if version.is_empty() {
            return Err(anyhow!("Could not parse Java version from output"));
        }

        Ok((version, major_version, vendor))
    }

    /// Parse from PATH-based java
    fn parse_from_path(version_output: &str) -> Option<Self> {
        let (version, major_version, vendor) = Self::parse_version_output(version_output).ok()?;

        // Find java executable in PATH
        let java_executable = which::which("java").ok()?;
        let javac_executable = which::which("javac").ok()?;

        // Derive JAVA_HOME from executable path
        let java_home = java_executable.parent()?.parent()?.to_path_buf();

        Some(Self {
            java_home,
            version,
            major_version,
            vendor,
            java_executable,
            javac_executable,
        })
    }

    /// Check if this installation matches a toolchain spec
    pub fn matches(&self, toolchain: &JavaToolchain) -> bool {
        // Check version
        if self.major_version != toolchain.language_version {
            return false;
        }

        // Check vendor if specified
        if let Some(ref required_vendor) = toolchain.vendor {
            if let Some(ref actual_vendor) = self.vendor {
                let matches = match required_vendor {
                    JavaVendor::Amazon => actual_vendor.to_lowercase().contains("amazon") || 
                                         actual_vendor.to_lowercase().contains("corretto"),
                    JavaVendor::Azul => actual_vendor.to_lowercase().contains("azul") ||
                                       actual_vendor.to_lowercase().contains("zulu"),
                    JavaVendor::Oracle => actual_vendor.to_lowercase().contains("oracle"),
                    JavaVendor::Ibm => actual_vendor.to_lowercase().contains("ibm") ||
                                      actual_vendor.to_lowercase().contains("semeru"),
                    JavaVendor::Microsoft => actual_vendor.to_lowercase().contains("microsoft"),
                    JavaVendor::Graal => actual_vendor.to_lowercase().contains("graal"),
                    JavaVendor::Other(name) => actual_vendor.to_lowercase().contains(&name.to_lowercase()),
                    _ => true,
                };
                if !matches {
                    return false;
                }
            }
        }

        true
    }
}

/// Toolchain resolver - finds Java installations matching requirements
#[derive(Debug, Default)]
pub struct ToolchainResolver {
    /// Known Java installations
    installations: Vec<JavaInstallation>,
}

impl ToolchainResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover Java installations on the system
    pub fn discover(&mut self) -> Result<()> {
        // Try current JAVA_HOME
        if let Ok(installation) = JavaInstallation::detect() {
            self.installations.push(installation);
        }

        // Could add more discovery logic here:
        // - Check common installation directories
        // - Parse SDKMAN installations
        // - Check Homebrew installations on macOS
        // - etc.

        Ok(())
    }

    /// Find an installation matching the toolchain spec
    pub fn find(&self, toolchain: &JavaToolchain) -> Option<&JavaInstallation> {
        self.installations.iter().find(|i| i.matches(toolchain))
    }

    /// Get all discovered installations
    pub fn all(&self) -> &[JavaInstallation] {
        &self.installations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_creation() {
        let toolchain = JavaToolchain::new(17)
            .with_vendor(JavaVendor::Amazon);

        assert_eq!(toolchain.language_version, 17);
        assert_eq!(toolchain.vendor, Some(JavaVendor::Amazon));
    }

    #[test]
    fn test_parse_version_output_new_format() {
        let output = r#"openjdk version "17.0.1" 2021-10-19
OpenJDK Runtime Environment (build 17.0.1+12-39)
OpenJDK 64-Bit Server VM (build 17.0.1+12-39, mixed mode, sharing)"#;

        let (version, major, vendor) = JavaInstallation::parse_version_output(output).unwrap();
        assert_eq!(version, "17.0.1");
        assert_eq!(major, 17);
        assert_eq!(vendor, Some("OpenJDK".to_string()));
    }

    #[test]
    fn test_parse_version_output_old_format() {
        let output = r#"java version "1.8.0_301"
Java(TM) SE Runtime Environment (build 1.8.0_301-b09)
Java HotSpot(TM) 64-Bit Server VM (build 25.301-b09, mixed mode)"#;

        let (version, major, vendor) = JavaInstallation::parse_version_output(output).unwrap();
        assert_eq!(version, "1.8.0_301");
        assert_eq!(major, 8);
    }

    #[test]
    fn test_toolchain_matching() {
        let installation = JavaInstallation {
            java_home: PathBuf::from("/usr/lib/jvm/java-17"),
            version: "17.0.1".to_string(),
            major_version: 17,
            vendor: Some("OpenJDK".to_string()),
            java_executable: PathBuf::from("/usr/lib/jvm/java-17/bin/java"),
            javac_executable: PathBuf::from("/usr/lib/jvm/java-17/bin/javac"),
        };

        let toolchain = JavaToolchain::new(17);
        assert!(installation.matches(&toolchain));

        let toolchain_11 = JavaToolchain::new(11);
        assert!(!installation.matches(&toolchain_11));
    }
}
