//! Index Type Classification
//!
//! This module defines the classification of index types in the graph database.
//! Indexes are categorized into two main types:
//!
//! ## Primary Indexes (CSR-Aware)
//!
//! Primary indexes are tightly coupled with the CSR (Compressed Sparse Row) storage structure.
//! They provide fast access to data by internal IDs and are automatically maintained.
//!
//! - `EdgeIdIndex`: Maps edge_id -> (src, dst, prop_offset)
//! - `DegreeIndex`: Maps vertex_id -> (out_degree, in_degree)
//!
//! ## Secondary Indexes (Property Indexes)
//!
//! Secondary indexes are built on property values and support complex queries.
//! They are decoupled from the CSR structure and use BTreeMap for storage.
//!
//! - `VertexIndexManager`: Index on vertex properties
//! - `EdgeIndexManager`: Index on edge properties

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::Timestamp;

pub type VertexId = u64;
pub type EdgeId = u64;
pub type PropOffset = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexCategory {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryIndexType {
    EdgeId,
    Degree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecondaryIndexType {
    VertexProperty,
    EdgeProperty,
}

pub trait PrimaryIndex: Send + Sync {
    fn category(&self) -> IndexCategory {
        IndexCategory::Primary
    }

    fn index_name(&self) -> &str;

    fn entry_count(&self) -> usize;

    fn clear(&self);

    fn memory_usage(&self) -> usize;
}

pub trait SecondaryIndex: Send + Sync {
    fn category(&self) -> IndexCategory {
        IndexCategory::Secondary
    }

    fn index_name(&self) -> &str;

    fn space_id(&self) -> u64;

    fn entry_count(&self) -> usize;

    fn clear(&self);

    fn memory_usage(&self) -> usize;

    fn gc_tombstones(&self, safe_ts: Timestamp) -> StorageResult<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexKey {
    pub space_id: u64,
    pub index_type: IndexKeyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKeyType {
    VertexForward,
    VertexReverse,
    EdgeForward,
    EdgeReverse,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub category: IndexCategory,
    pub name: String,
    pub entry_count: usize,
    pub memory_usage: usize,
    pub tombstone_count: usize,
}

impl IndexStats {
    pub fn new(
        category: IndexCategory,
        name: String,
        entry_count: usize,
        memory_usage: usize,
        tombstone_count: usize,
    ) -> Self {
        Self {
            category,
            name,
            entry_count,
            memory_usage,
            tombstone_count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompositeIndexStats {
    pub primary_indexes: Vec<IndexStats>,
    pub secondary_indexes: Vec<IndexStats>,
}

impl CompositeIndexStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_entries(&self) -> usize {
        let primary: usize = self.primary_indexes.iter().map(|s| s.entry_count).sum();
        let secondary: usize = self.secondary_indexes.iter().map(|s| s.entry_count).sum();
        primary + secondary
    }

    pub fn total_memory(&self) -> usize {
        let primary: usize = self.primary_indexes.iter().map(|s| s.memory_usage).sum();
        let secondary: usize = self.secondary_indexes.iter().map(|s| s.memory_usage).sum();
        primary + secondary
    }

    pub fn total_tombstones(&self) -> usize {
        let primary: usize = self.primary_indexes.iter().map(|s| s.tombstone_count).sum();
        let secondary: usize = self
            .secondary_indexes
            .iter()
            .map(|s| s.tombstone_count)
            .sum();
        primary + secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_category() {
        assert_eq!(IndexCategory::Primary, IndexCategory::Primary);
        assert_ne!(IndexCategory::Primary, IndexCategory::Secondary);
    }

    #[test]
    fn test_composite_stats() {
        let mut stats = CompositeIndexStats::new();
        stats.primary_indexes.push(IndexStats::new(
            IndexCategory::Primary,
            "edge_id".to_string(),
            100,
            1024,
            0,
        ));
        stats.secondary_indexes.push(IndexStats::new(
            IndexCategory::Secondary,
            "vertex_name".to_string(),
            50,
            512,
            5,
        ));

        assert_eq!(stats.total_entries(), 150);
        assert_eq!(stats.total_memory(), 1536);
        assert_eq!(stats.total_tombstones(), 5);
    }
}
