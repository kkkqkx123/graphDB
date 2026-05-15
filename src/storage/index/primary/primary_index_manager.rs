//! Primary Index Manager
//!
//! Unified management for CSR-aware primary indexes.
//! Primary indexes are tightly coupled with CSR storage structure and provide
//! fast access to data by internal IDs.
//!
//! ## Managed Indexes
//!
//! - `EdgeIdIndex`: Maps edge_id -> (src, dst, prop_offset)
//! - `DegreeIndex`: Maps vertex_id -> (out_degree, in_degree)
//!
//! ## Characteristics
//!
//! - Native ID types (u64) for maximum performance
//! - No MVCC overhead (always consistent with CSR)
//! - Automatically maintained during DML operations
//! - Thread-safe with DashMap for concurrent access

use super::degree_index::{DegreeInfo, DegreeIndex};
use super::edge_id_index::{EdgeLocation, EdgeIdIndex};
use crate::core::types::{EdgeId, VertexId};
use crate::storage::index::index_types::{CompositeIndexStats, IndexCategory, IndexStats, PrimaryIndex, PropOffset};

#[derive(Debug)]
pub struct PrimaryIndexManager {
    edge_id_index: EdgeIdIndex,
    degree_index: DegreeIndex,
}

impl Default for PrimaryIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimaryIndexManager {
    pub fn new() -> Self {
        Self {
            edge_id_index: EdgeIdIndex::new(),
            degree_index: DegreeIndex::new(),
        }
    }

    pub fn with_capacity(edge_capacity: usize, vertex_capacity: usize) -> Self {
        Self {
            edge_id_index: EdgeIdIndex::with_capacity(edge_capacity),
            degree_index: DegreeIndex::with_capacity(vertex_capacity),
        }
    }

    // ========================================================================
    // Edge ID Index Operations
    // ========================================================================

