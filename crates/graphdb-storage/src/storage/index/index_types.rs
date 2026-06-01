//! Index Type Classification
//!
//! This module defines the classification of index types in the graph database.
//! This module defines the common types for property indexes in the graph database.
//!
//! Secondary indexes are built on property values and support complex queries.
//! They are decoupled from the CSR structure and use BTreeMap for storage.
//!
//! - `VertexIndexManager`: Index on vertex properties
//! - `EdgeIndexManager`: Index on edge properties

use crate::core::types::Timestamp;
use crate::core::StorageResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexCategory {
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexEstimate {
    pub total_entries: usize,
    pub visible_entries: usize,
    pub tombstone_entries: usize,
}

impl IndexEstimate {
    pub fn new(total_entries: usize, visible_entries: usize, tombstone_entries: usize) -> Self {
        Self {
            total_entries,
            visible_entries,
            tombstone_entries,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_entries == 0
    }
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
    pub secondary_indexes: Vec<IndexStats>,
}

impl CompositeIndexStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_entries(&self) -> usize {
        self.secondary_indexes.iter().map(|s| s.entry_count).sum()
    }

    pub fn total_memory(&self) -> usize {
        self.secondary_indexes.iter().map(|s| s.memory_usage).sum()
    }

    pub fn total_tombstones(&self) -> usize {
        self.secondary_indexes
            .iter()
            .map(|s| s.tombstone_count)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_category() {
        assert_eq!(IndexCategory::Secondary, IndexCategory::Secondary);
    }

    #[test]
    fn test_composite_stats() {
        let mut stats = CompositeIndexStats::new();
        stats.secondary_indexes.push(IndexStats::new(
            IndexCategory::Secondary,
            "vertex_name".to_string(),
            50,
            512,
            5,
        ));

        assert_eq!(stats.total_entries(), 50);
        assert_eq!(stats.total_memory(), 512);
        assert_eq!(stats.total_tombstones(), 5);
    }
}
