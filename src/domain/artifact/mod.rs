//! Artifact bounded context
//!
//! Responsible for artifact management, resolution, and dependency handling.
//! This is a core domain with rich business logic around version resolution,
//! conflict resolution, and transitive dependency management.

pub mod entities;
pub mod value_objects;
pub mod repositories;
pub mod services;
pub mod aggregates;

#[cfg(test)]
pub mod test_utils;

pub use value_objects::*;
pub use repositories::*;
pub use services::*;
pub use entities::*;
pub use aggregates::*;
