//! CSR Variant
//!
//! Enum wrapper for different CSR implementations (mutable).
//! Provides runtime polymorphism without dynamic dispatch (dyn).
//!
//! # CSR Type Selection
//!
//! The `EdgeStrategy` enum determines which CSR implementation to use:
//! - `Multiple`: Standard `MutableCsr` for general multi-edge scenarios
//! - `Single`: `SingleMutableCsr` for one-edge-per-vertex (O(1) access)
//! - `None`: No edges stored
//!
//! # Variants
//!
//! - `CsrVariant::Multiple`: Mutable CSR with dynamic capacity growth
//! - `CsrVariant::Single`: Mutable single-edge CSR
//! - `CsrVariant::MultiSingle`: Multi-single mutable CSR
//! - `CsrVariant::Labeled`: Label-aware mutable CSR
//! - `CsrVariant::None`: Placeholder for relationships with no edges

use crate::core::StorageResult;

use super::{
    CsrBase, EdgeId, EdgeStrategy, LabeledMutableCsr, LabeledMutableCsrIterator,
    MutableCsr, MutableCsrIterator, MutableCsrTrait, MultiSingleMutableCsr,
    MultiSingleMutableCsrIterator, Nbr, SingleMutableCsr, SingleMutableCsrIterator, Timestamp,
    VertexId,
};

/// Polymorphic CSR wrapper supporting multiple implementation strategies.
///
/// Combines mutable and immutable CSR implementations into a single enum,
/// allowing runtime selection without virtual function overhead.
///
/// # Example
///
/// ```ignore
/// // Create a multi-edge CSR
/// let csr = CsrVariant::from_strategy(EdgeStrategy::Multiple, 1000, 10000)?;
///
/// // Use the same interface for all variants
/// let edges = csr.edges_of(vertex_id, timestamp);
/// ```
#[derive(Debug, Clone)]
pub enum CsrVariant {
    /// Multi-edge mutable CSR: each vertex can have multiple outgoing edges
    Multiple(MutableCsr),
    /// Single-edge mutable CSR: each vertex has at most one outgoing edge
    Single(SingleMutableCsr),
    /// Multi-single mutable CSR: each vertex has multiple outgoing edges (limited by capacity)
    MultiSingle(MultiSingleMutableCsr),
    /// Label-aware mutable CSR: edges grouped by label for fast label-based queries
    Labeled(LabeledMutableCsr),
    /// No-edge placeholder: vertices exist but have no outgoing edges
    None { vertex_capacity: usize },
}

impl CsrVariant {
    /// Create a CSR from an edge strategy
    pub fn from_strategy(
        strategy: EdgeStrategy,
        vertex_capacity: usize,
        edge_capacity: usize,
    ) -> StorageResult<Self> {
        match strategy {
            EdgeStrategy::Multiple => Ok(CsrVariant::Multiple(MutableCsr::with_capacity(
                vertex_capacity,
                edge_capacity,
            ))),
            EdgeStrategy::Single => Ok(CsrVariant::Single(SingleMutableCsr::with_capacity(
                vertex_capacity,
            ))),
            EdgeStrategy::MultiSingle { max_edges } => {
                Ok(CsrVariant::MultiSingle(MultiSingleMutableCsr::with_capacity(
                    vertex_capacity,
                    max_edges,
                )))
            }
            EdgeStrategy::Labeled => Ok(CsrVariant::Labeled(LabeledMutableCsr::with_capacity(
                vertex_capacity,
                edge_capacity,
            ))),
            EdgeStrategy::None => Ok(CsrVariant::None { vertex_capacity }),
        }
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        match self {
            CsrVariant::None { .. } => {},
            CsrVariant::Multiple(csr) => csr.clear(),
            CsrVariant::Single(csr) => csr.clear(),
            CsrVariant::MultiSingle(csr) => csr.clear(),
            CsrVariant::Labeled(csr) => csr.clear(),
        }
    }

