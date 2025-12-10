//! jbuild.lock generation and parsing

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JbuildLock {
    pub package: LockPackage,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
}

pub fn write_lock_file(path: &Path, cfg: &crate::config::JbuildConfig) -> Result<()> {
    let lock = JbuildLock {
        package: LockPackage {
            name: cfg.package.name.clone(),
            version: cfg.package.version.clone(),
        },
        dependencies: cfg.dependencies.clone(),
        dev_dependencies: cfg.dev_dependencies.clone(),
    };

    let toml_str = toml::to_string_pretty(&lock)
        .context("Failed to serialize jbuild.lock")?;
    fs::write(path, toml_str)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

