//! Trait Implementations for PropertyGraph
//!
//! This module contains implementations of transaction-related traits
//! for the PropertyGraph storage engine. These traits enable:
//!
//! - **Atomic transactions**: All-or-nothing semantics for batch operations
//! - **Rollback support**: Automatic undo on transaction failure
//! - **Crash recovery**: WAL replay for durability
//! - **Concurrent access**: MVCC-based snapshot isolation
//!
//! # Trait Overview
//!
//! | Trait | Purpose | Used By |
//! |-------|---------|---------|
//! | `InsertTarget` | Insert operations for atomic batch inserts | `InsertTransaction` |
//! | `CompactTarget` | Storage compaction operations | `CompactTransaction` |
//! | `UndoTarget` | Rollback operations for transaction abort | `TransactionContext` |
//! | `RecoveryApplier` | WAL replay for crash recovery | `RecoveryManager` |
//!
//! # Integration Status
//!
//! These implementations are now integrated into the main data path.
//! Current status:
//!
//! | Trait | Implemented | Integrated | Priority |
//! |-------|:-----------:|:----------:|:--------:|
//! | `InsertTarget` | ✅ | ✅ | P2 |
//! | `CompactTarget` | ✅ | ✅ | P3 |
//! | `UndoTarget` | ✅ | ✅ | P1 |
//! | `RecoveryApplier` | ✅ | ✅ | P0 |
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Current Data Flow                        │
//! │  StorageClient → GraphStorage → PropertyGraph (direct)      │
//! └─────────────────────────────────────────────────────────────┘
//!                           ↓ Target
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Future Data Flow                         │
//! │  StorageClient → GraphStorage → Transaction → PropertyGraph │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example (Future Usage)
//!
//! ```rust,ignore
//! // Transactional insert using InsertTarget
//! let mut txn = InsertTransaction::new(&mut graph, &vm, &mut wal)?;
//! txn.add_vertex(label, oid, properties)?;
//! txn.commit()?;  // Atomic commit
//!
//! // Rollback using UndoTarget
//! if let Err(e) = operation {
//!     undo_log.execute_undo(&mut graph, ts)?;  // Automatic rollback
//! }
//!
//! // Recovery using RecoveryApplier
//! let mut recovery = RecoveryManager::new(config);
//! recovery.recover_with_applier(&mut graph)?;  // WAL replay
//! ```
//!
//! # See Also
//!
//! - [`docs/architecture/transaction_trait_integration.md`](docs/architecture/transaction_trait_integration.md)
//!   for detailed integration plan

mod compact_target;
mod insert_target;
mod recovery;
mod undo_target;
