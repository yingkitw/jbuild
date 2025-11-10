pub mod model;
pub mod artifact;
pub mod core;
pub mod resolver;
pub mod settings;
pub mod plugin_api;
pub mod compiler;
pub mod packaging;
pub mod testing;

// Re-export commonly used types without glob imports to avoid conflicts
pub use model::{Model, Dependency, Build, Profile as ModelProfile, Repository as ModelRepository};
pub use artifact::{Artifact, ArtifactCoordinates, LocalRepository};
pub use core::{MavenExecutionRequest, MavenExecutionResult, MavenProject, MavenSession, LifecyclePhase};
pub use resolver::{DependencyResolver, RemoteRepository};
pub use settings::{Settings, Profile as SettingsProfile, Server, Mirror};
pub use plugin_api::{Mojo, Plugin, PluginDescriptor};

