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

pub mod build_system;
pub mod maven;
pub mod gradle;
pub mod artifact;
pub mod compilation;
pub mod testing;
pub mod packaging;
pub mod plugin;
pub mod config;
pub mod quality;
pub mod shared;

pub use build_system::*;
pub use maven::*;
pub use gradle::*;
pub use artifact::*;
pub use shared::*;
