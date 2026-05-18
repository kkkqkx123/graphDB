//! Transaction Targets
//!
//! Contains transaction target implementations for PropertyGraph:
//! - undo: Undo log execution for PropertyGraph
//! - recovery: WAL recovery replay for PropertyGraph
//! - compact: Compaction operations for PropertyGraph

mod compact;
mod recovery;
mod undo;
