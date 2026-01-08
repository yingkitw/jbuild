//! Maven bounded context
//!
//! Responsible for Maven-specific build system implementation.
//! Contains Maven domain models, services, and business logic.

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;

pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
