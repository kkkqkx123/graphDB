//! Edge Index Actuator
//!
//! Edge indexes are not supported. All executors return an error.

use crate::core::error::DBError;
use crate::core::types::Index;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::RwLock;
use std::sync::Arc;

/// Creating a Side-Indexed Executor
#[derive(Debug)]
pub struct CreateEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_info: Index,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateEdgeIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_info: Index,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "CreateEdgeIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_info,
            if_not_exists: false,
        }
    }

    pub fn with_if_not_exists(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_info: Index,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "CreateEdgeIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateEdgeIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        Err(DBError::storage("edge indexes are not supported"))
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }
    fn is_open(&self) -> bool {
        self.base.is_open()
    }
    fn id(&self) -> i64 {
        self.base.id
    }
    fn name(&self) -> &str {
        "CreateEdgeIndexExecutor"
    }
    fn description(&self) -> &str {
        "Creates an edge index"
    }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for ShowEdgeIndexesExecutor<S> {
    fn get_storage(&self) -> &Arc<RwLock<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<RwLock<S>> {
        self.base.get_storage()
    }
}

/// Delete Side Index Enforcer
#[derive(Debug)]
pub struct DropEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropEdgeIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DropEdgeIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_name,
            if_exists: false,
        }
    }

    pub fn with_if_exists(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DropEdgeIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_name,
            if_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropEdgeIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        Err(DBError::storage("edge indexes are not supported"))
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }
    fn is_open(&self) -> bool {
        self.base.is_open()
    }
    fn id(&self) -> i64 {
        self.base.id
    }
    fn name(&self) -> &str {
        "DropEdgeIndexExecutor"
    }
    fn description(&self) -> &str {
        "Drops an edge index"
    }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DropEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<RwLock<S>> {
        self.base.get_storage()
    }
}

/// Description Edge Indexing Actuator
#[derive(Debug)]
pub struct DescEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageClient> DescEdgeIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DescEdgeIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DescEdgeIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        Err(DBError::storage("edge indexes are not supported"))
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }
    fn is_open(&self) -> bool {
        self.base.is_open()
    }
    fn id(&self) -> i64 {
        self.base.id
    }
    fn name(&self) -> &str {
        "DescEdgeIndexExecutor"
    }
    fn description(&self) -> &str {
        "Describes an edge index"
    }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DescEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<RwLock<S>> {
        self.base.get_storage()
    }
}

/// List Edge Indexing Actuator
#[derive(Debug)]
pub struct ShowEdgeIndexesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> ShowEdgeIndexesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ShowEdgeIndexesExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowEdgeIndexesExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        Err(DBError::storage("edge indexes are not supported"))
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }
    fn is_open(&self) -> bool {
        self.base.is_open()
    }
    fn id(&self) -> i64 {
        self.base.id
    }
    fn name(&self) -> &str {
        "ShowEdgeIndexesExecutor"
    }
    fn description(&self) -> &str {
        "Shows all edge indexes"
    }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}