    pub fn insert_edge(
        &self,
        edge_id: EdgeId,
        src: VertexId,
        dst: VertexId,
        prop_offset: PropOffset,
    ) {
        self.edge_id_index.insert(edge_id, src, dst, prop_offset);
        self.degree_index.insert_edge(src, dst);
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> Option<EdgeLocation> {
        self.edge_id_index.get(edge_id)
    }

    pub fn remove_edge(&self, edge_id: EdgeId) -> Option<EdgeLocation> {
        let location = self.edge_id_index.remove(edge_id)?;
        self.degree_index.remove_edge(location.src, location.dst);
        Some(location)
    }

    pub fn contains_edge(&self, edge_id: EdgeId) -> bool {
        self.edge_id_index.contains(edge_id)
    }

    pub fn update_edge_prop_offset(&self, edge_id: EdgeId, new_prop_offset: PropOffset) -> bool {
        self.edge_id_index.update_prop_offset(edge_id, new_prop_offset)
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_id_index.edge_count()
    }

    pub fn edges_by_src(&self, src: VertexId) -> Vec<(EdgeId, EdgeLocation)> {
        self.edge_id_index.entries_by_src(src)
    }

    pub fn edges_by_dst(&self, dst: VertexId) -> Vec<(EdgeId, EdgeLocation)> {
        self.edge_id_index.entries_by_dst(dst)
    }

    // ========================================================================
    // Degree Index Operations
    // ========================================================================

    pub fn get_degree(&self, vertex_id: VertexId) -> Option<DegreeInfo> {
        self.degree_index.get(vertex_id)
    }

    pub fn out_degree(&self, vertex_id: VertexId) -> u32 {
        self.degree_index.out_degree(vertex_id)
    }

    pub fn in_degree(&self, vertex_id: VertexId) -> u32 {
        self.degree_index.in_degree(vertex_id)
    }

    pub fn total_degree(&self, vertex_id: VertexId) -> u64 {
        self.degree_index.total_degree(vertex_id)
    }

    pub fn total_out_edges(&self) -> u64 {
        self.degree_index.total_out_edges()
    }

    pub fn total_in_edges(&self) -> u64 {
        self.degree_index.total_in_edges()
    }

    pub fn total_edges(&self) -> u64 {
        self.degree_index.total_edges()
    }

    pub fn vertices_with_out_degree_at_least(&self, min_degree: u32) -> Vec<VertexId> {
        self.degree_index.vertices_with_out_degree_at_least(min_degree)
    }

    pub fn vertices_with_in_degree_at_least(&self, min_degree: u32) -> Vec<VertexId> {
        self.degree_index.vertices_with_in_degree_at_least(min_degree)
    }

    pub fn isolated_vertices(&self) -> Vec<VertexId> {
        self.degree_index.isolated_vertices()
    }

    // ========================================================================
    // Vertex Operations
    // ========================================================================

    pub fn remove_vertex(&self, vertex_id: VertexId) -> Option<DegreeInfo> {
        self.degree_index.remove_vertex(vertex_id)
    }

    pub fn vertex_count(&self) -> usize {
        self.degree_index.len()
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    pub fn clear(&self) {
        self.edge_id_index.clear();
        self.degree_index.clear();
    }

    pub fn clear_edge_index(&self) {
        self.edge_id_index.clear();
    }

    pub fn clear_degree_index(&self) {
        self.degree_index.clear();
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    pub fn stats(&self) -> CompositeIndexStats {
        CompositeIndexStats {
            primary_indexes: vec![
                IndexStats::new(
                    IndexCategory::Primary,
                    self.edge_id_index.index_name().to_string(),
                    self.edge_id_index.entry_count(),
                    self.edge_id_index.memory_usage(),
                    0,
                ),
                IndexStats::new(
                    IndexCategory::Primary,
                    self.degree_index.index_name().to_string(),
                    self.degree_index.entry_count(),
                    self.degree_index.memory_usage(),
                    0,
                ),
            ],
            secondary_indexes: Vec::new(),
        }
    }

    pub fn total_memory_usage(&self) -> usize {
        self.edge_id_index.memory_usage() + self.degree_index.memory_usage()
    }

    pub fn total_entry_count(&self) -> usize {
        self.edge_id_index.entry_count() + self.degree_index.entry_count()
    }

    // ========================================================================
    // Direct Access (for advanced use cases)
    // ========================================================================

    pub fn edge_id_index(&self) -> &EdgeIdIndex {
        &self.edge_id_index
    }

    pub fn degree_index(&self) -> &DegreeIndex {
        &self.degree_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let manager = PrimaryIndexManager::new();

        manager.insert_edge(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        manager.insert_edge(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);
        manager.insert_edge(3, VertexId::from_u64(200), VertexId::from_u64(300), 30);

        assert_eq!(manager.edge_count(), 3);
        assert_eq!(manager.vertex_count(), 3);

        let loc = manager.get_edge(1).expect("Should find edge 1");
        assert_eq!(loc.src, VertexId::from_u64(100));
        assert_eq!(loc.dst, VertexId::from_u64(200));
        assert_eq!(loc.prop_offset, 10);

        assert_eq!(manager.out_degree(VertexId::from_u64(100)), 2);
        assert_eq!(manager.in_degree(VertexId::from_u64(300)), 2);
    }

    #[test]
    fn test_remove_edge() {
        let manager = PrimaryIndexManager::new();

        manager.insert_edge(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        manager.insert_edge(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);

        assert_eq!(manager.edge_count(), 2);
        assert_eq!(manager.out_degree(VertexId::from_u64(100)), 2);

        let removed = manager.remove_edge(1).expect("Should remove edge 1");
        assert_eq!(removed.src, VertexId::from_u64(100));
        assert_eq!(removed.dst, VertexId::from_u64(200));

        assert_eq!(manager.edge_count(), 1);
        assert_eq!(manager.out_degree(VertexId::from_u64(100)), 1);
        assert_eq!(manager.in_degree(VertexId::from_u64(200)), 0);
    }

    #[test]
    fn test_stats() {
        let manager = PrimaryIndexManager::new();

        manager.insert_edge(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        manager.insert_edge(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);

        let stats = manager.stats();
        assert_eq!(stats.total_entries(), 5);
        assert_eq!(stats.primary_indexes.len(), 2);
    }

    #[test]
    fn test_clear() {
        let manager = PrimaryIndexManager::new();

        manager.insert_edge(1, VertexId::from_u64(100), VertexId::from_u64(200), 10);
        manager.insert_edge(2, VertexId::from_u64(100), VertexId::from_u64(300), 20);

        assert_eq!(manager.edge_count(), 2);

        manager.clear();

        assert_eq!(manager.edge_count(), 0);
        assert_eq!(manager.vertex_count(), 0);
    }
}
