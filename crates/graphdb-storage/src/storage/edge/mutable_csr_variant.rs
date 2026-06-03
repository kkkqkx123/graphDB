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

/// Macro to eliminate repetitive match-based delegation to both CSR variants.
macro_rules! delegate {
    ($self:ident.$method:ident($($args:expr),* $(,)?)) => {
        match $self {
            MutableCsrVariant::Multiple(csr) => csr.$method($($args),*),
            MutableCsrVariant::Single(csr) => csr.$method($($args),*),
        }
    };
}

#[derive(Debug, Clone)]
pub enum MutableCsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
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
            EdgeStrategy::None => Err(crate::core::StorageError::invalid_operation(
                "Cannot create MutableCsrVariant with EdgeStrategy::None",
            )),
        }
    }

    pub fn clear(&mut self) {
        delegate!(self.clear())
    }
}

impl CsrBase for MutableCsrVariant {
    fn vertex_capacity(&self) -> usize {
        delegate!(self.vertex_capacity())
    }

    fn edge_count(&self) -> u64 {
        delegate!(self.edge_count())
    }

    fn dump(&self) -> Vec<u8> {
        delegate!(self.dump())
    }

    fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        delegate!(self.load(data))
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
        delegate!(self.insert_edge(src, dst, edge_id, prop_offset, ts))
    }

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.delete_edge(src, edge_id, ts),
            MutableCsrVariant::Single(csr) => csr.delete_edge_by_id(src, edge_id, ts),
        }
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        delegate!(self.delete_edge_by_dst(src, dst, ts))
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        delegate!(self.delete_edge_by_offset(src, offset, ts))
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        delegate!(self.revert_delete_by_offset(src, offset, ts))
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        match self {
            MutableCsrVariant::Multiple(csr) => csr.get_edge(src, dst, ts),
            MutableCsrVariant::Single(csr) => csr.get_edge_by_dst(src, dst, ts),
        }
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        delegate!(self.edges_of(src, ts))
    }

    fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        delegate!(self.compact_with_ts(ts, reserve_ratio))
    }

    fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        delegate!(self.find_deleted_edge(src, dst))
    }

    fn used_memory_size(&self) -> usize {
        delegate!(self.used_memory_size())
    }
}

impl MutableCsrVariant {
    pub fn iter(&self, ts: Timestamp) -> CsrIterator<'_> {
        match self {
            MutableCsrVariant::Multiple(csr) => CsrIterator::Multiple(csr.iter(ts)),
            MutableCsrVariant::Single(csr) => CsrIterator::Single(csr.iter(ts)),
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
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_single_csr_variant() {
        let mut csr = MutableCsrVariant::from_strategy(EdgeStrategy::Single, 10, 100).unwrap();

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_clone() {
        let mut csr1 = MutableCsrVariant::from_strategy(EdgeStrategy::Multiple, 10, 100).unwrap();
        csr1.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);

        let csr2 = csr1.clone();
        assert_eq!(csr2.edge_count(), 1);
    }
}