    /// Get fragmentation ratio for diagnostics
    ///
    /// Returns:
    /// - `Multiple(ratio)`: Fragmentation ratio of the CSR
    /// - `Single/MultiSingle/Labeled/None/Immutable`: 0.0 (no fragmentation)
    pub fn fragmentation_ratio(&self) -> f32 {
        match self {
            CsrVariant::Multiple(csr) => csr.fragmentation_ratio(),
            _ => 0.0,
        }
    }

    /// Estimate average bytes per edge for this CSR strategy.
    ///
    /// Used for merge heuristics to more accurately calculate segment sizes.
    /// These are empirical values derived from profiling.
    ///
    /// Returns estimated bytes per edge:
    /// - Multiple: ~26 bytes (fixed arrays, standard offsets)
    /// - Single: ~20 bytes (minimal metadata, single-per-vertex)
    /// - MultiSingle: ~28 bytes (capacity array overhead)
    /// - Labeled: ~36 bytes (label indexing adds overhead)
    /// - None: 0 bytes (no edges)
    pub fn bytes_per_edge(&self) -> usize {
        match self {
            CsrVariant::Multiple(_) => 26,
            CsrVariant::Single(_) => 20,
            CsrVariant::MultiSingle(_) => 28,
            CsrVariant::Labeled(_) => 36,
            CsrVariant::None { .. } => 0,
        }
    }
}

impl CsrBase for CsrVariant {
    fn vertex_capacity(&self) -> usize {
        match self {
            CsrVariant::None { vertex_capacity } => *vertex_capacity,
            CsrVariant::Multiple(csr) => csr.vertex_capacity(),
            CsrVariant::Single(csr) => csr.vertex_capacity(),
            CsrVariant::MultiSingle(csr) => csr.vertex_capacity(),
            CsrVariant::Labeled(csr) => csr.vertex_capacity(),
        }
    }

    fn edge_count(&self) -> u64 {
        match self {
            CsrVariant::None { .. } => 0,
            CsrVariant::Multiple(csr) => csr.edge_count(),
            CsrVariant::Single(csr) => csr.edge_count(),
            CsrVariant::MultiSingle(csr) => csr.edge_count(),
            CsrVariant::Labeled(csr) => csr.edge_count(),
        }
    }

    fn dump(&self) -> Vec<u8> {
        match self {
            CsrVariant::None { vertex_capacity } => {
                let mut result = vec![0u8];
                result.extend((*vertex_capacity as u64).to_le_bytes());
                result
            }
            CsrVariant::Multiple(csr) => {
                let mut result = vec![1u8];
                result.extend(csr.dump());
                result
            }
            CsrVariant::Single(csr) => {
                let mut result = vec![2u8];
                result.extend(csr.dump());
                result
            }
            CsrVariant::MultiSingle(csr) => {
                let mut result = vec![3u8];
                result.extend(csr.dump());
                result
            }
            CsrVariant::Labeled(csr) => {
                let mut result = vec![4u8];
                result.extend(csr.dump());
                result
            }
        }
    }

    fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.is_empty() {
            return Err(crate::core::StorageError::deserialize_error(
                "Cannot load CSR variant: empty data",
            ));
        }

        match data[0] {
            0 => {
                if data.len() < 9 {
                    return Err(crate::core::StorageError::deserialize_error(
                        "Cannot load None CSR variant: data too short",
                    ));
                }
                let vertex_capacity = u64::from_le_bytes([
                    data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
                ]) as usize;
                *self = CsrVariant::None { vertex_capacity };
                Ok(())
            }
            1 => {
                let mut csr = MutableCsr::new();
                csr.load(&data[1..])?;
                *self = CsrVariant::Multiple(csr);
                Ok(())
            }
            2 => {
                let mut csr = SingleMutableCsr::new();
                csr.load(&data[1..])?;
                *self = CsrVariant::Single(csr);
                Ok(())
            }
            3 => {
                let mut csr = MultiSingleMutableCsr::new();
                csr.load(&data[1..])?;
                *self = CsrVariant::MultiSingle(csr);
                Ok(())
            }
            4 => {
                let mut csr = LabeledMutableCsr::new();
                csr.load(&data[1..])?;
                *self = CsrVariant::Labeled(csr);
                Ok(())
            }
            _ => Err(crate::core::StorageError::deserialize_error(
                "Invalid CSR variant tag in serialized data",
            )),
        }
    }
}

