//! Undo Operation Interface
//!
//! Defines the interface for transaction rollback operations.
//! This module re-exports the undo types from core for cross-module access.

pub use crate::core::types::{PropertyValue, UndoLogError, UndoLogResult, UndoTarget};
pub use crate::transaction::undo_log::{UndoLogEntry, UndoLogManager};