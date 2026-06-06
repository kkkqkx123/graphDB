//! Property Graph Configuration

use std::time::Duration;

use crate::storage::compression::CompressionType;

/// Configuration for flush operations
#[derive(Debug, Clone)]
pub struct FlushConfig {
    pub flush_threshold: usize,
    pub flush_interval: Duration,
    pub compression: CompressionType,
}

impl Default for FlushConfig {
    fn default() -> Self {
        Self {
            flush_threshold: 1000,
            flush_interval: Duration::from_secs(60),
            compression: CompressionType::Zstd { level: 3 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub enable_cache: bool,
    pub cache_memory: usize,
    pub flush_config: FlushConfig,
}

impl Default for PropertyGraphConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_memory: 128 * 1024 * 1024,
            flush_config: FlushConfig::default(),
        }
    }
}
