//! Cache Configuration
//!
//! Configuration types for cache behavior tuning.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    pub max_memory: usize,
    /// Memory distribution ratio: (vertex, edge_query, id_index)
    /// - vertex: Vertex record cache
    /// - edge_query: Edge query result cache
    /// - id_index: External ID to internal ID mapping cache
    pub memory_ratio: (u32, u32, u32),
    pub ttl: Option<Duration>,
    pub tti: Option<Duration>,
    pub high_priority_ratio: f32,
}

impl Default for RecordCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            memory_ratio: (50, 30, 20),
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
