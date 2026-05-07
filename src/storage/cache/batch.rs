//! Batch Operations
//!
//! Types for batch cache operations.

use super::types::{EdgeQueryKey, VertexCacheKey};

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
    EdgeQuery(EdgeQueryKey),
    IdIndex(u16, &'a str),
}
