//! Edge ID Index for CSR
//!
//! A **Primary Index** that provides fast lookup of edge by edge_id.
//! This is a CSR-aware index that uses native VertexId and EdgeId types.
//!
//! ## Index Category: Primary
//!
//! This index is tightly coupled with CSR storage:
//! - Maps `edge_id -> (src, dst, prop_offset)`
//! - Automatically maintained during edge insert/delete
//! - No MVCC overhead (always consistent with CSR)

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::types::{EdgeId, VertexId};
use crate::storage::index::index_types::PropOffset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeLocation {
    pub src: VertexId,
    pub dst: VertexId,
    pub prop_offset: PropOffset,
}

impl EdgeLocation {
    pub fn new(src: VertexId, dst: VertexId, prop_offset: PropOffset) -> Self {
        Self {
            src,
            dst,
            prop_offset,
        }
    }
}

#[derive(Debug)]
pub struct EdgeIdIndex {
    index: DashMap<EdgeId, EdgeLocation>,
    edge_count: AtomicU64,
}

impl Default for EdgeIdIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeIdIndex {
    pub fn new() -> Self {
        Self {
            index: DashMap::new(),
            edge_count: AtomicU64::new(0),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: DashMap::with_capacity(capacity),
            edge_count: AtomicU64::new(0),
        }
    }

    pub fn insert(&self, edge_id: EdgeId, src: VertexId, dst: VertexId, prop_offset: PropOffset) {
        let location = EdgeLocation::new(src, dst, prop_offset);
        self.index.insert(edge_id, location);
        self.edge_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self, edge_id: EdgeId) -> Option<EdgeLocation> {
        self.index.get(&edge_id).map(|v| v.clone())
    }

    pub fn remove(&self, edge_id: EdgeId) -> Option<EdgeLocation> {
        let result = self.index.remove(&edge_id).map(|(_, v)| v);
        if result.is_some() {
            self.edge_count.fetch_sub(1, Ordering::Relaxed);
        }
        result
    }

    pub fn contains(&self, edge_id: EdgeId) -> bool {
        self.index.contains_key(&edge_id)
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        self.index.clear();
        self.edge_count.store(0, Ordering::Relaxed);
    }

    pub fn update_prop_offset(&self, edge_id: EdgeId, new_prop_offset: PropOffset) -> bool {
        if let Some(mut entry) = self.index.get_mut(&edge_id) {
            entry.prop_offset = new_prop_offset;
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (EdgeId, EdgeLocation)> + '_ {
        self.index
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
    }

    pub fn entries_by_src(&self, src: VertexId) -> Vec<(EdgeId, EdgeLocation)> {
        self.index
            .iter()
            .filter(|entry| entry.value().src == src)
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    pub fn entries_by_dst(&self, dst: VertexId) -> Vec<(EdgeId, EdgeLocation)> {
        self.index
            .iter()
            .filter(|entry| entry.value().dst == dst)
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    pub fn index_name(&self) -> &str {
        "edge_id_index"
    }

    pub fn entry_count(&self) -> usize {
        self.len()
    }

    pub fn memory_usage(&self) -> usize {
        self.index.len() * (std::mem::size_of::<EdgeId>() + std::mem::size_of::<EdgeLocation>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let index = EdgeIdIndex::new();

        index.insert(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        index.insert(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);
        index.insert(3, VertexId::from_u64(200), VertexId::from_u64(300), 30);

        assert_eq!(index.len(), 3);
        assert_eq!(index.edge_count(), 3);

        let loc = index.get(1).expect("Should find edge 1");
        assert_eq!(loc.src, VertexId::from_u64(100));
        assert_eq!(loc.dst, VertexId::from_u64(200));
        assert_eq!(loc.prop_offset, 10);

        let loc = index.get(2).expect("Should find edge 2");
        assert_eq!(loc.src, VertexId::from_u64(100));
        assert_eq!(loc.dst, VertexId::from_u64(300));

        assert!(index.contains(1));
        assert!(!index.contains(999));
    }

    #[test]
    fn test_remove() {
        let index = EdgeIdIndex::new();

        index.insert(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        assert_eq!(index.len(), 1);

        let removed = index.remove(1).expect("Should remove edge 1");
        assert_eq!(removed.src, VertexId::from_u64(100));
        assert_eq!(removed.dst, VertexId::from_u64(200));

        assert_eq!(index.len(), 0);
        assert!(index.get(1).is_none());
    }

    #[test]
    fn test_update_prop_offset() {
        let index = EdgeIdIndex::new();

        index.insert(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);

        let updated = index.update_prop_offset(1, 99);
        assert!(updated);

        let loc = index.get(1).expect("Should find edge 1");
        assert_eq!(loc.prop_offset, 99);

        let not_updated = index.update_prop_offset(999, 50);
        assert!(!not_updated);
    }

    #[test]
    fn test_entries_by_src_dst() {
        let index = EdgeIdIndex::new();

        index.insert(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        index.insert(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);
        index.insert(3, VertexId::from_u64(200), VertexId::from_u64(300), 30);

        let src_entries = index.entries_by_src(VertexId::from_u64(100));
        assert_eq!(src_entries.len(), 2);

        let dst_entries = index.entries_by_dst(VertexId::from_u64(300));
        assert_eq!(dst_entries.len(), 2);
    }

    #[test]
    fn test_clear() {
        let index = EdgeIdIndex::new();

        index.insert(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        index.insert(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);

        assert_eq!(index.len(), 2);

        index.clear();

        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
        assert_eq!(index.edge_count(), 0);
    }
}
