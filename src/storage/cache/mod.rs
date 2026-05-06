//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.

mod record_cache;

pub use record_cache::{
    CachedEdge, CachedVertex, EdgeCacheKey, EdgeQueryKey, IdIndexCacheKey, RecordCache,
    RecordCacheConfig, RecordCacheStats, SharedRecordCache, VertexCacheKey,
};
