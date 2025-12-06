// Build system abstraction
pub mod build;

// Shared/common functionality
pub mod artifact;
pub mod resolver;
pub mod compiler;
pub mod packaging;
pub mod testing;
pub mod error;
pub mod testing_utils;

// Maven-specific modules
pub mod maven;
pub mod model;  // Maven POM model (kept for backward compatibility)
pub mod core;   // Maven core execution (kept for backward compatibility)
pub mod settings;  // Maven settings (kept for backward compatibility)
pub mod plugin_api;  // Maven plugins (kept for backward compatibility)

// Gradle-specific modules
pub mod gradle;

// Re-export Gradle types
pub use gradle::{GradleProject, Task as GradleTask, Dependency as GradleDependency, parse_gradle_build_script};

// Re-export commonly used types without glob imports to avoid conflicts
pub use model::{Model, Dependency, Build, Profile as ModelProfile, Repository as ModelRepository};
pub use artifact::{Artifact, ArtifactCoordinates, LocalRepository};
pub use core::{MavenExecutionRequest, MavenExecutionResult, MavenProject, MavenSession, LifecyclePhase};
pub use resolver::{DependencyResolver, RemoteRepository};
pub use settings::{Settings, Profile as SettingsProfile, Server, Mirror};
pub use plugin_api::{Mojo, Plugin, PluginDescriptor};
pub use error::{MavenError, MavenResult};
pub use testing_utils::{MockArtifactRepository, MockDependencyResolver, TestProjectBuilder};

