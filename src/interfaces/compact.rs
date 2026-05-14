//! Compact Operation Interface
//!
//! Defines the interface for storage compaction operations.
//! This trait abstracts storage-specific compaction details from the transaction layer.

use crate::core::types::Timestamp;

/// Configuration for compact operations
#[derive(Debug, Clone)]
pub struct CompactConfig {
    pub enable_structure_compaction: bool,
    pub reserve_ratio: f32,
}

impl CompactConfig {
    pub fn new(enable_structure_compaction: bool, reserve_ratio: f32) -> Self {
        Self {
            enable_structure_compaction,
            reserve_ratio: reserve_ratio.clamp(0.0, 1.0),
        }
    }
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self::new(true, 0.8)
    }
}

/// Statistics about storage compaction
#[derive(Debug, Clone)]
pub struct CompactStats {
    pub total_size: usize,
    pub used_size: usize,
    pub fragmentation_ratio: f32,
}

impl CompactStats {
    pub fn new(total_size: usize, used_size: usize) -> Self {
        let fragmentation_ratio = if total_size > 0 {
            1.0 - (used_size as f32 / total_size as f32)
        } else {
            0.0
        };
        Self {
            total_size,
            used_size,
            fragmentation_ratio,
        }
    }
}

/// Compact transaction result type
pub type CompactResult<T> = Result<T, CompactError>;

/// Compact transaction error
#[derive(Debug, Clone, thiserror::Error)]
pub enum CompactError {
    #[error("Compact operation failed: {0}")]
    CompactFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Trait for targets that can be compacted
/// This abstracts the storage-specific implementation details from the transaction layer
pub trait CompactTarget: Send + Sync {
    fn compact(&mut self, config: &CompactConfig, ts: Timestamp) -> CompactResult<()>;
    fn get_compact_stats(&self) -> CompactStats;
}
