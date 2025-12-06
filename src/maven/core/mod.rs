//! Maven core execution engine
//! 
//! Re-exports from the main core module for Maven-specific execution.

pub mod executor;

pub use crate::core::*;
pub use executor::MavenBuildExecutor;

