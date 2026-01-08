//! Artifact bounded context
//!
//! Responsible for artifact management, resolution, and dependency handling.
//! This is a core domain with rich business logic around version resolution,
//! conflict resolution, and transitive dependency management.

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;

pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use repositories::*;
