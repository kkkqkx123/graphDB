//! Mutable CSR Variant
//!
//! Enum wrapper for different mutable CSR implementations.
//! Provides runtime polymorphism without dynamic dispatch (dyn).
//!
//! # CSR Type Selection
//!
//! The `EdgeStrategy` enum determines which CSR implementation to use:
//! - `Multiple`: Standard `MutableCsr` for general multi-edge scenarios
//! - `Single`: `SingleMutableCsr` for one-edge-per-vertex (O(1) access)
//! - `None`: No edges stored

use crate::core::StorageResult;

use super::{
    CsrBase, EdgeId, EdgeStrategy, MutableCsr, MutableCsrIterator, MutableCsrTrait, Nbr,
    SingleMutableCsr, SingleMutableCsrIterator, Timestamp, VertexId,
};

#[derive(Debug, Clone)]
pub enum MutableCsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
    None { vertex_capacity: usize },  // Store capacity even for None variant
}

impl MutableCsrVariant {
    pub fn from_strategy(
        strategy: EdgeStrategy,
        vertex_capacity: usize,
        edge_capacity: usize,
    ) -> StorageResult<Self> {
        match strategy {
            EdgeStrategy::Multiple => Ok(MutableCsrVariant::Multiple(MutableCsr::with_capacity(
                vertex_capacity,
                edge_capacity,
            ))),
            EdgeStrategy::Single => Ok(MutableCsrVariant::Single(SingleMutableCsr::with_capacity(
                vertex_capacity,
            ))),
            EdgeStrategy::None => Ok(MutableCsrVariant::None { vertex_capacity }),
        }
    }

    pub fn clear(&mut self) {
        match self {
            MutableCsrVariant::None { .. } => {},
            MutableCsrVariant::Multiple(csr) => csr.clear(),
            MutableCsrVariant::Single(csr) => csr.clear(),
        }
    }

    /// Conditionally compact if fragmentation exceeds threshold
    ///
    /// Only applicable to `Multiple` variant; no-op for others.
    /// See `MutableCsr::fragmentation_ratio()` for interpretation.
    ///
    /// # Arguments
    /// - `threshold`: Fragmentation ratio limit (e.g., 2.5)
    /// - `ts`: Timestamp for visibility filtering
    /// - `reserve_ratio`: Reserve capacity ratio (e.g., 0.25)
    pub fn maybe_compact(&mut self, threshold: f32, ts: Timestamp, reserve_ratio: f32) {
        if let MutableCsrVariant::Multiple(csr) = self {
            if csr.should_compact(threshold) {
                csr.compact_with_ts(ts, reserve_ratio);
            }
        }
    }

    /// Get fragmentation ratio for diagnostics
    ///
    /// Returns:
    /// - `Multiple(ratio)`: Fragmentation ratio of the CSR
    /// - `Single/None`: 0.0 (no fragmentation)
    pub fn fragmentation_ratio(&self) -> f32 {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.fragmentation_ratio(),
            _ => 0.0,
        }
    }
}

impl CsrBase for MutableCsrVariant {
    fn vertex_capacity(&self) -> usize {
        match self {
            MutableCsrVariant::None { vertex_capacity } => *vertex_capacity,
            MutableCsrVariant::Multiple(csr) => csr.vertex_capacity(),
            MutableCsrVariant::Single(csr) => csr.vertex_capacity(),
        }
    }

    fn edge_count(&self) -> u64 {
        match self {
            MutableCsrVariant::None { .. } => 0,
            MutableCsrVariant::Multiple(csr) => csr.edge_count(),
            MutableCsrVariant::Single(csr) => csr.edge_count(),
        }
    }

    fn dump(&self) -> Vec<u8> {
        match self {
            MutableCsrVariant::None { vertex_capacity } => {
                let mut result = vec![0u8];
                result.extend((*vertex_capacity as u64).to_le_bytes());
                result
            }
            MutableCsrVariant::Multiple(csr) => {
                let mut result = vec![1u8];
                result.extend(csr.dump());
                result
            }
            MutableCsrVariant::Single(csr) => {
                let mut result = vec![2u8];
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
                *self = MutableCsrVariant::None { vertex_capacity };
                Ok(())
            }
            1 => {
                let mut csr = MutableCsr::new();
                csr.load(&data[1..])?;
                *self = MutableCsrVariant::Multiple(csr);
                Ok(())
            }
            2 => {
                let mut csr = SingleMutableCsr::new();
                csr.load(&data[1..])?;
                *self = MutableCsrVariant::Single(csr);
                Ok(())
            }
            _ => Err(crate::core::StorageError::deserialize_error(
                "Invalid CSR variant tag in serialized data",
            )),
        }
    }
}

