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

use super::{
    CsrBase, CsrType, EdgeId, EdgeStrategy, MutableCsr, MutableCsrEdgeIterator,
    MutableCsrIterator, MutableCsrTrait, Nbr, SingleCsrEdgeIterator, SingleMutableCsr,
    SingleMutableCsrIterator, Timestamp, VertexId,
};

#[derive(Debug, Clone)]
pub enum MutableCsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
}

impl MutableCsrVariant {
    pub fn from_strategy(strategy: EdgeStrategy, vertex_capacity: usize, edge_capacity: usize) -> Self {
        match strategy {
            EdgeStrategy::Multiple => {
                MutableCsrVariant::Multiple(MutableCsr::with_capacity(vertex_capacity, edge_capacity))
            }
            EdgeStrategy::Single => {
                MutableCsrVariant::Single(SingleMutableCsr::with_capacity(vertex_capacity))
            }
            EdgeStrategy::None => {
                panic!("Cannot create MutableCsrVariant with EdgeStrategy::None")
            }
        }
    }

    #[inline]
    pub fn is_single(&self) -> bool {
        matches!(self, MutableCsrVariant::Single(_))
    }

    #[inline]
    pub fn is_multiple(&self) -> bool {
        matches!(self, MutableCsrVariant::Multiple(_))
    }
}

impl CsrBase for MutableCsrVariant {
    fn vertex_capacity(&self) -> usize {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.vertex_capacity(),
            MutableCsrVariant::Single(csr) => csr.vertex_capacity(),
        }
    }

    fn edge_count(&self) -> u64 {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.edge_count(),
            MutableCsrVariant::Single(csr) => csr.edge_count(),
        }
    }

    fn csr_type(&self) -> CsrType {
        match self {
            MutableCsrVariant::Multiple(_) => CsrType::Mutable,
            MutableCsrVariant::Single(_) => CsrType::SingleMutable,
        }
    }

    fn resize(&mut self, new_vertex_capacity: usize) {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.resize(new_vertex_capacity),
            MutableCsrVariant::Single(csr) => csr.resize(new_vertex_capacity),
        }
    }

    fn clear(&mut self) {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.clear(),
            MutableCsrVariant::Single(csr) => csr.clear(),
        }
    }

    fn dump(&self) -> Vec<u8> {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.dump(),
            MutableCsrVariant::Single(csr) => csr.dump(),
        }
    }

    fn load(&mut self, data: &[u8]) {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.load(data),
            MutableCsrVariant::Single(csr) => csr.load(data),
        }
    }
}

impl MutableCsrTrait for MutableCsrVariant {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.insert_edge(src, dst, edge_id, prop_offset, ts),
            MutableCsrVariant::Single(csr) => csr.insert_edge(src, dst, edge_id, prop_offset, ts),
        }
    }

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.delete_edge(src, edge_id, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_id(src, edge_id, ts),
        }
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.delete_edge_by_dst(src, dst, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_dst(src, dst, ts),
        }
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.delete_edge_by_offset(src, offset, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_offset(src, offset, ts),
        }
    }

    fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.revert_delete(src, edge_id, ts),
            MutableCsrVariant::Single(csr) => MutableCsrTrait::revert_delete(csr, src, edge_id, ts),
        }
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.revert_delete_by_offset(src, offset, ts),
            MutableCsrVariant::Single(csr) => csr.revert_delete_by_offset(src, offset, ts),
        }
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.get_edge(src, dst, ts),
            MutableCsrVariant::Single(csr) => csr.get_edge_by_dst(src, dst, ts),
        }
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.edges_of(src, ts),
            MutableCsrVariant::Single(csr) => csr.edges_of(src, ts),
        }
    }

    fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.degree(src, ts),
            MutableCsrVariant::Single(csr) => csr.degree(src, ts),
        }
    }

    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.has_edge(src, dst, ts),
            MutableCsrVariant::Single(csr) => csr.has_edge(src, dst, ts),
        }
    }

    fn compact(&mut self) {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.compact(),
            MutableCsrVariant::Single(csr) => csr.compact(),
        }
    }

    fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        match self {
            MutableCsrVariant::Multiple(csr) => {
                csr.batch_put_edges(src_list, dst_list, edge_ids, prop_offsets, ts)
            }
            MutableCsrVariant::Single(csr) => {
                csr.batch_put_edges(src_list, dst_list, edge_ids, prop_offsets, ts)
            }
        }
    }
}

impl MutableCsrVariant {
    pub fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.find_deleted_edge(src, dst),
            MutableCsrVariant::Single(csr) => csr.find_deleted_edge(src, dst),
        }
    }

    pub fn used_memory_size(&self) -> usize {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.used_memory_size(),
            MutableCsrVariant::Single(csr) => csr.used_memory_size(),
        }
    }

    pub fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.compact_with_ts(ts, reserve_ratio),
            MutableCsrVariant::Single(csr) => csr.compact_with_ts(ts, reserve_ratio),
        }
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> CsrEdgeIterator<'_> {
        match self {
            MutableCsrVariant::Multiple(csr) => {
                CsrEdgeIterator::Multiple(csr.iter_edges(src, ts))
            }
            MutableCsrVariant::Single(csr) => {
                CsrEdgeIterator::Single(csr.iter_edges(src, ts))
            }
        }
    }

    pub fn iter(&self, ts: Timestamp) -> CsrIterator<'_> {
        match self {
            MutableCsrVariant::Multiple(csr) => CsrIterator::Multiple(csr.iter(ts)),
            MutableCsrVariant::Single(csr) => CsrIterator::Single(csr.iter(ts)),
        }
    }
}

pub enum CsrEdgeIterator<'a> {
    Multiple(MutableCsrEdgeIterator<'a>),
    Single(SingleCsrEdgeIterator<'a>),
}

impl<'a> Iterator for CsrEdgeIterator<'a> {
    type Item = Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CsrEdgeIterator::Multiple(iter) => iter.next(),
            CsrEdgeIterator::Single(iter) => iter.next(),
        }
    }
}

pub enum CsrIterator<'a> {
    Multiple(MutableCsrIterator<'a>),
    Single(SingleMutableCsrIterator<'a>),
}

impl<'a> Iterator for CsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CsrIterator::Multiple(iter) => iter.next(),
            CsrIterator::Single(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100);

        assert!(csr.is_multiple());
        assert!(!csr.is_single());
        assert_eq!(csr.csr_type(), CsrType::Mutable);

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert!(csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_single_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Single, 10, 100);

        assert!(csr.is_single());
        assert!(!csr.is_multiple());
        assert_eq!(csr.csr_type(), CsrType::SingleMutable);

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert!(csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_clone() {
        let mut csr1 = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100);
        csr1.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);

        let csr2 = csr1.clone();
        assert_eq!(csr2.edge_count(), 1);
        assert!(csr2.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 1));
    }
}