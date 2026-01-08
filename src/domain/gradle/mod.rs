//! Gradle bounded context
//!
//! Responsible for Gradle-specific build system implementation.
//! Contains Gradle domain models, services, and business logic.

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;

pub use aggregates::*;
pub use services::*;
