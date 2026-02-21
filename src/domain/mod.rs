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
//! - **compilation**: Java source compilation
//! - **testing**: Test discovery and execution
//! - **packaging**: Artifact packaging (JAR, WAR)
//! - **plugin**: Plugin loading and execution
//! - **config**: Project configuration (jbuild.toml)
//! - **quality**: Code quality checks
//!
//! ## Domain Model Patterns
//!
//! - **Entities**: Objects with identity (e.g., MavenProject, Task)
//! - **Value Objects**: Immutable objects without identity (e.g., ArtifactCoordinates, Version)
//! - **Aggregates**: Consistency boundaries (e.g., MavenProject aggregate)
//! - **Domain Services**: Business logic that doesn't fit in entities
//! - **Repositories**: Abstraction for data access
//! - **Domain Events**: Decoupled communication between contexts

pub mod artifact;
pub mod build_system;
pub mod compilation;
pub mod config;
pub mod gradle;
pub mod maven;
pub mod packaging;
pub mod plugin;
pub mod quality;
pub mod shared;
pub mod testing;

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
    repositories::{LocalRepository, RemoteRepository, RepositoryChain},
    services::DependencyResolver,
    value_objects::{ArtifactCoordinates, Scope},
};

// Re-export shared
pub use shared::value_objects::{FilePath, JavaVersion, Version};
