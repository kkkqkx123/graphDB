//! Transaction Module
//!
//! Unified transaction module containing core transaction operations:
//! - Transaction operations (ops.rs) - core transaction ops for vertex/edge manipulation
//!
//! Note: The following remain in their original locations due to dependency constraints:
//! - TransactionalWriter: stays in graph_storage/ (depends on GraphStorageContext)
//! - Transaction support (with_rollback, execute_in_transaction): stays in graph_storage/
//! - Undo/Recovery/Compact impls: stay in property_graph/transaction_targets/

mod ops;

pub use ops::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, InsertEdgeUndoParams,
    RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
