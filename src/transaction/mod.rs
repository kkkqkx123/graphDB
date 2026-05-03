//! Transaction Management Module
//!
//! Provides transaction management functionality for GraphDB, including:
//! - Transaction lifecycle management (start, commit, abort)
//! - Transaction statistics and monitoring
//! - MVCC version management
//! - Write-Ahead Log (WAL) for durability
//! - Undo Log for transaction rollback
//!
//! ## Usage Example
//!
//! ```rust
//! use graphdb::transaction::{TransactionManager, TransactionOptions};
//!
//! // Create transaction manager
//! let manager = TransactionManager::new(db, Default::default());
//!
//! // Start transaction
//! let txn_id = manager.begin_transaction(TransactionOptions::default())?;
//!
//! // Execute operations...
//!
//! // Commit transaction
//! manager.commit_transaction(txn_id)?;
//! ```

pub mod cleaner;
pub mod context;
pub mod index_buffer;
pub mod manager;
pub mod monitor;
pub mod types;
pub mod undo_log;
pub mod version_manager;
pub mod wal;

#[cfg(test)]
pub mod context_test;
#[cfg(test)]
pub mod manager_test;

pub use cleaner::TransactionCleaner;
pub use context::TransactionContext;
pub use index_buffer::IndexUpdateBuffer;
pub use manager::TransactionManager;
pub use monitor::TransactionMonitor;
pub use types::*;

// Re-export from version_manager module
pub use version_manager::{
    InsertTimestampGuard, ReadTimestampGuard, UpdateTimestampGuard, VersionManager,
    VersionManagerConfig, VersionManagerError, VersionManagerResult,
};

// Re-export from undo_log module
pub use undo_log::{
    AddEdgePropUndo, AddVertexPropUndo, CreateEdgeTypeUndo, CreateVertexTypeUndo,
    DeleteEdgePropUndo, DeleteEdgeTypeUndo, DeleteVertexPropUndo, DeleteVertexTypeUndo,
    InsertEdgeUndo, InsertVertexUndo, PropertyValue, RemoveEdgeUndo, RemoveVertexUndo,
    RelatedEdgeInfo, UndoLog, UndoLogError, UndoLogManager, UndoLogResult, UndoTarget,
    UpdateEdgePropUndo, UpdateVertexPropUndo,
};

// Re-export from wal module
pub use wal::{
    ColumnId, CreateEdgeTypeRedo, CreateVertexTypeRedo, DeleteEdgeRedo, DeleteVertexRedo,
    DummyWalWriter, EdgeId, InsertEdgeRedo, InsertVertexRedo, LabelId, LocalWalParser,
    LocalWalWriter, Timestamp, UpdateEdgePropRedo, UpdateVertexPropRedo, UpdateWalUnit,
    WalConfig, WalContentUnit, WalEntry, WalEntryIter, WalError, WalHeader, WalOpType,
    WalParser, WalParserFactory, WalResult, WalWriter, WalWriterFactory, VertexId,
};

/// Transaction Management Module Version
pub const VERSION: &str = "1.0.0";

/// Create transaction manager with default configuration
pub fn create_transaction_manager(db: std::sync::Arc<redb::Database>) -> TransactionManager {
    TransactionManager::new(db, TransactionManagerConfig::default())
}

/// Create read-only transaction options
pub fn readonly_options() -> TransactionOptions {
    TransactionOptions::new().read_only()
}

/// Create high-performance write transaction options (does not guarantee immediate durability)
pub fn high_performance_write_options() -> TransactionOptions {
    TransactionOptions::new().with_durability(DurabilityLevel::None)
}

/// Create repeatable read transaction options
pub fn repeatable_read_options() -> TransactionOptions {
    TransactionOptions::new().with_isolation_level(IsolationLevel::RepeatableRead)
}

/// Create default retry configuration
pub fn default_retry_config() -> RetryConfig {
    RetryConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio;

    fn create_test_db() -> (Arc<redb::Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = Arc::new(
            redb::Database::create(temp_dir.path().join("test.db"))
                .expect("Failed to create test database"),
        );
        (db, temp_dir)
    }

    #[test]
    fn test_module_version() {
        assert_eq!(VERSION, "1.0.0");
    }

    #[tokio::test]
    async fn test_create_transaction_manager() {
        let (db, _temp) = create_test_db();
        let manager = create_transaction_manager(db);

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");
    }

    #[tokio::test]
    async fn test_readonly_options() {
        let (db, _temp) = create_test_db();
        let manager = create_transaction_manager(db);

        let options = readonly_options();
        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin readonly transaction");

        let ctx = manager
            .get_context(txn_id)
            .expect("Failed to get transaction context");
        assert!(ctx.read_only);

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");
    }

    #[test]
    fn test_high_performance_options() {
        let options = high_performance_write_options();
        assert_eq!(options.durability, DurabilityLevel::None);
    }
}
