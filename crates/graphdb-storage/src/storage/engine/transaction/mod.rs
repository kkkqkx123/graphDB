//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - ops: Core transaction operations for vertex/edge manipulation
//! - transactional: Transaction writer and undo-log-based rollback utilities
//! - targets: Undo/Recovery/Compact implementations for PropertyGraph

mod ops;
mod targets;
mod transactional;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, EdgeLabelParams,
    RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
