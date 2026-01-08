//! Packaging bounded context
//!
//! Responsible for creating distributable artifacts (JAR, WAR).

pub mod aggregates;
pub mod value_objects;
pub mod services;

pub use aggregates::*;
pub use value_objects::*;
pub use services::*;
