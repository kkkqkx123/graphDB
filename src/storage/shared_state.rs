//! Storage layer shared state module
//!
//! Aggregates all states that need to be shared across storage layer components, reducing Arc nesting

use crate::storage::metadata::{RedbIndexMetadataManager, RedbSchemaManager};
use crate::storage::operations::{RedbReader, RedbWriter};
use crate::sync::SyncManager;
use crate::transaction::context::TransactionContext;
use parking_lot::Mutex;
use redb::Database;
use std::sync::Arc;

/// Storage layer shared state
///
/// These fields are shared across multiple storage components, wrapped with Arc
#[derive(Clone)]
pub struct StorageSharedState {
    pub db: Arc<Database>,
    pub schema_manager: Arc<RedbSchemaManager>,
    pub index_metadata_manager: Arc<RedbIndexMetadataManager>,
    pub sync_manager: Option<Arc<SyncManager>>,
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
            sync_manager: None,
        }
    }

    pub fn with_sync_manager(&mut self, sync_manager: Arc<SyncManager>) {
        self.sync_manager = Some(sync_manager);
    }
}

/// Storage layer internal state
///
/// These fields do not need to be shared outside of Storage
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
}
