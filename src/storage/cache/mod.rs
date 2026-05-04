//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.

mod block_cache;
mod record_cache;

pub use block_cache::{BlockCache, BlockId, CacheConfig, CacheStats, SharedBlockCache, TableType};
pub use record_cache::{
    CachedEdge, CachedVertex, EdgeCacheKey, RecordCache, RecordCacheConfig, RecordCacheStats,
    SharedRecordCache, VertexCacheKey,
};
