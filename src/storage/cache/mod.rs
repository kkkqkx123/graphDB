//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.

mod types;
mod stats;
mod config;
mod batch;
mod predictor;
mod record_cache;
mod graph_aware_cache;
mod warmup;

#[cfg(test)]
mod record_cache_test;

pub use types::*;
pub use stats::{CacheTypeStats, CacheTypeStatsSnapshot, RecordCacheStats};
pub use config::*;
pub use batch::*;
pub use predictor::*;
pub use record_cache::{RecordCache, SharedRecordCache};
pub use graph_aware_cache::{
    GraphAwareCache, GraphCacheConfig, GraphCacheStats,
    NeighborCacheKey, PropertyCacheKey, CachedNeighbor, CachedProperty,
    NeighborEntry, AccessFrequency,
};
pub use warmup::{CacheWarmup, WarmupDataProvider};
