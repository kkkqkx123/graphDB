//! Batch Operations
//!
//! Types for batch cache operations.

use std::collections::HashMap;

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

impl<T: Clone> BatchGetResult<T> {
    /// Convert batch results into a map of key -> value for successful hits
    pub fn to_map(&self, keys: &[VertexCacheKey]) -> HashMap<VertexCacheKey, T> {
        let mut map = HashMap::with_capacity(self.hits);
        for (key, result) in keys.iter().zip(self.results.iter()) {
            if let Some(value) = result {
                map.insert(*key, value.clone());
            }
        }
        map
    }
}

/// Reference to a cache key for batch invalidation
pub enum CacheKeyRef<'a> {
    /// Vertex cache key
    Vertex(VertexCacheKey),
    /// ID index cache key: (label_id, external_id)
    IdIndex(u32, &'a str),
}
