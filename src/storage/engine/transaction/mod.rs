//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - ops: Core transaction operations for vertex/edge manipulation
//! - targets: Undo/Recovery/Compact implementations for PropertyGraph
//!
//! Note: The following remain in graph_storage/ due to dependency constraints:
//! - TransactionalWriter: depends on GraphStorageContext
//! - Transaction support utilities: depends on PropertyGraph and UndoLogManager

mod ops;
mod targets;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, InsertEdgeUndoParams,
    RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
