//! Storage Operations Module
//!
//! Re-exports rollback functionality from the transaction module.
//! Rollback is a transaction-level concern, not a storage concern.

pub use crate::transaction::rollback::{
    AddEdgePropUndo, AddVertexPropUndo, CombinedRollback, CreateEdgeTypeUndo, CreateVertexTypeUndo,
    DeleteEdgePropUndo, DeleteEdgeTypeUndo, DeleteVertexPropUndo, DeleteVertexTypeUndo,
    InsertEdgeUndo, InsertVertexUndo, OperationLogContext, PropertyValue, RelatedEdgeInfo,
    RemoveEdgeUndo, RemoveVertexUndo, RenameEdgePropUndo, RenameVertexPropUndo, RollbackHelper,
    UndoLogContext, UndoLogRollback, UpdateEdgePropUndo, UpdateVertexPropUndo,
    CreateUpdateEdgePropUndoParams, CreateRemoveVertexUndoParams, CreateRemoveEdgeUndoParams,
};
