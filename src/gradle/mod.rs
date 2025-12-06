//! Gradle build system implementation
//! 
//! This module contains Gradle-specific functionality including:
//! - Gradle build script parsing (Groovy/Kotlin DSL)
//! - Gradle task execution and task graph
//! - Gradle dependency management with configurations
//! - Gradle plugin system (java, java-library, application)
//! - Multi-project builds (settings.gradle support)
//! - Composite builds (includeBuild support)
//! - Source sets for organizing source files
//! - Version catalogs (libs.versions.toml)
//! - Java toolchain support

pub mod model;
pub mod core;
pub mod settings;
pub mod task_graph;
pub mod configuration;
pub mod source_set;
pub mod version_catalog;
pub mod toolchain;
pub mod custom_task;
pub mod composite_build;
pub mod application_plugin;

pub use model::{GradleProject, Task, Dependency, Repository, Plugin, parse_gradle_build_script};
pub use core::GradleExecutor;
pub use settings::{GradleSettings, SubprojectConfig, parse_settings_file, find_settings_file};
pub use task_graph::{TaskGraph, TaskNode, build_java_task_graph};
pub use configuration::{Configuration, ConfigurationContainer, ConfigurationDependency};
pub use source_set::{SourceSet, SourceSetContainer};
pub use version_catalog::{VersionCatalog, LibraryDeclaration, VersionSpec, parse_version_catalog};
pub use toolchain::{JavaToolchain, JavaInstallation, JavaVendor, ToolchainResolver};
pub use custom_task::{CustomTask, TaskAction, TaskRegistry};
pub use composite_build::{CompositeBuild, IncludedBuild};
pub use application_plugin::{ApplicationPlugin, ApplicationExtension};

