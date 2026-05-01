//! Storage layer shared state module
//!
//! Aggregates all states that need to be shared across storage layer components, reducing Arc nesting

use crate::search::manager::FulltextIndexManager;
use crate::storage::metadata::{RedbIndexMetadataManager, RedbSchemaManager};
use crate::storage::operations::{RedbReader, RedbWriter};
use crate::sync::SyncManager;
use crate::transaction::context::TransactionContext;
use parking_lot::{Mutex, RwLock};
use redb::Database;
use std::sync::Arc;

/// Storage layer shared state
///
/// These fields are shared across multiple storage layer components, wrapped with Arc
#[derive(Clone)]
pub struct StorageSharedState {
    pub db: Arc<Database>,
    pub schema_manager: Arc<RedbSchemaManager>,
    pub index_metadata_manager: Arc<RedbIndexMetadataManager>,
    pub sync_manager: Arc<RwLock<Option<Arc<SyncManager>>>>,
    pub fulltext_manager: Arc<RwLock<Option<Arc<FulltextIndexManager>>>>,
}

impl StorageSharedState {
    pub fn new(
        db: Arc<Database>,
        schema_manager: Arc<RedbSchemaManager>,
        index_metadata_manager: Arc<RedbIndexMetadataManager>,
    ) -> Self {
        Self {
            db,
            schema_manager,
            index_metadata_manager,
            sync_manager: Arc::new(RwLock::new(None)),
            fulltext_manager: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_sync_manager(&mut self, sync_manager: Arc<SyncManager>) {
        *self.sync_manager.write() = Some(sync_manager);
    }

    pub fn set_sync_manager(&self, sync_manager: Arc<SyncManager>) {
        *self.sync_manager.write() = Some(sync_manager);
    }

    pub fn get_sync_manager(&self) -> Option<Arc<SyncManager>> {
        self.sync_manager.read().clone()
    }

    pub fn with_fulltext_manager(&mut self, fulltext_manager: Arc<FulltextIndexManager>) {
        *self.fulltext_manager.write() = Some(fulltext_manager);
    }

    pub fn set_fulltext_manager(&self, fulltext_manager: Arc<FulltextIndexManager>) {
        *self.fulltext_manager.write() = Some(fulltext_manager);
    }

    pub fn get_fulltext_manager(&self) -> Option<Arc<FulltextIndexManager>> {
        self.fulltext_manager.read().clone()
    }
}

/// Storage layer internal state
///
/// These fields do not need to be shared outside of Storage.
///
/// Lock ordering convention to prevent deadlocks:
/// Always acquire locks in this order: reader -> writer -> current_txn_context
/// Never acquire an earlier lock while holding a later one.
///
/// IMPORTANT: To prevent deadlocks, we NEVER hold multiple locks simultaneously.
/// Each lock is acquired, used, and released before acquiring the next one.
pub struct StorageInner {
    pub reader: Arc<Mutex<RedbReader>>,
    pub writer: Arc<Mutex<RedbWriter>>,
    pub current_txn_context: Mutex<Option<Arc<TransactionContext>>>,
}

impl StorageInner {
    pub fn new(reader: RedbReader, writer: RedbWriter) -> Self {
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            current_txn_context: Mutex::new(None),
        }
    }

    /// Set the current transaction context.
    ///
    /// This method carefully avoids holding multiple locks simultaneously to prevent deadlocks.
    /// The order of operations is:
    /// 1. Update reader's transaction context (acquire/release reader lock)
    /// 2. Update current_txn_context (acquire/release current_txn_context lock)
    ///
    /// This ensures no deadlock can occur with other methods that may acquire
    /// these locks in different orders.
    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        // Step 1: Update reader's transaction context first
        // This is safe because we release the lock before acquiring the next one
        {
            let mut reader_guard = self.reader.lock();
            if let Some(ref ctx) = &context {
                reader_guard.set_transaction_context(Some(ctx.clone()));
            } else {
                reader_guard.set_transaction_context(None);
            }
        }
        // reader lock is now released

        // Step 2: Update current_txn_context
        {
            let mut txn_guard = self.current_txn_context.lock();
            *txn_guard = context;
        }
        // current_txn_context lock is now released
    }

    /// Get the current transaction context
    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    /// Get the current transaction ID without holding the lock.
    /// This is a convenience method that clones the transaction ID if present.
    pub fn get_current_txn_id(&self) -> crate::transaction::types::TransactionId {
        let ctx = self.current_txn_context.lock().clone();
        ctx.map(|c| c.id).unwrap_or(0)
    }
}
