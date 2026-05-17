//! Storage configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Edge property cache configuration
///
/// Optional caching for edge properties in high-load scenarios.
/// Disabled by default, enable when:
/// - Edge property access frequency exceeds threshold
/// - Property size is small (< 1KB)
/// - Edge update frequency is low
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EdgePropertyCacheConfig {
    /// Enable edge property cache (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Maximum number of cache entries
    #[serde(default = "default_edge_cache_max_entries")]
    pub max_entries: usize,

    /// Maximum memory usage in bytes
    #[serde(default = "default_edge_cache_max_memory")]
    pub max_memory: usize,

    /// TTL in seconds
    #[serde(default = "default_edge_cache_ttl")]
    pub ttl_secs: u64,

    /// Minimum access frequency to cache
    #[serde(default = "default_min_access_frequency")]
    pub min_access_frequency: u32,

    /// Maximum property size to cache (bytes)
    #[serde(default = "default_max_property_size")]
    pub max_property_size: usize,
}

fn default_edge_cache_max_entries() -> usize {
    10_000
}

fn default_edge_cache_max_memory() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_edge_cache_ttl() -> u64 {
    300 // 5 minutes
}

fn default_min_access_frequency() -> u32 {
    5
}

fn default_max_property_size() -> usize {
    1024 // 1KB
}

impl Default for EdgePropertyCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: default_edge_cache_max_entries(),
            max_memory: default_edge_cache_max_memory(),
            ttl_secs: default_edge_cache_ttl(),
            min_access_frequency: default_min_access_frequency(),
            max_property_size: default_max_property_size(),
        }
    }
}

impl EdgePropertyCacheConfig {
    /// Create an enabled configuration with default values
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Create a high-performance configuration for server environments
    pub fn high_performance() -> Self {
        Self {
            enabled: true,
            max_entries: 100_000,
            max_memory: 100 * 1024 * 1024, // 100MB
            ttl_secs: 600,
            min_access_frequency: 3,
            max_property_size: 2048,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_entries == 0 {
            return Err("max_entries must be greater than 0".to_string());
        }
        if self.max_memory == 0 {
            return Err("max_memory must be greater than 0".to_string());
        }
        if self.ttl_secs == 0 {
            return Err("ttl_secs must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl From<EdgePropertyCacheConfig> for crate::storage::cache::EdgePropertyCacheConfig {
    fn from(config: EdgePropertyCacheConfig) -> Self {
        Self {
            enabled: config.enabled,
            max_entries: config.max_entries,
            max_memory: config.max_memory,
            ttl: Duration::from_secs(config.ttl_secs),
            min_access_frequency: config.min_access_frequency,
            max_property_size: config.max_property_size,
        }
    }
}

/// Storage engine type
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageEngine {
    /// PropertyGraph storage engine (columnar + CSR)
    #[default]
    PropertyGraph,
    /// RocksDB storage engine (future support)
    #[serde(rename = "rocksdb")]
    RocksDB,
}

impl std::fmt::Display for StorageEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PropertyGraph => write!(f, "propertygraph"),
            Self::RocksDB => write!(f, "rocksdb"),
        }
    }
}

/// Compression algorithm
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// No compression
    #[default]
    None,
    /// Zstandard compression
    Zstd,
}

impl std::fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Zstd => write!(f, "zstd"),
        }
    }
}

/// Storage configuration
///
/// Configures the storage engine behavior and performance characteristics.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    /// Storage engine type (propertygraph, rocksdb, etc.)
    #[serde(default)]
    pub engine: StorageEngine,

    /// Compression algorithm (none, lz4, zstd, snappy)
    #[serde(default)]
    pub compression: CompressionAlgorithm,

    /// Compression level (0-9, engine-dependent)
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,

    /// WAL flush interval (milliseconds, 0 = immediate)
    #[serde(default)]
    pub wal_flush_interval_ms: u64,

    /// Checkpoint interval (seconds, 0 = disabled)
    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval_secs: u64,

    /// Maximum database size (bytes, 0 = unlimited)
    #[serde(default)]
    pub max_db_size: u64,

    /// Enable automatic statistics collection
    #[serde(default = "default_true")]
    pub auto_statistics: bool,

    /// Statistics collection interval (seconds)
    #[serde(default = "default_statistics_interval")]
    pub statistics_interval_secs: u64,

    /// Edge property cache configuration
    #[serde(default)]
    pub edge_property_cache: EdgePropertyCacheConfig,
}

fn default_compression_level() -> u32 {
    3
}

fn default_checkpoint_interval() -> u64 {
    300 // 5 minutes
}

fn default_statistics_interval() -> u64 {
    60 // 1 minute
}

fn default_true() -> bool {
    true
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            engine: StorageEngine::default(),
            compression: CompressionAlgorithm::default(),
            compression_level: default_compression_level(),
            wal_flush_interval_ms: 0, // Immediate flush for safety
            checkpoint_interval_secs: default_checkpoint_interval(),
            max_db_size: 0, // Unlimited
            auto_statistics: true,
            statistics_interval_secs: default_statistics_interval(),
            edge_property_cache: EdgePropertyCacheConfig::default(),
        }
    }
}

impl StorageConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.compression_level > 9 {
            return Err("Compression level must be between 0 and 9".to_string());
        }
        self.edge_property_cache.validate()?;
        Ok(())
    }

    /// Check if compression is enabled
    pub fn is_compression_enabled(&self) -> bool {
        !matches!(self.compression, CompressionAlgorithm::None)
    }

    /// Check if edge property cache is enabled
    pub fn is_edge_cache_enabled(&self) -> bool {
        self.edge_property_cache.enabled
    }
}

