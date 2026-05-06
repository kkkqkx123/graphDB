//! Batch Operations
//!
//! Types for batch cache operations.

use super::types::{EdgeCacheKey, EdgeQueryKey, VertexCacheKey};

pub struct BatchInsertResult {
    pub inserted: usize,
    pub total_size: usize,
}

pub struct BatchGetResult<T> {
    pub results: Vec<Option<T>>,
    pub hits: usize,
    pub misses: usize,
}

pub enum CacheKeyRef<'a> {
    Vertex(VertexCacheKey),
    Edge(EdgeCacheKey),
    EdgeQuery(EdgeQueryKey),
    IdIndex(u16, &'a str),
}
