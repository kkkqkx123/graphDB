//! Storage Iterator - provides configuration and utilities for storage iteration
//!
//! Offer:
//! - IterConfig: Iterator configuration
//! - IterError: Iterator error types

use crate::core::StorageError;

/// Iterator error types
#[derive(Debug, Clone, PartialEq)]
pub enum IterError {
    InvalidState(String),
    IoError(String),
    SchemaMismatch(String),
    NotFound(String),
}

impl std::fmt::Display for IterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterError::InvalidState(msg) => write!(f, "Invalid iterator state: {}", msg),
            IterError::IoError(msg) => write!(f, "IO error: {}", msg),
            IterError::SchemaMismatch(msg) => write!(f, "Schema mismatch: {}", msg),
            IterError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for IterError {}

impl From<IterError> for StorageError {
    fn from(err: IterError) -> Self {
        StorageError::db_error(err.to_string())
    }
}

/// Iterator Configuration
#[derive(Debug, Clone)]
pub struct IterConfig {
    pub prefetch_size: usize,
    pub use_cache: bool,
    pub cache_size: usize,
    pub parallel_scan: bool,
}

impl Default for IterConfig {
    fn default() -> Self {
        Self {
            prefetch_size: 100,
            use_cache: true,
            cache_size: 10000,
            parallel_scan: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_config() {
        let config = IterConfig::default();
        assert_eq!(config.prefetch_size, 100);
        assert!(config.use_cache);
        assert_eq!(config.cache_size, 10000);
        assert!(!config.parallel_scan);
    }

    #[test]
    fn test_iter_error_display() {
        let err = IterError::InvalidState("test".to_string());
        assert_eq!(format!("{}", err), "Invalid iterator state: test");

        let err = IterError::NotFound("key".to_string());
        assert_eq!(format!("{}", err), "Not found: key");
    }
}
