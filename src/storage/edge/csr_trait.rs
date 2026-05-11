//! CSR Trait Definitions
//!
//! Unified trait interface for different CSR implementations.
//! Supports runtime polymorphism for edge storage selection.

use super::{EdgeId, EdgeStrategy, ImmutableNbr, Nbr, Timestamp, VertexId};

pub trait CsrBase: std::fmt::Debug + Send + Sync {
    fn vertex_capacity(&self) -> usize;

    fn edge_count(&self) -> u64;

    fn is_empty(&self) -> bool {
        self.edge_count() == 0
    }

    fn csr_type(&self) -> CsrType;

    fn resize(&mut self, new_vertex_capacity: usize);

    fn clear(&mut self);

    fn dump(&self) -> Vec<u8>;

    fn load(&mut self, data: &[u8]);
}

pub trait MutableCsrTrait: CsrBase {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool;

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool;

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool;

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool;

    fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool;

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool;

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr>;

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr>;

    fn degree(&self, src: VertexId, ts: Timestamp) -> usize;

    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool;

    fn compact(&mut self);

    fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    );
}

pub trait ImmutableCsrTrait: CsrBase {
    fn get_edge(&self, src: VertexId, dst: VertexId) -> Option<&ImmutableNbr>;

    fn edges_of(&self, src: VertexId) -> &[ImmutableNbr];

    fn degree(&self, src: VertexId) -> usize;

    fn has_edge(&self, src: VertexId, dst: VertexId) -> bool;

    fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrType {
    Mutable,
    SingleMutable,
    Immutable,
    SingleImmutable,
    Empty,
}

impl CsrType {
    pub fn from_strategy(strategy: EdgeStrategy, is_immutable: bool) -> Self {
        match (strategy, is_immutable) {
            (EdgeStrategy::Multiple, false) => CsrType::Mutable,
            (EdgeStrategy::Multiple, true) => CsrType::Immutable,
            (EdgeStrategy::Single, false) => CsrType::SingleMutable,
            (EdgeStrategy::Single, true) => CsrType::SingleImmutable,
            (EdgeStrategy::None, _) => CsrType::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_type_from_strategy() {
        assert_eq!(CsrType::from_strategy(EdgeStrategy::Multiple, false), CsrType::Mutable);
        assert_eq!(CsrType::from_strategy(EdgeStrategy::Multiple, true), CsrType::Immutable);
        assert_eq!(CsrType::from_strategy(EdgeStrategy::Single, false), CsrType::SingleMutable);
        assert_eq!(CsrType::from_strategy(EdgeStrategy::Single, true), CsrType::SingleImmutable);
        assert_eq!(CsrType::from_strategy(EdgeStrategy::None, false), CsrType::Empty);
        assert_eq!(CsrType::from_strategy(EdgeStrategy::None, true), CsrType::Empty);
    }
}
