//! Vertex Degree Index for CSR
//!
//! A **Primary Index** that provides fast degree queries and degree-based filtering.
//! This is a CSR-aware index that uses native VertexId type.
//!
//! ## Index Category: Primary
//!
//! This index is tightly coupled with CSR storage:
//! - Maps `vertex_id -> (out_degree, in_degree)`
//! - Automatically maintained during edge insert/delete
//! - No MVCC overhead (always consistent with CSR)

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::storage::index::index_types::{PrimaryIndex, VertexId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DegreeInfo {
    pub out_degree: u32,
    pub in_degree: u32,
}

impl DegreeInfo {
    pub fn new(out_degree: u32, in_degree: u32) -> Self {
        Self {
            out_degree,
            in_degree,
        }
    }

    pub fn total_degree(&self) -> u64 {
        self.out_degree as u64 + self.in_degree as u64
    }

    pub fn is_isolated(&self) -> bool {
        self.out_degree == 0 && self.in_degree == 0
    }
}

#[derive(Debug)]
pub struct DegreeIndex {
    degrees: DashMap<VertexId, DegreeInfo>,
    total_out_edges: AtomicU64,
    total_in_edges: AtomicU64,
}

impl Default for DegreeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl DegreeIndex {
    pub fn new() -> Self {
        Self {
            degrees: DashMap::new(),
            total_out_edges: AtomicU64::new(0),
            total_in_edges: AtomicU64::new(0),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            degrees: DashMap::with_capacity(capacity),
            total_out_edges: AtomicU64::new(0),
            total_in_edges: AtomicU64::new(0),
        }
    }

    pub fn get(&self, vertex_id: VertexId) -> Option<DegreeInfo> {
        self.degrees.get(&vertex_id).map(|v| *v)
    }

    pub fn out_degree(&self, vertex_id: VertexId) -> u32 {
        self.degrees
            .get(&vertex_id)
            .map(|v| v.out_degree)
            .unwrap_or(0)
    }

    pub fn in_degree(&self, vertex_id: VertexId) -> u32 {
        self.degrees
            .get(&vertex_id)
            .map(|v| v.in_degree)
            .unwrap_or(0)
    }

    pub fn total_degree(&self, vertex_id: VertexId) -> u64 {
        self.degrees
            .get(&vertex_id)
            .map(|v| v.total_degree())
            .unwrap_or(0)
    }

    pub fn increment_out_degree(&self, vertex_id: VertexId) -> u32 {
        let mut entry = self
            .degrees
            .entry(vertex_id)
            .or_insert_with(DegreeInfo::default);
        entry.out_degree += 1;
        self.total_out_edges.fetch_add(1, Ordering::Relaxed);
        entry.out_degree
    }

    pub fn increment_in_degree(&self, vertex_id: VertexId) -> u32 {
        let mut entry = self
            .degrees
            .entry(vertex_id)
            .or_insert_with(DegreeInfo::default);
        entry.in_degree += 1;
        self.total_in_edges.fetch_add(1, Ordering::Relaxed);
        entry.in_degree
    }

    pub fn decrement_out_degree(&self, vertex_id: VertexId) -> Option<u32> {
        if let Some(mut entry) = self.degrees.get_mut(&vertex_id) {
            if entry.out_degree > 0 {
                entry.out_degree -= 1;
                self.total_out_edges.fetch_sub(1, Ordering::Relaxed);
                return Some(entry.out_degree);
            }
        }
        None
    }

    pub fn decrement_in_degree(&self, vertex_id: VertexId) -> Option<u32> {
        if let Some(mut entry) = self.degrees.get_mut(&vertex_id) {
            if entry.in_degree > 0 {
                entry.in_degree -= 1;
                self.total_in_edges.fetch_sub(1, Ordering::Relaxed);
                return Some(entry.in_degree);
            }
        }
        None
    }

    pub fn insert_edge(&self, src: VertexId, dst: VertexId) {
        self.increment_out_degree(src);
        self.increment_in_degree(dst);
    }

    pub fn remove_edge(&self, src: VertexId, dst: VertexId) {
        self.decrement_out_degree(src);
        self.decrement_in_degree(dst);
    }

    pub fn set_degree(&self, vertex_id: VertexId, out_degree: u32, in_degree: u32) {
        let old = self
            .degrees
            .insert(vertex_id, DegreeInfo::new(out_degree, in_degree));
        if let Some(old_info) = old {
            let out_diff = out_degree as i64 - old_info.out_degree as i64;
            let in_diff = in_degree as i64 - old_info.in_degree as i64;

            if out_diff > 0 {
                self.total_out_edges
                    .fetch_add(out_diff as u64, Ordering::Relaxed);
            } else if out_diff < 0 {
                self.total_out_edges
                    .fetch_sub((-out_diff) as u64, Ordering::Relaxed);
            }

            if in_diff > 0 {
                self.total_in_edges
                    .fetch_add(in_diff as u64, Ordering::Relaxed);
            } else if in_diff < 0 {
                self.total_in_edges
                    .fetch_sub((-in_diff) as u64, Ordering::Relaxed);
            }
        } else {
            self.total_out_edges
                .fetch_add(out_degree as u64, Ordering::Relaxed);
            self.total_in_edges
                .fetch_add(in_degree as u64, Ordering::Relaxed);
        }
    }

    pub fn len(&self) -> usize {
        self.degrees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.degrees.is_empty()
    }

    pub fn total_out_edges(&self) -> u64 {
        self.total_out_edges.load(Ordering::Relaxed)
    }

    pub fn total_in_edges(&self) -> u64 {
        self.total_in_edges.load(Ordering::Relaxed)
    }

    pub fn total_edges(&self) -> u64 {
        self.total_out_edges() + self.total_in_edges()
    }

    pub fn clear(&self) {
        self.degrees.clear();
        self.total_out_edges.store(0, Ordering::Relaxed);
        self.total_in_edges.store(0, Ordering::Relaxed);
    }

    pub fn remove_vertex(&self, vertex_id: VertexId) -> Option<DegreeInfo> {
        let removed = self.degrees.remove(&vertex_id).map(|(_, v)| v);
        if let Some(info) = &removed {
            self.total_out_edges
                .fetch_sub(info.out_degree as u64, Ordering::Relaxed);
            self.total_in_edges
                .fetch_sub(info.in_degree as u64, Ordering::Relaxed);
        }
        removed
    }

    pub fn vertices_with_out_degree_at_least(&self, min_degree: u32) -> Vec<VertexId> {
        self.degrees
            .iter()
            .filter(|entry| entry.value().out_degree >= min_degree)
            .map(|entry| *entry.key())
            .collect()
    }

    pub fn vertices_with_in_degree_at_least(&self, min_degree: u32) -> Vec<VertexId> {
        self.degrees
            .iter()
            .filter(|entry| entry.value().in_degree >= min_degree)
            .map(|entry| *entry.key())
            .collect()
    }

    pub fn isolated_vertices(&self) -> Vec<VertexId> {
        self.degrees
            .iter()
            .filter(|entry| entry.value().is_isolated())
            .map(|entry| *entry.key())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (VertexId, DegreeInfo)> + '_ {
        self.degrees
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
    }
}

impl PrimaryIndex for DegreeIndex {
    fn index_name(&self) -> &str {
        "degree_index"
    }

    fn entry_count(&self) -> usize {
        self.degrees.len()
    }

    fn clear(&self) {
        self.degrees.clear();
        self.total_out_edges.store(0, Ordering::Relaxed);
        self.total_in_edges.store(0, Ordering::Relaxed);
    }

    fn memory_usage(&self) -> usize {
        self.degrees.len() * (std::mem::size_of::<VertexId>() + std::mem::size_of::<DegreeInfo>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let index = DegreeIndex::new();

        index.insert_edge(100, 200);
        index.insert_edge(100, 300);
        index.insert_edge(200, 300);

        assert_eq!(index.out_degree(100), 2);
        assert_eq!(index.in_degree(100), 0);
        assert_eq!(index.out_degree(200), 1);
        assert_eq!(index.in_degree(200), 1);
        assert_eq!(index.out_degree(300), 0);
        assert_eq!(index.in_degree(300), 2);

        assert_eq!(index.total_out_edges(), 3);
        assert_eq!(index.total_in_edges(), 3);
    }

    #[test]
    fn test_degree_info() {
        let info = DegreeInfo::new(5, 3);
        assert_eq!(info.out_degree, 5);
        assert_eq!(info.in_degree, 3);
        assert_eq!(info.total_degree(), 8);
        assert!(!info.is_isolated());

        let isolated = DegreeInfo::default();
        assert!(isolated.is_isolated());
    }

    #[test]
    fn test_remove_edge() {
        let index = DegreeIndex::new();

        index.insert_edge(100, 200);
        index.insert_edge(100, 300);

        assert_eq!(index.out_degree(100), 2);

        index.remove_edge(100, 200);

        assert_eq!(index.out_degree(100), 1);
        assert_eq!(index.in_degree(200), 0);
    }

    #[test]
    fn test_set_degree() {
        let index = DegreeIndex::new();

        index.set_degree(100, 5, 3);

        let info = index.get(100).expect("Should have degree info");
        assert_eq!(info.out_degree, 5);
        assert_eq!(info.in_degree, 3);

        index.set_degree(100, 10, 6);

        let info = index.get(100).expect("Should have degree info");
        assert_eq!(info.out_degree, 10);
        assert_eq!(info.in_degree, 6);
    }

    #[test]
    fn test_vertices_filtering() {
        let index = DegreeIndex::new();

        index.set_degree(100, 5, 0);
        index.set_degree(200, 3, 3);
        index.set_degree(300, 0, 5);

        let high_out = index.vertices_with_out_degree_at_least(4);
        assert_eq!(high_out.len(), 1);
        assert!(high_out.contains(&100));

        let high_in = index.vertices_with_in_degree_at_least(4);
        assert_eq!(high_in.len(), 1);
        assert!(high_in.contains(&300));
    }

    #[test]
    fn test_remove_vertex() {
        let index = DegreeIndex::new();

        index.set_degree(100, 5, 3);
        assert_eq!(index.len(), 1);

        let removed = index.remove_vertex(100).expect("Should remove vertex");
        assert_eq!(removed.out_degree, 5);
        assert_eq!(removed.in_degree, 3);

        assert_eq!(index.len(), 0);
        assert!(index.get(100).is_none());
    }

    #[test]
    fn test_clear() {
        let index = DegreeIndex::new();

        index.insert_edge(100, 200);
        index.insert_edge(200, 300);

        assert_eq!(index.len(), 3);

        index.clear();

        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
        assert_eq!(index.total_out_edges(), 0);
        assert_eq!(index.total_in_edges(), 0);
    }
}
