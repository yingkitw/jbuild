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
pub use model::*;
pub use core::*;
pub use settings::*;
pub use plugin::*;

