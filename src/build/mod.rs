//! Build system abstraction layer
//! 
//! This module provides abstractions for different build systems (Maven, Gradle)
//! and detection logic to determine which build system to use.
//!
//! ## Compatibility Features
//! - **Wrapper support**: Detects and uses mvnw/gradlew wrappers
//! - **Goal mapping**: Maps between Maven phases and Gradle tasks
//! - **Dependency notation**: Converts between Maven and Gradle formats
//! - **Property conversion**: Converts properties between systems

pub mod detection;
pub mod executor;
pub mod wrapper;
pub mod goal_mapping;
pub mod dependency_notation;
pub mod property_conversion;

pub use detection::BuildSystem;
pub use executor::*;
pub use wrapper::{BuildWrapper, WrapperType};
pub use goal_mapping::GoalMapper;
pub use dependency_notation::{DependencyCoordinates, ScopeMapper};
pub use property_conversion::PropertyConverter;

