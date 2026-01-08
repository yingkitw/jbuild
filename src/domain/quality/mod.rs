//! Code Quality bounded context
//!
//! Responsible for code quality checks (linting, formatting).

pub mod aggregates;
pub mod value_objects;
pub mod services;

pub use aggregates::*;
pub use value_objects::*;
pub use services::*;