impl MutableCsrTrait for CsrVariant {
    fn insert_edge(
        &mut self,
        src_vid: u32,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        match self {
            CsrVariant::None { .. } => false,
            CsrVariant::Multiple(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::Single(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::MultiSingle(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::Labeled(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
        }
    }

    fn delete_edge(&mut self, src_vid: u32, edge_id: EdgeId, ts: Timestamp) -> bool {
        match self {
            CsrVariant::None { .. } => false,
            CsrVariant::Multiple(csr) => csr.delete_edge(src_vid, edge_id, ts),
            CsrVariant::Single(csr) => csr.delete_edge(src_vid, edge_id, ts),
            CsrVariant::MultiSingle(csr) => csr.delete_edge(src_vid, edge_id, ts),
            CsrVariant::Labeled(csr) => csr.delete_edge(src_vid, edge_id, ts),
        }
    }

    fn delete_edge_by_dst(&mut self, src_vid: u32, dst: VertexId, ts: Timestamp) -> bool {
        match self {
            CsrVariant::None { .. } => false,
            CsrVariant::Multiple(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
            CsrVariant::Single(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
            CsrVariant::MultiSingle(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
            CsrVariant::Labeled(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
        }
    }

    fn delete_edge_by_offset(&mut self, src_vid: u32, offset: i32, ts: Timestamp) -> bool {
        match self {
            CsrVariant::None { .. } => false,
            CsrVariant::Multiple(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
            CsrVariant::Single(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
            CsrVariant::MultiSingle(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
            CsrVariant::Labeled(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
        }
    }

    fn revert_delete_by_offset(&mut self, src_vid: u32, offset: i32, ts: Timestamp) -> bool {
        match self {
            CsrVariant::None { .. } => false,
            CsrVariant::Multiple(csr) => {
                csr.revert_delete_by_offset(src_vid, offset, ts)
            }
            CsrVariant::Single(csr) => csr.revert_delete_by_offset(src_vid, offset, ts),
            CsrVariant::MultiSingle(csr) => csr.revert_delete_by_offset(src_vid, offset, ts),
            CsrVariant::Labeled(csr) => csr.revert_delete_by_offset(src_vid, offset, ts),
        }
    }

    fn get_edge(&self, src_vid: u32, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        match self {
            CsrVariant::None { .. } => None,
            CsrVariant::Multiple(csr) => csr.get_edge(src_vid, dst, ts),
            CsrVariant::Single(csr) => csr.get_edge(src_vid, dst, ts),
            CsrVariant::MultiSingle(csr) => csr.get_edge(src_vid, dst, ts),
            CsrVariant::Labeled(csr) => csr.get_edge(src_vid, dst, ts),
        }
    }

    fn edges_of(&self, src_vid: u32, ts: Timestamp) -> Vec<Nbr> {
        match self {
            CsrVariant::None { .. } => Vec::new(),
            CsrVariant::Multiple(csr) => csr.edges_of(src_vid, ts),
            CsrVariant::Single(csr) => csr.edges_of(src_vid, ts),
            CsrVariant::MultiSingle(csr) => csr.edges_of(src_vid, ts),
            CsrVariant::Labeled(csr) => csr.edges_of(src_vid, ts),
        }
    }

    fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        match self {
            CsrVariant::None { .. } => 0,
            CsrVariant::Multiple(csr) => csr.compact_with_ts(ts, reserve_ratio),
            CsrVariant::Single(csr) => csr.compact_with_ts(ts, reserve_ratio),
            CsrVariant::MultiSingle(csr) => csr.compact_with_ts(ts, reserve_ratio),
            CsrVariant::Labeled(csr) => csr.compact_with_ts(ts, reserve_ratio),
        }
    }

    fn used_memory_size(&self) -> usize {
        match self {
            CsrVariant::None { .. } => std::mem::size_of::<Self>(),
            CsrVariant::Multiple(csr) => csr.used_memory_size(),
            CsrVariant::Single(csr) => csr.used_memory_size(),
            CsrVariant::MultiSingle(csr) => csr.used_memory_size(),
            CsrVariant::Labeled(csr) => csr.used_memory_size(),
        }
    }
}

impl CsrVariant {
    /// Create an iterator over edges
    pub fn iter(&self, ts: Timestamp) -> CsrIterator<'_> {
        match self {
            CsrVariant::Multiple(csr) => CsrIterator::Multiple(csr.iter(ts)),
            CsrVariant::Single(csr) => CsrIterator::Single(csr.iter(ts)),
            CsrVariant::MultiSingle(csr) => CsrIterator::MultiSingle(csr.iter(ts)),
            CsrVariant::Labeled(csr) => CsrIterator::Labeled(csr.iter(ts)),
            CsrVariant::None { .. } => CsrIterator::None,
        }
    }
}

/// Iterator over CSR edges, supporting multiple implementation types
pub enum CsrIterator<'a> {
    /// Iterator over multi-edge CSR
    Multiple(MutableCsrIterator<'a>),
    /// Iterator over single-edge CSR
    Single(SingleMutableCsrIterator<'a>),
    /// Iterator over multi-single CSR
    MultiSingle(MultiSingleMutableCsrIterator<'a>),
    /// Iterator over labeled CSR
    Labeled(LabeledMutableCsrIterator<'a>),
    /// Empty iterator
    None,
}

impl<'a> Iterator for CsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CsrIterator::Multiple(iter) => iter.next(),
            CsrIterator::Single(iter) => iter.next(),
            CsrIterator::MultiSingle(iter) => iter.next(),
            CsrIterator::Labeled(iter) => iter.next(),
            CsrIterator::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_csr_variant() {
        let mut csr = CsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_single_csr_variant() {
        let mut csr = CsrVariant::from_strategy(EdgeStrategy::Single, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_multi_single_csr_variant() {
        let mut csr = CsrVariant::from_strategy(EdgeStrategy::MultiSingle { max_edges: 4 }, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_labeled_csr_variant() {
        let mut csr = CsrVariant::from_strategy(EdgeStrategy::Labeled, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_none_csr_variant() {
        let mut csr = CsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();

        // None variant should return the configured vertex capacity
        assert_eq!(csr.vertex_capacity(), 10);
        assert_eq!(csr.edge_count(), 0);
        assert!(csr.edges_of(0, 1).is_empty());

        // None variant should reject all insertions
        assert!(!csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 0);

        // None variant should reject all deletions
        assert!(!csr.delete_edge(0, EdgeId(100), 1));
        assert!(!csr.delete_edge_by_dst(0, VertexId::from_int64(1), 1));
        assert!(!csr.delete_edge_by_offset(0, 0, 1));
        assert!(!csr.revert_delete_by_offset(0, 0, 1));

        // None variant should return None for get_edge
        assert!(csr.get_edge(0, VertexId::from_int64(1), 1).is_none());

        // Compact and clear should be no-ops
        assert_eq!(csr.compact_with_ts(1, 0.5), 0);
        csr.clear();
        assert_eq!(csr.edge_count(), 0);
    }

    #[test]
    fn test_none_csr_iter() {
        let csr = CsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let mut iter = csr.iter(1);

        // Iterator should produce no items
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_none_csr_dump_load() {
        let csr1 = CsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let data = csr1.dump();

        // Data should start with variant tag (0 for None)
        assert!(!data.is_empty());
        assert_eq!(data[0], 0u8);

        let mut csr2 = CsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();
        csr2.load(&data).unwrap();

        // After loading, should be None variant
        assert_eq!(csr2.edge_count(), 0);
        assert!(!csr2.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 1));
    }

    #[test]
    fn test_clone() {
        let mut csr1 = CsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();
        csr1.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1);

        let csr2 = csr1.clone();
        assert_eq!(csr2.edge_count(), 1);
    }

    #[test]
    fn test_clone_none() {
        let csr1 = CsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let mut csr2 = csr1.clone();

        assert_eq!(csr2.edge_count(), 0);
        assert!(!csr2.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 1));
    }
}
