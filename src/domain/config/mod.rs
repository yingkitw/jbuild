//! Configuration bounded context
//!
//! Responsible for project configuration (jbuild.toml, workspace).

pub mod aggregates;
pub mod value_objects;
pub mod services;

pub use aggregates::*;
pub use value_objects::*;
pub use services::*;
