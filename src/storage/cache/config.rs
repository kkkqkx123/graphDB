//! Cache Configuration
//!
//! Configuration types for cache behavior tuning.
//!
//! ## Memory Distribution
//!
//! Memory is distributed between two cache types:
//! - **Vertex Cache**: Stores vertex records for fast point lookups
//! - **ID Index Cache**: Stores external_id -> internal_id mappings

use std::time::Duration;

/// Configuration for record cache
#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Memory distribution ratio: (vertex, id_index)
    /// - vertex: Vertex record cache
    /// - id_index: External ID to internal ID mapping cache
    pub memory_ratio: (u32, u32),
    /// Time-to-live for cache entries
    pub ttl: Option<Duration>,
    /// Time-to-idle for cache entries
    pub tti: Option<Duration>,
    /// Ratio of memory allocated for high-priority entries (id_index)
    pub high_priority_ratio: f32,
}

impl Default for RecordCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            memory_ratio: (70, 30),
            ttl: Some(Duration::from_secs(3600)),
            tti: Some(Duration::from_secs(300)),
            high_priority_ratio: 0.0,
        }
    }
}

/// Memory pressure level for cache management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryPressureLevel {
    /// Normal operation
    Normal,
    /// Memory usage is high, consider reducing cache
    Warning,
    /// Memory usage is critical, clear cache immediately
    Critical,
}

/// Configuration for memory pressure response
#[derive(Debug, Clone)]
pub struct MemoryPressureConfig {
    /// Enable memory pressure response
    pub enabled: bool,
    /// High watermark ratio (0.0 - 1.0) to trigger warning
    pub high_watermark: f32,
    /// Low watermark ratio (0.0 - 1.0) to stop reduction
    pub low_watermark: f32,
    /// Factor to reduce cache size when under pressure
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
