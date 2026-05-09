//! Property Graph Configuration

use std::path::PathBuf;

use crate::storage::memory::MemoryConfig;

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub work_dir: PathBuf,
    pub enable_cache: bool,
    pub cache_memory: usize,
    pub memory_config: MemoryConfig,
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

    pub fn with_work_dir(mut self, work_dir: PathBuf) -> Self {
        self.work_dir = work_dir;
        self
    }

    pub fn with_capacity(mut self, vertex_capacity: usize, edge_capacity: usize) -> Self {
        self.initial_vertex_capacity = vertex_capacity;
        self.initial_edge_capacity = edge_capacity;
        self
    }
}
