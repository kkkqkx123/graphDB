//! Cache Configuration
//!
//! Configuration types for cache behavior tuning.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    pub max_memory: usize,
    pub memory_ratio: (u32, u32, u32, u32),
    pub ttl: Option<Duration>,
    pub tti: Option<Duration>,
    pub high_priority_ratio: f32,
}

impl Default for RecordCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            memory_ratio: (40, 0, 40, 20),
            ttl: Some(Duration::from_secs(3600)),
            tti: Some(Duration::from_secs(300)),
            high_priority_ratio: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryPressureLevel {
    Normal,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct MemoryPressureConfig {
    pub enabled: bool,
    pub high_watermark: f32,
    pub low_watermark: f32,
    pub reduction_factor: f32,
}

impl Default for MemoryPressureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            high_watermark: 0.9,
            low_watermark: 0.7,
            reduction_factor: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheWarmupConfig {
    pub enabled: bool,
    pub warmup_vertex_labels: Vec<u16>,
    pub warmup_edge_labels: Vec<u16>,
    pub max_warmup_entries: usize,
}

impl Default for CacheWarmupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            warmup_vertex_labels: vec![],
            warmup_edge_labels: vec![],
            max_warmup_entries: 10000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WarmupStats {
    pub vertices_loaded: usize,
    pub edges_loaded: usize,
    pub id_indexes_loaded: usize,
    pub total_bytes: usize,
    pub duration_ms: u64,
}

impl Default for WarmupStats {
    fn default() -> Self {
        Self {
            vertices_loaded: 0,
            edges_loaded: 0,
            id_indexes_loaded: 0,
            total_bytes: 0,
            duration_ms: 0,
        }
    }
}
