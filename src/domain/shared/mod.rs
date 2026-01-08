//! Shared domain concepts used across bounded contexts
//!
//! This module contains value objects and domain primitives that are shared
//! across multiple bounded contexts.

pub mod value_objects;
pub mod events;

pub use value_objects::*;
pub use events::*;
