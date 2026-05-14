//! Undo Operation Interface
//!
//! Defines the interface for transaction rollback operations.
//! This module re-exports the undo types from the transaction layer for cross-module access.
//!
//! NOTE: The actual UndoTarget trait and related types are defined in crate::transaction::undo_log.
//! This module serves as a centralized access point for cross-module usage.

pub use crate::transaction::undo_log::{
    PropertyValue, UndoLogEntry, UndoLogError, UndoLogManager, UndoLogResult, UndoTarget,
};