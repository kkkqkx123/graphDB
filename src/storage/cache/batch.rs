//! Batch Operations
//!
//! Types for batch cache operations.

use super::types::VertexCacheKey;

/// Result of batch insert operation
pub struct BatchInsertResult {
    pub inserted: usize,
    pub total_size: usize,
}

/// Result of batch get operation
pub struct BatchGetResult<T> {
    pub results: Vec<Option<T>>,
    pub hits: usize,
    pub misses: usize,
}

/// Reference to a cache key for batch invalidation
pub enum CacheKeyRef<'a> {
    /// Vertex cache key
    Vertex(VertexCacheKey),
    /// ID index cache key: (label_id, external_id)
    IdIndex(u16, &'a str),
}
