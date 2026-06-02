//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - ops: Core transaction operations for vertex/edge manipulation
//! - targets: Undo/Recovery/Compact implementations for PropertyGraph

mod ops;
mod targets;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, EdgeLabelParams, RevertDeleteEdgeParams,
    TransactionOps, UpdateEdgePropertyUndoParams,
};
