//! WAL Traits
//!
//! Core traits for WAL writer and recovery applier.

use crate::core::types::{LabelId, Timestamp};
use crate::core::StorageResult;

use super::redo::{InsertEdgeRedo, UpdateEdgePropRedo};
use super::types::WalResult;

/// WAL writer trait
pub trait WalWriter: Send + Sync {
    fn open(&mut self) -> WalResult<()>;
    fn close(&mut self);
    fn append(&mut self, data: &[u8]) -> WalResult<bool>;
    fn sync(&self) -> WalResult<()>;
}

/// Trait for applying recovered operations to the storage engine.
/// Implementors handle the actual data modifications during WAL replay.
pub trait RecoveryApplier {
    fn replay_insert_vertex(
        &self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_insert_edge(
        &self,
        redo: &InsertEdgeRedo,
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_update_vertex_prop(
        &self,
        label: LabelId,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_update_edge_prop(
        &self,
        redo: &UpdateEdgePropRedo,
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_delete_vertex(&self, label: LabelId, oid: &[u8], ts: Timestamp) -> StorageResult<()>;

    fn replay_delete_edge(
        &self,
        src_label: LabelId,
        src_oid: &[u8],
        dst_label: LabelId,
        dst_oid: &[u8],
        edge_label: LabelId,
        ts: Timestamp,
    ) -> StorageResult<()>;
}