/// Query resource configuration
///
/// Controls resource limits for query execution.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueryResourceConfig {
    /// Maximum memory per query (bytes, 0 = unlimited)
    #[serde(default)]
    pub max_memory_per_query: u64,

    /// Maximum concurrent queries (0 = unlimited)
    #[serde(default = "default_max_concurrent_queries")]
    pub max_concurrent_queries: usize,

    /// Query timeout (seconds, 0 = no timeout)
    #[serde(default)]
    pub query_timeout_secs: u64,

    /// Maximum result set size (0 = unlimited)
    #[serde(default)]
    pub max_result_size: usize,

    /// Maximum number of vertices to scan in a single query
    #[serde(default = "default_max_vertex_scan")]
    pub max_vertex_scan: usize,

    /// Maximum number of edges to scan in a single query
    #[serde(default = "default_max_edge_scan")]
    pub max_edge_scan: usize,
}

fn default_max_concurrent_queries() -> usize {
    100
}

fn default_max_vertex_scan() -> usize {
    1_000_000
}

fn default_max_edge_scan() -> usize {
    10_000_000
}

impl Default for QueryResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_per_query: 0, // Unlimited
            max_concurrent_queries: default_max_concurrent_queries(),
            query_timeout_secs: 0, // No timeout
            max_result_size: 0,    // Unlimited
            max_vertex_scan: default_max_vertex_scan(),
            max_edge_scan: default_max_edge_scan(),
        }
    }
}

impl QueryResourceConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_queries == 0 {
            return Err("Max concurrent queries must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Check if memory limit is enabled
    pub fn has_memory_limit(&self) -> bool {
        self.max_memory_per_query > 0
    }

    /// Check if query timeout is enabled
    pub fn has_timeout(&self) -> bool {
        self.query_timeout_secs > 0
    }

    /// Check if result size limit is enabled
    pub fn has_result_size_limit(&self) -> bool {
        self.max_result_size > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.engine, StorageEngine::PropertyGraph);
        assert_eq!(config.compression, CompressionAlgorithm::None);
        assert_eq!(config.compression_level, 3);
        assert_eq!(config.checkpoint_interval_secs, 300);
        assert!(config.auto_statistics);
    }

    #[test]
    fn test_storage_config_validate() {
        let config = StorageConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = StorageConfig {
            compression_level: 10,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_query_resource_config_default() {
        let config = QueryResourceConfig::default();
        assert_eq!(config.max_concurrent_queries, 100);
        assert_eq!(config.max_vertex_scan, 1_000_000);
        assert_eq!(config.max_edge_scan, 10_000_000);
        assert!(!config.has_memory_limit());
        assert!(!config.has_timeout());
    }

    #[test]
    fn test_query_resource_config_validate() {
        let config = QueryResourceConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = QueryResourceConfig {
            max_concurrent_queries: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_storage_engine_display() {
        assert_eq!(StorageEngine::PropertyGraph.to_string(), "propertygraph");
        assert_eq!(StorageEngine::RocksDB.to_string(), "rocksdb");
    }

    #[test]
    fn test_compression_algorithm_display() {
        assert_eq!(CompressionAlgorithm::None.to_string(), "none");
        assert_eq!(CompressionAlgorithm::Zstd.to_string(), "zstd");
    }

    #[test]
    fn test_edge_property_cache_config_default() {
        let config = EdgePropertyCacheConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_entries, 10_000);
        assert_eq!(config.max_memory, 10 * 1024 * 1024);
        assert_eq!(config.ttl_secs, 300);
        assert_eq!(config.min_access_frequency, 5);
        assert_eq!(config.max_property_size, 1024);
    }

    #[test]
    fn test_edge_property_cache_config_enabled() {
        let config = EdgePropertyCacheConfig::enabled();
        assert!(config.enabled);
    }

    #[test]
    fn test_edge_property_cache_config_high_performance() {
        let config = EdgePropertyCacheConfig::high_performance();
        assert!(config.enabled);
        assert_eq!(config.max_entries, 100_000);
        assert_eq!(config.max_memory, 100 * 1024 * 1024);
    }

    #[test]
    fn test_edge_property_cache_config_validate() {
        let config = EdgePropertyCacheConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = EdgePropertyCacheConfig {
            max_entries: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_storage_config_edge_cache_enabled() {
        let config = StorageConfig::default();
        assert!(!config.is_edge_cache_enabled());

        let enabled_config = StorageConfig {
            edge_property_cache: EdgePropertyCacheConfig::enabled(),
            ..Default::default()
        };
        assert!(enabled_config.is_edge_cache_enabled());
    }

    #[test]
    fn test_edge_property_cache_config_conversion_coverage() {
        let config = EdgePropertyCacheConfig {
            enabled: true,
            max_entries: 50000,
            max_memory: 50 * 1024 * 1024,
            ttl_secs: 600,
            min_access_frequency: 10,
            max_property_size: 2048,
        };

        let cache_config: crate::storage::cache::EdgePropertyCacheConfig = config.into();

        assert!(cache_config.enabled);
        assert_eq!(cache_config.max_entries, 50000);
        assert_eq!(cache_config.max_memory, 50 * 1024 * 1024);
        assert_eq!(cache_config.ttl, std::time::Duration::from_secs(600));
        assert_eq!(cache_config.min_access_frequency, 10);
        assert_eq!(cache_config.max_property_size, 2048);
    }
}
