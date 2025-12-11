//! Configuration handling (jbuild.toml, jbuild-workspace.toml)

pub mod jbuild_toml;
pub mod lock;
pub mod workspace;

pub use jbuild_toml::{JbuildConfig, PackageSection};
pub use lock::{JbuildLock, write_lock_file};
pub use workspace::{JbuildWorkspace, Workspace, WorkspaceMember, WorkspaceDependency, WorkspaceDependencyType};

