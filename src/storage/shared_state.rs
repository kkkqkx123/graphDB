//! Storage layer shared state module
//!
//! Aggregates all states that need to be shared across storage layer components.

use std::sync::Arc;
use parking_lot::RwLock;

use crate::search::manager::FulltextIndexManager;
use crate::storage::metadata::{IndexMetadataManager, SchemaManager};
use crate::storage::property_graph::PropertyGraph;
use crate::transaction::version_manager::VersionManager;
use crate::sync::SyncManager;
use crate::transaction::context::TransactionContext;

#[derive(Clone)]
pub struct StorageSharedState {
    pub graph: Arc<RwLock<PropertyGraph>>,
    pub version_manager: Arc<VersionManager>,
    pub schema_manager: Arc<dyn SchemaManager + Send + Sync>,
    pub index_metadata_manager: Arc<dyn IndexMetadataManager + Send + Sync>,
    pub sync_manager: Arc<RwLock<Option<Arc<SyncManager>>>>,
    pub fulltext_manager: Arc<RwLock<Option<Arc<FulltextIndexManager>>>>,
}

impl StorageSharedState {
    pub fn new(
        graph: Arc<RwLock<PropertyGraph>>,
        version_manager: Arc<VersionManager>,
        schema_manager: Arc<dyn SchemaManager + Send + Sync>,
        index_metadata_manager: Arc<dyn IndexMetadataManager + Send + Sync>,
    ) -> Self {
        Self {
            graph,
            version_manager,
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

pub struct StorageInner {
    pub graph: Arc<RwLock<PropertyGraph>>,
    pub version_manager: Arc<VersionManager>,
    pub current_txn_context: parking_lot::Mutex<Option<Arc<TransactionContext>>>,
}

impl StorageInner {
    pub fn new(graph: Arc<RwLock<PropertyGraph>>, version_manager: Arc<VersionManager>) -> Self {
        Self {
            graph,
            version_manager,
            current_txn_context: parking_lot::Mutex::new(None),
        }
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        let mut txn_guard = self.current_txn_context.lock();
        *txn_guard = context;
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    pub fn get_current_txn_id(&self) -> crate::transaction::types::TransactionId {
        let ctx = self.current_txn_context.lock().clone();
        ctx.map(|c| c.id).unwrap_or(0)
    }
}

impl std::fmt::Debug for StorageSharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageSharedState")
            .field("has_sync_manager", &self.sync_manager.read().is_some())
            .field("has_fulltext_manager", &self.fulltext_manager.read().is_some())
            .finish()
    }
}

impl std::fmt::Debug for StorageInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageInner")
            .field("has_transaction_context", &self.current_txn_context.lock().is_some())
            .finish()
    }
}
