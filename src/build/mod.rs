//! Build system abstraction layer
//! 
//! This module provides abstractions for different build systems (Maven, Gradle)
//! and detection logic to determine which build system to use.

pub mod detection;
pub mod executor;

pub use detection::*;
pub use executor::*;