impl MutableCsrTrait for MutableCsrVariant {
    fn insert_edge(
        &mut self,
        src_vid: u32,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        match self {
            MutableCsrVariant::None { .. } => false,
            MutableCsrVariant::Multiple(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            MutableCsrVariant::Single(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
        }
    }

    fn delete_edge(&mut self, src_vid: u32, edge_id: EdgeId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::None { .. } => false,
            MutableCsrVariant::Multiple(csr) => csr.delete_edge(src_vid, edge_id, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge(src_vid, edge_id, ts),
        }
    }

    fn delete_edge_by_dst(&mut self, src_vid: u32, dst: VertexId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::None { .. } => false,
            MutableCsrVariant::Multiple(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_dst(src_vid, dst, ts),
        }
    }

    fn delete_edge_by_offset(&mut self, src_vid: u32, offset: i32, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::None { .. } => false,
            MutableCsrVariant::Multiple(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_offset(src_vid, offset, ts),
        }
    }

    fn revert_delete_by_offset(&mut self, src_vid: u32, offset: i32, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::None { .. } => false,
            MutableCsrVariant::Multiple(csr) => {
                csr.revert_delete_by_offset(src_vid, offset, ts)
            }
            MutableCsrVariant::Single(csr) => csr.revert_delete_by_offset(src_vid, offset, ts),
        }
    }

    fn get_edge(&self, src_vid: u32, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        match self {
            MutableCsrVariant::None { .. } => None,
            MutableCsrVariant::Multiple(csr) => csr.get_edge(src_vid, dst, ts),
            MutableCsrVariant::Single(csr) => csr.get_edge(src_vid, dst, ts),
        }
    }

    fn edges_of(&self, src_vid: u32, ts: Timestamp) -> Vec<Nbr> {
        match self {
            MutableCsrVariant::None { .. } => Vec::new(),
            MutableCsrVariant::Multiple(csr) => csr.edges_of(src_vid, ts),
            MutableCsrVariant::Single(csr) => csr.edges_of(src_vid, ts),
        }
    }

    fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        match self {
            MutableCsrVariant::None { .. } => 0,
            MutableCsrVariant::Multiple(csr) => csr.compact_with_ts(ts, reserve_ratio),
            MutableCsrVariant::Single(csr) => csr.compact_with_ts(ts, reserve_ratio),
        }
    }

    fn used_memory_size(&self) -> usize {
        match self {
            MutableCsrVariant::None { .. } => std::mem::size_of::<Self>(),
            MutableCsrVariant::Multiple(csr) => csr.used_memory_size(),
            MutableCsrVariant::Single(csr) => csr.used_memory_size(),
        }
    }
}

impl MutableCsrVariant {
    pub fn iter(&self, ts: Timestamp) -> CsrIterator<'_> {
        match self {
            MutableCsrVariant::Multiple(csr) => CsrIterator::Multiple(csr.iter(ts)),
            MutableCsrVariant::Single(csr) => CsrIterator::Single(csr.iter(ts)),
            MutableCsrVariant::None { .. } => CsrIterator::None,
        }
    }
}

pub enum CsrIterator<'a> {
    Multiple(MutableCsrIterator<'a>),
    Single(SingleMutableCsrIterator<'a>),
    None,
}

impl<'a> Iterator for CsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CsrIterator::Multiple(iter) => iter.next(),
            CsrIterator::Single(iter) => iter.next(),
            CsrIterator::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_single_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Single, 10, 100).unwrap();

        assert!(csr.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_none_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();

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
        let csr = MutableCsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let mut iter = csr.iter(1);

        // Iterator should produce no items
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_none_csr_dump_load() {
        let csr1 = MutableCsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let data = csr1.dump();

        // Data should start with variant tag (0 for None)
        assert!(!data.is_empty());
        assert_eq!(data[0], 0u8);

        let mut csr2 = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();
        csr2.load(&data).unwrap();

        // After loading, should be None variant
        assert_eq!(csr2.edge_count(), 0);
        assert!(!csr2.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 1));
    }

    #[test]
    fn test_clone() {
        let mut csr1 = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();
        csr1.insert_edge(0u32, VertexId::from_int64(1), EdgeId(100), 0, 1);

        let csr2 = csr1.clone();
        assert_eq!(csr2.edge_count(), 1);
    }

    #[test]
    fn test_clone_none() {
        let csr1 = MutableCsrVariant::from_strategy(EdgeStrategy::None, 10, 100).unwrap();
        let mut csr2 = csr1.clone();

        assert_eq!(csr2.edge_count(), 0);
        assert!(!csr2.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 1));
    }
}
