//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - ops: Core transaction operations for vertex/edge manipulation
//! - transactional: Transaction writer and undo-log-based rollback utilities
//! - targets: Undo/Recovery/Compact implementations for PropertyGraph

mod ops;
mod transactional;
mod targets;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, EdgeLabelParams,
    InsertEdgeUndoParams, RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
pub use transactional::{execute_in_transaction, with_rollback, TransactionWriter};
