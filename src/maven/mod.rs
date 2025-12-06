//! Maven build system implementation
//! 
//! This module contains all Maven-specific functionality including:
//! - POM model parsing and building
//! - Maven lifecycle execution
//! - Maven plugin system
//! - Maven settings management

pub mod model;
pub mod core;
pub mod settings;
pub mod plugin;

// Re-export commonly used Maven types
// Note: Using explicit re-exports to avoid ambiguous glob re-exports
// between model::Profile/settings::Profile and model::Plugin/plugin::Plugin
pub use model::{Model, Dependency, Build, Parent, Repository, DistributionManagement};
pub use core::MavenBuildExecutor;
pub use settings::Settings;

