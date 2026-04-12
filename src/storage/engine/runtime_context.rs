//! Runtime Context Module - Storage Layer Context Management
//!
//! Provides storage layer context information during query execution, including:
//! - StorageEnv
//! - PlanContext
//! - RuntimeContext

use crate::storage::engine::RedbStorage;
use crate::storage::metadata::RedbSchemaManager;
use std::sync::Arc;

/// storage environment
#[derive(Clone)]
pub struct StorageEnv {
    /// storage engine
    pub storage_engine: Arc<RedbStorage>,
    /// Schema Manager
    pub schema_manager: Arc<RedbSchemaManager>,
}

impl std::fmt::Debug for StorageEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageEnv")
            .field("storage_engine", &"<RedbStorage>")
            .field("schema_manager", &"<RedbSchemaManager>")
            .finish()
    }
}

/// Program context (storage layer)
/// Storing information that remains unchanged during processing
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// Storage Environment References
    pub storage_env: Arc<StorageEnv>,
    /// Space ID
    pub space_id: u64,
}

/// runtime context
/// Storing information that may change during processing
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Program Context Citation
    pub plan_context: Arc<PlanContext>,
}

impl RuntimeContext {
    /// Creating a new runtime context
    pub fn new(plan_context: Arc<PlanContext>) -> Self {
        Self { plan_context }
    }

    /// Getting the storage environment
    pub fn env(&self) -> &Arc<StorageEnv> {
        &self.plan_context.storage_env
    }

    /// Get Space ID
    pub fn space_id(&self) -> u64 {
        self.plan_context.space_id
    }
}

impl RuntimeContext {
    /// Creating a simple runtime context (for scenarios where a full PlanContext is not required)
    pub fn new_simple() -> Arc<Self> {
        let storage = Arc::new(RedbStorage::new().expect("Failed to create RedbStorage"));
        let storage_env = Arc::new(StorageEnv {
            storage_engine: storage.clone(),
            schema_manager: storage.state().schema_manager.clone(),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 0,
        });

        Arc::new(Self::new(plan_context))
    }
}
