//! Weighted CSR Implementation
//!
//! This module provides a CSR implementation with edge weight support.
//! Weights are stored in a separate columnar array for efficient access
//! and SIMD optimization potential.
//!
//! ## Design Note
//!
//! `WeightedCsr` is designed as a specialized CSR for graphs with edge weights
//! (e.g., social networks with relationship strength, road networks with distances).
//! It wraps a `MutableCsr` and maintains a parallel weight array.
//!
//! ## When to Use
//!
//! - Edge weights are frequently accessed or updated
//! - Weight-based graph algorithms (shortest path, PageRank, etc.)
//! - Need O(1) weight access without property table lookup
//!
//! ## Integration with EdgeTable
//!
//! Currently, `WeightedCsr` is not integrated into `EdgeTable`. To use weighted edges:
//!
//! Option 1: Store weights as edge properties in `PropertyTable` (simpler, works with all CSR types)
//! Option 2: Extend `EdgeSchema` with `has_weight: bool` flag and use `WeightedCsr` (faster weight access)
//!
//! ## Limitations
//!
//! - Fixed to `MutableCsr`, cannot combine with `SingleMutableCsr` or `CacheOptimizedCsr`
//! - Not selectable via `EdgeStrategy` enum

use std::sync::atomic::{AtomicU64, Ordering};

use super::{CsrBase, CsrType, EdgeId, MutableCsrTrait, Nbr, Timestamp, VertexId, INVALID_TIMESTAMP};
use super::mutable_csr::MutableCsr;

/// CSR with edge weights stored in columnar format
///
/// This structure wraps a standard `MutableCsr` and adds a parallel array
/// for edge weights. This design provides:
/// - O(1) weight access
/// - Cache-friendly weight storage
/// - SIMD optimization potential
/// - Optional weights (not all edges need weights)
pub struct WeightedCsr {
    csr: MutableCsr,
    weights: Vec<f64>,
    edge_count: AtomicU64,
}

