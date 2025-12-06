//! Maven build system implementation
//! 
//! This module contains all Maven-specific functionality including:
//! - POM model parsing and building
//! - Maven lifecycle execution and mapping
//! - Maven plugin system
//! - Maven settings management
//! - Reactor build for multi-module projects
//! - Dependency context and resolution
//! - Execution plan calculation

pub mod model;
pub mod core;
pub mod settings;
pub mod plugin;
pub mod execution_plan;
pub mod reactor_build;
pub mod dependency_context;
pub mod lifecycle_mapping;
pub mod project_dependencies_resolver;

// Re-export commonly used Maven types
// Note: Using explicit re-exports to avoid ambiguous glob re-exports
// between model::Profile/settings::Profile and model::Plugin/plugin::Plugin
pub use model::{Model, Dependency, Build, Parent, Repository, DistributionManagement};
pub use core::MavenBuildExecutor;
pub use settings::Settings;
pub use execution_plan::{MavenExecutionPlan, ExecutionPlanItem};
pub use reactor_build::{ReactorBuildStatus, ReactorProject, ProjectBuildStatus, ReactorSummary};
pub use dependency_context::{DependencyContext, DependencyScope, ResolvedDependency};
pub use lifecycle_mapping::{LifecycleMapping, LifecycleMappingRegistry, PluginBinding};
pub use project_dependencies_resolver::{ProjectDependenciesResolver, DependencyResolutionRequest, ResolutionScope};

