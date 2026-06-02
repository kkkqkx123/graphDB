//! CSR Trait Definitions
//!
//! Unified trait interface for different CSR implementations.
//! Supports runtime polymorphism for edge storage selection.

use crate::core::StorageResult;

use super::{EdgeId, EdgeStrategy, Nbr, Timestamp, VertexId};

pub trait CsrBase: std::fmt::Debug + Send + Sync {
    fn vertex_capacity(&self) -> usize;

    fn edge_count(&self) -> u64;

    fn is_empty(&self) -> bool {
        self.edge_count() == 0
    }

    fn csr_type(&self) -> CsrType;

    fn dump(&self) -> Vec<u8>;

    fn load(&mut self, data: &[u8]) -> StorageResult<()>;
}

pub trait MutableCsrTrait: CsrBase {
    /// Insert an edge.
    ///
    /// - `MutableCsr`: checks for duplicate (neighbor + valid timestamp) across primary and overflow,
    ///   writes to primary if space available, otherwise spills to overflow with auto-expansion.
    /// - `SingleMutableCsr`: overwrites based on timestamp ordering (only if new ts > existing ts).
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool;

    /// Delete an edge by edge_id.
    ///
    /// - `MutableCsr`: uses `edge_id` to locate and delete the specific edge.
    /// - `SingleMutableCsr`: `edge_id` is **ignored** since there is only one edge per vertex.
    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool;

    /// Delete all edges matching (src, dst).
    ///
    /// - `MutableCsr`: scans primary and overflow, deletes **all** matching edges.
    /// - `SingleMutableCsr`: deletes the single edge if dst matches.
    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool;

    /// Delete an edge by its offset position in the primary block.
    ///
    /// - `MutableCsr`: offset indexes into the primary block of the vertex.
    /// - `SingleMutableCsr`: only offset == 0 is valid; returns false otherwise.
    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool;

    /// Revert a deleted edge by its offset position.
    ///
    /// - `MutableCsr`: offset indexes into the primary block.
    /// - `SingleMutableCsr`: only offset == 0 is valid.
    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool;

    /// Get a specific edge by source and destination.
    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr>;

    /// Get all valid edges of a vertex at the given timestamp.
    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr>;

    /// Get the number of valid edges of a vertex.
    fn degree(&self, src: VertexId, ts: Timestamp) -> usize;

    /// Check if an edge exists between source and destination.
    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool;

    /// Compact the CSR by removing deleted edges and reclaiming space.
    ///
    /// - `MutableCsr`: merges overflow back into primary, restores flat CSR layout.
    /// - `SingleMutableCsr`: no-op (no tombstones to compact).
    fn compact(&mut self);

    /// Compact with timestamp threshold and reserve ratio.
    ///
    /// Returns the number of removed edges.
    ///
    /// - `MutableCsr`: removes edges with timestamp > `ts`, reserves `reserve_ratio` free space.
    /// - `SingleMutableCsr`: no-op, returns 0.
    fn compact_with_ts(&mut self, _ts: Timestamp, _reserve_ratio: f32) -> usize {
        0
    }

    /// Find a deleted edge by destination vertex.
    fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId>;

    /// Return the approximate memory usage in bytes.
    fn used_memory_size(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrType {
    Mutable,
    SingleMutable,
    Immutable,
}

impl CsrType {
    pub fn from_strategy(strategy: EdgeStrategy, is_immutable: bool) -> Self {
        match (strategy, is_immutable) {
            (EdgeStrategy::Multiple, false) => CsrType::Mutable,
            (EdgeStrategy::Multiple, true) => CsrType::Immutable,
            (EdgeStrategy::Single, false) => CsrType::SingleMutable,
            (EdgeStrategy::Single, true) => CsrType::Immutable,
            (EdgeStrategy::None, _) => CsrType::Immutable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_type_from_strategy() {
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::Multiple, false),
            CsrType::Mutable
        );
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::Multiple, true),
            CsrType::Immutable
        );
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::Single, false),
            CsrType::SingleMutable
        );
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::Single, true),
            CsrType::Immutable
        );
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::None, false),
            CsrType::Immutable
        );
        assert_eq!(
            CsrType::from_strategy(EdgeStrategy::None, true),
            CsrType::Immutable
        );
    }
}
