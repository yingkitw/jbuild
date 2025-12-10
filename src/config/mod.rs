//! Configuration handling (jbuild.toml)

pub mod jbuild_toml;
pub mod lock;

pub use jbuild_toml::{JbuildConfig, PackageSection};
pub use lock::{JbuildLock, write_lock_file};

