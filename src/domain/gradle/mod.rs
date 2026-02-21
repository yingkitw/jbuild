//! Gradle bounded context
//!
//! Responsible for Gradle-specific build system implementation.
//! Contains Gradle domain models, services, and business logic.

pub mod aggregates;
pub mod entities;
pub mod services;
pub mod value_objects;

pub use aggregates::*;
pub use services::*;
