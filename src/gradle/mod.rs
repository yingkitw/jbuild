//! Gradle build system implementation
//! 
//! This module contains Gradle-specific functionality including:
//! - Gradle build script parsing (Groovy/Kotlin DSL)
//! - Gradle task execution
//! - Gradle dependency management
//! - Gradle plugin system

pub mod model;
pub mod core;

pub use model::{GradleProject, Task, Dependency, Repository, Plugin, parse_gradle_build_script};
pub use core::GradleExecutor;

