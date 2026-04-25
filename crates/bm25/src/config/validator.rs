//! Configuration validator framework
//!
//! Provides traits and implementations for validating configuration.
//!
//! # Examples
//!
//! ```rust
//! use bm25_service::config::{ConfigValidator, IndexManagerConfig};
//!
//! let config = IndexManagerConfig::builder()
//!     .writer_memory_mb(100)
//!     .writer_threads(4)
//!     .build();
//!
//! // Validate configuration
//! config.validate().expect("Config should be valid");
//! ```

use std::error::Error;
use std::fmt;

/// Configuration validation error
#[derive(Debug)]
pub enum ValidationError {
    /// Invalid value for a field
    InvalidValue {
        /// Field name
        field: String,
        /// Invalid value
        value: String,
        /// Reason why it's invalid
        reason: String,
    },
    /// Missing required field
    MissingField(String),
    /// Configuration dependency error
    DependencyError {
        /// Field that has the dependency
        field: String,
        /// Dependency description
        dependency: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidValue {
                field,
                value,
                reason,
            } => {
                write!(f, "Invalid value for {}: {} ({})", field, value, reason)
            }
            ValidationError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ValidationError::DependencyError { field, dependency } => {
                write!(
                    f,
                    "Configuration dependency error for {}: {}",
                    field, dependency
                )
            }
        }
    }
}

impl Error for ValidationError {}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Configuration validator trait
///
/// Implement this trait to add validation logic to configuration types.
///
/// # Examples
///
/// ```rust
/// use bm25_service::config::{ConfigValidator, ValidationError, ValidationResult};
///
/// struct MyConfig {
///     value: usize,
/// }
///
/// impl ConfigValidator for MyConfig {
///     fn validate(&self) -> ValidationResult<()> {
///         if self.value == 0 {
///             return Err(ValidationError::InvalidValue {
///                 field: "value".to_string(),
///                 value: self.value.to_string(),
///                 reason: "must be greater than 0".to_string(),
///             });
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait ConfigValidator {
    /// Validate the configuration
    ///
    /// Returns `Ok(())` if the configuration is valid, or an error describing
    /// the validation failure.
    fn validate(&self) -> ValidationResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestConfig {
        value: usize,
    }

    impl ConfigValidator for TestConfig {
        fn validate(&self) -> ValidationResult<()> {
            if self.value == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "value".to_string(),
                    value: self.value.to_string(),
                    reason: "must be greater than 0".to_string(),
                });
            }
            Ok(())
        }
    }

    #[test]
    fn test_valid_config() {
        let config = TestConfig { value: 42 };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let config = TestConfig { value: 0 };
        let result = config.validate();
        assert!(result.is_err());

        if let Err(ValidationError::InvalidValue {
            field,
            value,
            reason,
        }) = result
        {
            assert_eq!(field, "value");
            assert_eq!(value, "0");
            assert!(reason.contains("greater than 0"));
        } else {
            panic!("Expected InvalidValue error");
        }
    }

    #[test]
    fn test_error_display() {
        let err = ValidationError::InvalidValue {
            field: "test".to_string(),
            value: "123".to_string(),
            reason: "invalid".to_string(),
        };
        assert_eq!(format!("{}", err), "Invalid value for test: 123 (invalid)");
    }
}
