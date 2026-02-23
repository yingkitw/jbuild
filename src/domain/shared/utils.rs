//! Shared utility functions and traits for domain layer

use anyhow::{anyhow, Result};

/// Validates that a string field is not empty after trimming whitespace
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(anyhow!("{field_name} cannot be empty"))
    } else {
        Ok(())
    }
}

/// Validates multiple string fields at once
pub fn validate_fields(fields: &[(&str, &str)]) -> Result<()> {
    for (value, name) in fields {
        validate_non_empty(value, name)?;
    }
    Ok(())
}

/// Macro to generate string conversion methods for enums
///
/// # Example
/// ```
/// // This is a macro, usage is shown in artifact::value_objects::Scope
/// // Example usage (not executable in doctest):
/// // use jbuild::impl_str_conversion;
/// //
/// // #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// // pub enum MyEnum {
/// //     VariantA,
/// //     VariantB,
/// // }
/// //
/// // impl_str_conversion!(MyEnum, {
/// //     VariantA => "variant_a",
/// //     VariantB => "variant_b",
/// // });
/// ```
#[macro_export]
macro_rules! impl_str_conversion {
    ($enum_name:ident, { $($variant:ident => $str:expr),* $(,)? }) => {
        impl $enum_name {
            pub fn as_str(&self) -> &str {
                match self {
                    $($enum_name::$variant => $str),*
                }
            }
        }

        impl std::str::FromStr for $enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $($str => Ok($enum_name::$variant),)*
                    _ => Err(format!("Invalid {}: {}", stringify!($enum_name), s)),
                }
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

/// Macro to generate data-driven enum with associated values
///
/// # Example
/// ```
/// // This is a macro, usage is shown in maven::value_objects::LifecyclePhase
/// // Example usage (not executable in doctest):
/// // use jbuild::impl_data_driven_enum;
/// //
/// // #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// // pub enum MyEnum {
/// //     VariantA,
/// //     VariantB,
/// // }
/// //
/// // impl_data_driven_enum!(MyEnum, {
/// //     VariantA => { name: "variant_a", order: 0 },
/// //     VariantB => { name: "variant_b", order: 1 },
/// // });
/// ```
#[macro_export]
macro_rules! impl_data_driven_enum {
    ($enum_name:ident, { $($variant:ident => { name: $name:expr, order: $order:expr }),* $(,)? }) => {
        impl $enum_name {
            pub fn as_str(&self) -> &str {
                match self {
                    $($enum_name::$variant => $name),*
                }
            }

            pub fn order(&self) -> u32 {
                match self {
                    $($enum_name::$variant => $order),*
                }
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("test", "field").is_ok());
        assert!(validate_non_empty("  test  ", "field").is_ok());
        assert!(validate_non_empty("", "field").is_err());
        assert!(validate_non_empty("   ", "field").is_err());
    }

    #[test]
    fn test_validate_fields() {
        assert!(validate_fields(&[("test1", "field1"), ("test2", "field2")]).is_ok());
        assert!(validate_fields(&[("test", "field1"), ("", "field2")]).is_err());
    }
}
