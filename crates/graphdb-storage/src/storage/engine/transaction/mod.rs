//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - ops: Core transaction operations for vertex/edge manipulation
//! - targets: Undo/Recovery/Compact implementations for GraphStorageContext

mod ops;
mod targets;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, EdgeTypeLabelParams,
    RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
