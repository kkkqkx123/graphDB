//! Storage Iterator - provides configuration and utilities for storage iteration
//!
//! Offer:
//! - IterStats: Iterator statistics (records to metrics crate)
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
        StorageError::DbError(err.to_string())
    }
}

/// Iterator statistics using metrics crate
///
/// All metrics are recorded via the `metrics` crate.
/// No internal counters are maintained to avoid double recording.
#[derive(Debug, Clone, Default)]
pub struct IterStats;

impl IterStats {
    pub fn new() -> Self {
        Self
    }

    pub fn record_scan(&self) {
        metrics::counter!("graphdb_storage_iter_items_scanned_total").increment(1);
    }

    pub fn record_return(&self) {
        metrics::counter!("graphdb_storage_iter_items_returned_total").increment(1);
    }

    pub fn record_seek(&self) {
        metrics::counter!("graphdb_storage_iter_seek_operations_total").increment(1);
    }

    pub fn record_cache_hit(&self) {
        metrics::counter!("graphdb_storage_iter_cache_hits_total").increment(1);
    }

    pub fn record_cache_miss(&self) {
        metrics::counter!("graphdb_storage_iter_cache_misses_total").increment(1);
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
    fn test_iter_stats() {
        let stats = IterStats::new();
        stats.record_scan();
        stats.record_scan();
        stats.record_return();
        stats.record_seek();
        stats.record_cache_hit();
        stats.record_cache_miss();
    }

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