impl std::fmt::Debug for WeightedCsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeightedCsr")
            .field("csr", &self.csr)
            .field("weight_count", &self.weights.len())
            .field("edge_count", &self.edge_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl WeightedCsr {
    /// Create a new weighted CSR
    pub fn new() -> Self {
        Self {
            csr: MutableCsr::new(),
            weights: Vec::new(),
            edge_count: AtomicU64::new(0),
        }
    }

    /// Create a new weighted CSR with specified capacities
    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        Self {
            csr: MutableCsr::with_capacity(vertex_capacity, edge_capacity),
            weights: Vec::with_capacity(edge_capacity),
            edge_count: AtomicU64::new(0),
        }
    }

    /// Insert a weighted edge
    pub fn insert_weighted_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
        weight: f64,
    ) -> bool {
        let src_idx = src as usize;
        
        // Get the position where the edge will be inserted
        let insert_position = self.get_insert_position(src);
        
        if self.csr.insert_edge(src, dst, edge_id, prop_offset, ts) {
            // Ensure weights array is large enough
            if insert_position >= self.weights.len() {
                self.weights.resize(insert_position + 1, 0.0);
            }
            self.weights[insert_position] = weight;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Get the insert position for a new edge
    fn get_insert_position(&self, src: VertexId) -> usize {
        let src_idx = src as usize;
        if src_idx >= self.csr.get_vertex_capacity() {
            return 0;
        }

        let degree = self.csr.degree(src, u32::MAX) as usize;
        let offset = self.csr.get_adj_offsets()[src_idx];
        offset + degree
    }

    /// Find the offset of an edge in the neighbor list
    fn find_edge_offset(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<usize> {
        let src_idx = src as usize;
        if src_idx >= self.csr.get_vertex_capacity() {
            return None;
        }

        let degree = self.csr.get_degrees()[src_idx] as usize;
        let offset = self.csr.get_adj_offsets()[src_idx];

        for i in 0..degree {
            let nbr_offset = offset + i;
            if nbr_offset >= self.csr.get_nbr_list().len() {
                break;
            }
            let nbr = &self.csr.get_nbr_list()[nbr_offset];
            if nbr.neighbor == dst
                && nbr.timestamp <= ts
                && nbr.timestamp != INVALID_TIMESTAMP
            {
                return Some(nbr_offset);
            }
        }
        None
    }

    /// Get the weight of an edge
    pub fn get_weight(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<f64> {
        self.find_edge_offset(src, dst, ts)
            .and_then(|offset| self.weights.get(offset).copied())
    }

    /// Get the weight of an edge by edge_id
    pub fn get_weight_by_edge_id(&self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> Option<f64> {
        let src_idx = src as usize;
        if src_idx >= self.csr.get_vertex_capacity() {
            return None;
        }

        let degree = self.csr.get_degrees()[src_idx] as usize;
        let offset = self.csr.get_adj_offsets()[src_idx];

        for i in 0..degree {
            let nbr_offset = offset + i;
            if nbr_offset >= self.csr.get_nbr_list().len() {
                break;
            }
            let nbr = &self.csr.get_nbr_list()[nbr_offset];
            if nbr.edge_id == edge_id
                && nbr.timestamp <= ts
                && nbr.timestamp != INVALID_TIMESTAMP
            {
                return self.weights.get(nbr_offset).copied();
            }
        }
        None
    }

    /// Update the weight of an edge
    pub fn update_weight(&mut self, src: VertexId, dst: VertexId, weight: f64, ts: Timestamp) -> bool {
        if let Some(offset) = self.find_edge_offset(src, dst, ts) {
            if offset < self.weights.len() {
                self.weights[offset] = weight;
                return true;
            }
        }
        false
    }

    /// Get edges with their weights
    pub fn edges_with_weights(&self, src: VertexId, ts: Timestamp) -> Vec<(Nbr, f64)> {
        let edges = self.csr.edges_of(src, ts);
        edges
            .into_iter()
            .filter_map(|nbr| {
                let offset = self.find_edge_offset(src, nbr.neighbor, ts)?;
                let weight = self.weights.get(offset).copied().unwrap_or(0.0);
                Some((nbr, weight))
            })
            .collect()
    }

    /// Get all weights for a vertex's edges
    pub fn weights_of(&self, src: VertexId, ts: Timestamp) -> Vec<f64> {
        let src_idx = src as usize;
        if src_idx >= self.csr.get_vertex_capacity() {
            return Vec::new();
        }

        let degree = self.csr.get_degrees()[src_idx] as usize;
        let offset = self.csr.get_adj_offsets()[src_idx];

        let mut result = Vec::with_capacity(degree);
        for i in 0..degree {
            let nbr_offset = offset + i;
            if nbr_offset >= self.csr.get_nbr_list().len() {
                break;
            }
            let nbr = &self.csr.get_nbr_list()[nbr_offset];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                let weight = self.weights.get(nbr_offset).copied().unwrap_or(0.0);
                result.push(weight);
            }
        }
        result
    }

    /// Batch insert weighted edges
    pub fn batch_insert_weighted_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        weights: &[f64],
        ts: Timestamp,
    ) {
        assert_eq!(src_list.len(), dst_list.len());
        assert_eq!(src_list.len(), edge_ids.len());
        assert_eq!(src_list.len(), prop_offsets.len());
        assert_eq!(src_list.len(), weights.len());

        for i in 0..src_list.len() {
            self.insert_weighted_edge(
                src_list[i],
                dst_list[i],
                edge_ids[i],
                prop_offsets[i],
                ts,
                weights[i],
            );
        }
    }

    /// Delete a weighted edge
    pub fn delete_weighted_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        // Note: We don't actually remove the weight, just mark the edge as deleted
        // The weight will be cleaned up during compaction
        self.csr.delete_edge(src, edge_id, ts)
    }

    /// Compact: remove deleted edges and their weights
    pub fn compact(&mut self) {
        // First, compact the underlying CSR
        self.csr.compact();

        // Then, rebuild the weights array to match the compacted CSR
        let mut new_weights = Vec::with_capacity(self.weights.len());
        
        for src_idx in 0..self.csr.get_vertex_capacity() {
            let degree = self.csr.get_degrees()[src_idx] as usize;
            let offset = self.csr.get_adj_offsets()[src_idx];

            for i in 0..degree {
                let nbr_offset = offset + i;
                if nbr_offset < self.weights.len() {
                    new_weights.push(self.weights[nbr_offset]);
                }
            }
        }

        self.weights = new_weights;
    }
}

impl Default for WeightedCsr {
    fn default() -> Self {
        Self::new()
    }
}

impl CsrBase for WeightedCsr {
    fn vertex_capacity(&self) -> usize {
        self.csr.vertex_capacity()
    }

    fn edge_count(&self) -> u64 {
        self.csr.edge_count()
    }

    fn csr_type(&self) -> CsrType {
        CsrType::Mutable
    }

    fn resize(&mut self, new_vertex_capacity: usize) {
        self.csr.resize(new_vertex_capacity);
    }

    fn clear(&mut self) {
        self.csr.clear();
        self.weights.clear();
        self.edge_count.store(0, Ordering::Relaxed);
    }

    fn dump(&self) -> Vec<u8> {
        let mut result = self.csr.dump();
        
        // Append weights
        result.extend_from_slice(&(self.weights.len() as u64).to_le_bytes());
        for &weight in &self.weights {
            result.extend_from_slice(&weight.to_le_bytes());
        }
        
        result
    }

    fn load(&mut self, data: &[u8]) {
        // Find where weights start
        // The CSR data comes first, then weights
        // We need to parse the CSR to find the boundary
        
        // For simplicity, we'll use a different format:
        // [csr_length: u64][csr_data][weights_length: u64][weights_data]
        
        if data.len() < 8 {
            return;
        }
        
        let csr_length = u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])) as usize;
        if csr_length + 16 > data.len() {
            return;
        }
        
        self.csr.load(&data[8..8 + csr_length]);
        
        let weights_offset = 8 + csr_length;
        let weights_length = u64::from_le_bytes(
            data[weights_offset..weights_offset + 8].try_into().unwrap_or([0; 8])
        ) as usize;
        
        if weights_offset + 8 + weights_length * 8 > data.len() {
            return;
        }
        
        let mut weights = Vec::with_capacity(weights_length);
        let mut offset = weights_offset + 8;
        for _ in 0..weights_length {
            let weight = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            weights.push(weight);
            offset += 8;
        }
        
        self.weights = weights;
    }
}

