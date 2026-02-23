//! Multi-language support for JVM languages

pub mod java;
pub mod kotlin;
pub mod scala;
pub mod groovy;
pub mod detector;

pub use java::*;
pub use kotlin::*;
pub use scala::*;
pub use groovy::*;
pub use detector::*;
