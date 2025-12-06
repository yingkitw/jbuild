//! Common Utilities
//!
//! Shared utilities used across Maven and Gradle implementations.

pub mod version;

pub use version::{compare_versions, version_key, is_snapshot, base_version};
