//! Build System bounded context
//!
//! Responsible for detecting and abstracting different build systems (Maven, Gradle, JBuild).
//! Provides anti-corruption layers for translating between build system concepts.

pub mod services;
pub mod value_objects;

pub use services::*;
pub use value_objects::*;
