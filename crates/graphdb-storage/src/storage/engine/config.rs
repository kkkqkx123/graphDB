//! Property Graph Configuration

use std::path::PathBuf;
use std::time::Duration;

use crate::storage::compression::CompressionType;

/// Configuration for flush operations
#[derive(Debug, Clone)]
pub struct FlushConfig {
    pub flush_threshold: usize,
    pub flush_interval: Duration,
    pub compression: CompressionType,
    pub background_flush_enabled: bool,
    pub work_dir: PathBuf,
}

impl Default for FlushConfig {
    fn default() -> Self {
        Self {
            flush_threshold: 1000,
            flush_interval: Duration::from_secs(60),
            compression: CompressionType::Zstd { level: 3 },
            background_flush_enabled: true,
            work_dir: PathBuf::from("./data"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub work_dir: PathBuf,
    pub enable_cache: bool,
    pub cache_memory: usize,
    pub flush_config: FlushConfig,
    pub enable_background_flush: bool,
}

impl Default for PropertyGraphConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            work_dir: PathBuf::from("./data"),
            enable_cache: true,
            cache_memory: 128 * 1024 * 1024,
            flush_config: FlushConfig::default(),
            enable_background_flush: true,
        }
    }
}

impl PropertyGraphConfig {
    pub fn with_cache(mut self, enable: bool, cache_memory: usize) -> Self {
        self.enable_cache = enable;
        self.cache_memory = cache_memory;
        self
    }

    pub fn with_work_dir(mut self, work_dir: PathBuf) -> Self {
        self.work_dir = work_dir;
        self
    }

    pub fn with_capacity(mut self, vertex_capacity: usize, edge_capacity: usize) -> Self {
        self.initial_vertex_capacity = vertex_capacity;
        self.initial_edge_capacity = edge_capacity;
        self
    }

    pub fn with_flush_config(mut self, config: FlushConfig) -> Self {
        self.flush_config = config;
        self
    }

    pub fn with_background_flush(mut self, enable: bool) -> Self {
        self.enable_background_flush = enable;
        self
    }

    pub fn with_flush_threshold(mut self, threshold: usize) -> Self {
        self.flush_config.flush_threshold = threshold;
        self
    }

    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_config.flush_interval = interval;
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.flush_config.compression = compression;
        self
    }
}
