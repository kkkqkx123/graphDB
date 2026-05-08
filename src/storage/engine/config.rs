use std::path::PathBuf;

use crate::storage::memory::MemoryConfig;
use crate::storage::persistence::CompressionType;

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub work_dir: PathBuf,
    pub enable_cache: bool,
    pub cache_memory: usize,
    pub memory_config: MemoryConfig,
    pub flush_threshold: usize,
    pub flush_interval_secs: u64,
    pub compression: CompressionType,
    pub enable_incremental_flush: bool,
}

impl Default for PropertyGraphConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            work_dir: PathBuf::from("./data"),
            enable_cache: true,
            cache_memory: 256 * 1024 * 1024,
            memory_config: MemoryConfig::default(),
            flush_threshold: 1000,
            flush_interval_secs: 60,
            compression: CompressionType::Zstd { level: 3 },
            enable_incremental_flush: true,
        }
    }
}

impl PropertyGraphConfig {
    pub fn with_cache(mut self, enable: bool, cache_memory: usize) -> Self {
        self.enable_cache = enable;
        self.cache_memory = cache_memory;
        self
    }

    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    pub fn with_flush_config(mut self, threshold: usize, interval_secs: u64) -> Self {
        self.flush_threshold = threshold;
        self.flush_interval_secs = interval_secs;
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }
}
