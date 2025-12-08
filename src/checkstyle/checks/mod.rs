//! Checks for Checkstyle-rs

pub mod base;
pub mod empty_catch_block;
pub mod empty_statement;
pub mod line_length;
pub mod missing_switch_default;
pub mod multiple_variable_declarations;
pub mod package_name;
pub mod redundant_import;
pub mod simplify_boolean_return;
pub mod type_name;

pub use base::*;
pub use empty_catch_block::*;
pub use empty_statement::*;
pub use line_length::*;
pub use missing_switch_default::*;
pub use multiple_variable_declarations::*;
pub use package_name::*;
pub use redundant_import::*;
pub use simplify_boolean_return::*;
pub use type_name::*;
