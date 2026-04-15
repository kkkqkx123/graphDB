//! Vector Search Executor Builder
//!
//! Responsible for creating vector search related executors.
//! This module isolates the complex synchronization dependencies required by vector search operations.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::base::ExecutorEnum;
use crate::query::executor::data_access::{
    CreateVectorIndexExecutor, DropVectorIndexExecutor, VectorLookupExecutor, VectorMatchExecutor,
    VectorSearchExecutor,
};
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planning::plan::core::nodes::search::vector::data_access::{
    VectorLookupNode, VectorMatchNode, VectorSearchNode,
};
use crate::query::planning::plan::core::nodes::search::vector::management::{
    CreateVectorIndexNode, DropVectorIndexNode,
};
use crate::storage::StorageClient;
use crate::sync::SyncManager;

/// Vector Search Executor Builder
///
/// Handles the creation of all vector search related executors.
/// These executors require special coordination with the VectorSyncCoordinator
/// for managing vector index operations and external vector database synchronization.
pub struct VectorSearchBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> VectorSearchBuilder<S> {
    /// Create a new vector search builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Build VectorSearch executor
    pub fn build_vector_search(
        node: &VectorSearchNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        sync_manager: Option<&Arc<SyncManager>>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let coordinator = sync_manager
            .ok_or_else(|| QueryError::ExecutionError("Sync manager not available".to_string()))?
            .vector_coordinator()
            .cloned()
            .ok_or_else(|| {
                QueryError::ExecutionError("Vector coordinator not available".to_string())
            })?;

        let executor = VectorSearchExecutor::new(
            node.id(),
            node.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::VectorSearch(executor))
    }

    /// Build CreateVectorIndex executor
    pub fn build_create_vector_index(
        node: &CreateVectorIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        sync_manager: Option<&Arc<SyncManager>>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let coordinator = sync_manager
            .ok_or_else(|| QueryError::ExecutionError("Sync manager not available".to_string()))?
            .vector_coordinator()
            .cloned()
            .ok_or_else(|| {
                QueryError::ExecutionError("Vector coordinator not available".to_string())
            })?;

        let executor = CreateVectorIndexExecutor::new(
            node.id(),
            node.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::CreateVectorIndex(executor))
    }

    /// Build DropVectorIndex executor
    pub fn build_drop_vector_index(
        node: &DropVectorIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        sync_manager: Option<&Arc<SyncManager>>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let coordinator = sync_manager
            .ok_or_else(|| QueryError::ExecutionError("Sync manager not available".to_string()))?
            .vector_coordinator()
            .cloned()
            .ok_or_else(|| {
                QueryError::ExecutionError("Vector coordinator not available".to_string())
            })?;

        let executor = DropVectorIndexExecutor::new(
            node.id(),
            node.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::DropVectorIndex(executor))
    }

    /// Build VectorLookup executor
    pub fn build_vector_lookup(
        node: &VectorLookupNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        sync_manager: Option<&Arc<SyncManager>>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let coordinator = sync_manager
            .ok_or_else(|| QueryError::ExecutionError("Sync manager not available".to_string()))?
            .vector_coordinator()
            .cloned()
            .ok_or_else(|| {
                QueryError::ExecutionError("Vector coordinator not available".to_string())
            })?;

        let executor = VectorLookupExecutor::new(
            node.id(),
            node.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::VectorLookup(executor))
    }

    /// Build VectorMatch executor
    pub fn build_vector_match(
        node: &VectorMatchNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
        sync_manager: Option<&Arc<SyncManager>>,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let coordinator = sync_manager
            .ok_or_else(|| QueryError::ExecutionError("Sync manager not available".to_string()))?
            .vector_coordinator()
            .cloned()
            .ok_or_else(|| {
                QueryError::ExecutionError("Vector coordinator not available".to_string())
            })?;

        let executor = VectorMatchExecutor::new(
            node.id(),
            node.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::VectorMatch(executor))
    }
}

impl<S: StorageClient + 'static> Default for VectorSearchBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