impl MutableCsrTrait for WeightedCsr {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        // Insert with default weight of 0.0
        self.insert_weighted_edge(src, dst, edge_id, prop_offset, ts, 0.0)
    }

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        self.delete_weighted_edge(src, edge_id, ts)
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        // Find edge_id first
        if let Some(nbr) = self.csr.get_edge(src, dst, ts) {
            self.delete_weighted_edge(src, nbr.edge_id, ts)
        } else {
            false
        }
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        self.csr.delete_edge_by_offset(src, offset, ts)
    }

    fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        self.csr.revert_delete(src, edge_id, ts)
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        self.csr.revert_delete_by_offset(src, offset, ts)
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        self.csr.get_edge(src, dst, ts)
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        self.csr.edges_of(src, ts)
    }

    fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        self.csr.degree(src, ts)
    }

    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        self.csr.has_edge(src, dst, ts)
    }

    fn compact(&mut self) {
        self.compact();
    }

    fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        // Insert with default weights
        let weights = vec![0.0; src_list.len()];
        self.batch_insert_weighted_edges(src_list, dst_list, edge_ids, prop_offsets, &weights, ts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_operations() {
        let mut csr = WeightedCsr::new();

        // Insert weighted edges
        assert!(csr.insert_weighted_edge(0, 1, 100, 0, 10, 1.5));
        assert!(csr.insert_weighted_edge(0, 2, 101, 1, 10, 2.5));
        assert!(csr.insert_weighted_edge(1, 2, 102, 2, 10, 3.5));

        // Check edge count
        assert_eq!(csr.edge_count(), 3);

        // Get weights
        assert_eq!(csr.get_weight(0, 1, 10), Some(1.5));
        assert_eq!(csr.get_weight(0, 2, 10), Some(2.5));
        assert_eq!(csr.get_weight(1, 2, 10), Some(3.5));

        // Get edges with weights
        let edges_with_weights = csr.edges_with_weights(0, 10);
        assert_eq!(edges_with_weights.len(), 2);

        // Update weight
        assert!(csr.update_weight(0, 1, 5.0, 10));
        assert_eq!(csr.get_weight(0, 1, 10), Some(5.0));
    }

    #[test]
    fn test_weighted_delete() {
        let mut csr = WeightedCsr::new();

        csr.insert_weighted_edge(0, 1, 100, 0, 10, 1.5);
        csr.insert_weighted_edge(0, 2, 101, 1, 10, 2.5);

        // Delete edge
        assert!(csr.delete_weighted_edge(0, 100, 20));
        assert_eq!(csr.edge_count(), 1);

        // Weight should not be accessible
        assert_eq!(csr.get_weight(0, 1, 20), None);
    }

    #[test]
    fn test_batch_insert() {
        let mut csr = WeightedCsr::new();

        let src_list = vec![0, 1, 2];
        let dst_list = vec![1, 2, 3];
        let edge_ids = vec![100, 101, 102];
        let prop_offsets = vec![0, 1, 2];
        let weights = vec![1.0, 2.0, 3.0];

        csr.batch_insert_weighted_edges(&src_list, &dst_list, &edge_ids, &prop_offsets, &weights, 10);

        assert_eq!(csr.edge_count(), 3);
        assert_eq!(csr.get_weight(0, 1, 10), Some(1.0));
        assert_eq!(csr.get_weight(1, 2, 10), Some(2.0));
        assert_eq!(csr.get_weight(2, 3, 10), Some(3.0));
    }
}
