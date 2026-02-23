//! Artifact bounded context
//!
//! Responsible for artifact management, resolution, and dependency handling.
//! This is a core domain with rich business logic around version resolution,
//! conflict resolution, and transitive dependency management.

pub mod parallel_resolver;
pub mod repositories;
pub mod services;
pub mod value_objects;

#[cfg(test)]
pub mod test_utils;

pub use parallel_resolver::*;
pub use repositories::*;
pub use services::*;
pub use value_objects::*;
