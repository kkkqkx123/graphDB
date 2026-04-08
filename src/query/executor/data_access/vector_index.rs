//! Vector Index Management Executors
//!
//! This module implements executors for vector index DDL operations.

use std::sync::Arc;

use crate::core::error::DBError;
use crate::query::executor::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage,
};
use crate::query::planning::plan::core::nodes::data_access::vector_search::{
    CreateVectorIndexNode, DropVectorIndexNode,
};
use crate::storage::StorageClient;
use crate::vector::VectorCoordinator;
use parking_lot::Mutex;

fn convert_distance(
    dist: crate::query::parser::ast::vector::VectorDistance,
) -> crate::vector::config::VectorDistance {
    match dist {
        crate::query::parser::ast::vector::VectorDistance::Cosine => {
            crate::vector::config::VectorDistance::Cosine
        }
        crate::query::parser::ast::vector::VectorDistance::Euclidean => {
            crate::vector::config::VectorDistance::Euclid
        }
        crate::query::parser::ast::vector::VectorDistance::Dot => {
            crate::vector::config::VectorDistance::Dot
        }
    }
}

/// Create vector index executor
pub struct CreateVectorIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: CreateVectorIndexNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> CreateVectorIndexExecutor<S> {
    /// Create a new create vector index executor
    pub fn new(
        id: i64,
        node: CreateVectorIndexNode,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "CreateVectorIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StorageClient> Executor<S> for CreateVectorIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // Get space_id from execution context
        let space_id = self.base.context.current_space_id().unwrap_or(0);

        // Check if index already exists
        let exists =
            self.coordinator
                .index_exists(space_id, &self.node.tag_name, &self.node.field_name);

        if exists {
            if !self.node.if_not_exists {
                return Err(DBError::Validation(format!(
                    "Vector index '{}' already exists on {}.{}",
                    self.node.index_name, self.node.tag_name, self.node.field_name
                )));
            }
            return Ok(ExecutionResult::Success);
        }

        // Build vector index config
        let config = crate::vector::config::VectorIndexConfig {
            vector_size: self.node.vector_size,
            distance: convert_distance(self.node.distance),
            hnsw: Some(crate::vector::config::HnswConfigOptions {
                m: self.node.hnsw_m.unwrap_or(16),
                ef_construct: self.node.hnsw_ef_construct.unwrap_or(100),
                full_scan_threshold: None,
                on_disk: None,
            }),
            quantization: None,
        };

        // Create vector index using tokio runtime
        let coordinator = self.coordinator.clone();
        let tag_name = self.node.tag_name.clone();
        let field_name = self.node.field_name.clone();

        tokio::runtime::Handle::current()
            .block_on(async move {
                coordinator
                    .create_vector_index_with_config(space_id, &tag_name, &field_name, config)
                    .await
            })
            .map_err(|e| DBError::Internal(format!("Failed to create vector index: {}", e)))?;

        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Create vector index executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for CreateVectorIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("storage should be initialized")
    }
}

/// Drop vector index executor
pub struct DropVectorIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: DropVectorIndexNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> DropVectorIndexExecutor<S> {
    /// Create a new drop vector index executor
    pub fn new(
        id: i64,
        node: DropVectorIndexNode,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DropVectorIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StorageClient> Executor<S> for DropVectorIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // Get space_id from execution context
        let space_id = self.base.context.current_space_id().unwrap_or(0);

        // Find index metadata by name
        let indexes = self.coordinator.list_indexes();
        let index_metadata = indexes
            .iter()
            .find(|idx| idx.collection_name == self.node.index_name);

        if index_metadata.is_none() {
            if !self.node.if_exists {
                return Err(DBError::Validation(format!(
                    "Vector index '{}' does not exist",
                    self.node.index_name
                )));
            }
            return Ok(ExecutionResult::Success);
        }

        let metadata = index_metadata.unwrap();

        // Drop vector index using tokio runtime
        let coordinator = self.coordinator.clone();
        let tag_name = metadata.tag_name.clone();
        let field_name = metadata.field_name.clone();

        tokio::runtime::Handle::current()
            .block_on(async move {
                coordinator
                    .drop_vector_index(space_id, &tag_name, &field_name)
                    .await
            })
            .map_err(|e| DBError::Internal(format!("Failed to drop vector index: {}", e)))?;

        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Drop vector index executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DropVectorIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("storage should be initialized")
    }
}
