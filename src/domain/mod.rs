//! Domain Layer - Core business logic and domain models
//!
//! This module contains the domain layer following Domain-Driven Design (DDD) principles.
//! It is organized into bounded contexts, each representing a distinct area of the domain.
//!
//! ## Bounded Contexts
//!
//! - **build_system**: Build system detection and abstraction
//! - **maven**: Maven-specific domain models and services
//! - **gradle**: Gradle-specific domain models and services
//! - **artifact**: Artifact management and resolution
//! - **shared**: Common value objects used across contexts

pub mod artifact;
pub mod build_system;
pub mod gradle;
pub mod maven;
pub mod shared;

// Re-export build_system
pub use build_system::{
    BuildSystemDetector,
    value_objects::BuildFile,
};

// Re-export maven
pub use maven::{
    aggregates::{MavenDependency, MavenPlugin, MavenProject, PackagingType},
    services::LifecycleExecutor,
    value_objects::LifecyclePhase,
};

// Re-export gradle
pub use gradle::{
    aggregates::{Configuration, GradleProject, GradleTask},
    services::TaskExecutor,
};

// Re-export artifact
pub use artifact::{
    repositories::{LocalRepository, RemoteRepository},
    services::DependencyResolver,
    value_objects::{ArtifactCoordinates, Scope},
};

// Re-export shared
pub use shared::value_objects::{FilePath, JavaVersion, Version};
