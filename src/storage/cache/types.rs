//! Cache Types
//!
//! Core types for cache keys, values, and eviction handling.
//!
//! ## Design Note: Why No Edge Cache?
//!
//! Edge data is NOT cached separately because:
//!
//! 1. **CSR is already read-optimized**: The CSR (Compressed Sparse Row) structure
//!    provides O(1) edge list access with contiguous memory layout, which is
//!    CPU cache-friendly. Adding another cache layer would not improve performance.
//!
//! 2. **High memory cost**: Edge data volume is typically much larger than vertex
//!    data. Caching edges would consume significant memory with limited benefit.
//!
//! 3. **Frequent updates**: Edges are updated more frequently than vertices,
//!    making cache invalidation complex and potentially causing consistency issues.
//!
//! 4. **Property access is O(1)**: Edge properties are stored in PropertyTable
//!    with direct offset access, which is already optimal.
//!
//! The cache focuses on:
//! - **Vertex Cache**: Caches vertex records for fast point lookups
//! - **ID Index Cache**: Caches external_id -> internal_id mappings

use std::sync::Arc;

use moka::notification::RemovalCause;

use crate::core::Value;

/// Eviction cause for cache entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionCause {
    /// Entry was evicted due to capacity constraints
    Capacity,
    /// Entry expired due to TTL or TTI
    Expired,
    /// Entry was explicitly removed
    Explicit,
    /// Entry was replaced by a new value
    Replaced,
}

impl From<RemovalCause> for EvictionCause {
    fn from(cause: RemovalCause) -> Self {
        match cause {
            RemovalCause::Size => EvictionCause::Capacity,
            RemovalCause::Expired => EvictionCause::Expired,
            RemovalCause::Explicit => EvictionCause::Explicit,
            RemovalCause::Replaced => EvictionCause::Replaced,
        }
    }
}

/// Callback type for eviction notifications
pub type EvictionCallback = Arc<dyn Fn(&str, EvictionCause) + Send + Sync>;

/// Key for vertex cache: (label_id, internal_id)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u32,
    pub internal_id: u32,
}

impl VertexCacheKey {
    pub fn new(label_id: u32, internal_id: u32) -> Self {
        Self {
            label_id,
            internal_id,
        }
    }
}

/// Key for ID index cache: (label_id, external_id)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdIndexCacheKey {
    pub label_id: u32,
    pub external_id: Arc<str>,
}

impl IdIndexCacheKey {
    pub fn new(label_id: u32, external_id: Arc<str>) -> Self {
        Self {
            label_id,
            external_id,
        }
    }
}

/// Cached vertex record
#[derive(Debug, Clone)]
pub struct CachedVertex {
    pub internal_id: u32,
    pub external_id: String,
    pub properties: Vec<(String, Value)>,
}

impl CachedVertex {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<Self>();

        size += self.external_id.capacity();

        for (name, value) in &self.properties {
            size += name.capacity();
            size += value.estimated_size();
        }

        size as u32
    }
}

/// Snapshot entry for transaction rollback
#[derive(Debug, Clone)]
pub enum CacheSnapshotEntry {
    /// Previous vertex value before modification
    Vertex(VertexCacheKey, Option<CachedVertex>),
    /// Previous ID index value before modification
    IdIndex(IdIndexCacheKey, Option<u32>),
}

/// Transaction-level cache snapshot for rollback support
#[derive(Debug)]
pub struct TransactionCacheSnapshot {
    entries: Vec<CacheSnapshotEntry>,
}

impl Default for TransactionCacheSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionCacheSnapshot {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record_vertex(&mut self, key: VertexCacheKey, old_value: Option<CachedVertex>) {
        self.entries
            .push(CacheSnapshotEntry::Vertex(key, old_value));
    }

    pub fn record_id_index(&mut self, key: IdIndexCacheKey, old_value: Option<u32>) {
        self.entries
            .push(CacheSnapshotEntry::IdIndex(key, old_value));
    }

    pub fn into_entries(self) -> Vec<CacheSnapshotEntry> {
        self.entries
    }
}
