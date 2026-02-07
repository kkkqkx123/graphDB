//! 插入数据执行器
//!
//! 提供顶点插入和边插入功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::types::metadata::{InsertEdgeInfo, InsertVertexInfo};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 插入顶点执行器
///
/// 该执行器负责向图中插入新的顶点。
#[derive(Debug)]
pub struct InsertVertexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    insert_info: InsertVertexInfo,
}

impl<S: StorageClient> InsertVertexExecutor<S> {
    /// 创建新的 InsertVertexExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, insert_info: InsertVertexInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertVertexExecutor".to_string(), storage),
            insert_info,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for InsertVertexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.insert_vertex_data("default", &self.insert_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Vertex already exists: {}", self.insert_info.vertex_id))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to insert vertex: {}", e))),
        }
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
        "InsertVertexExecutor"
    }

    fn description(&self) -> &str {
        "Inserts a vertex"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for InsertVertexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 插入边执行器
///
/// 该执行器负责向图中插入新的边。
#[derive(Debug)]
pub struct InsertEdgeExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    insert_info: InsertEdgeInfo,
}

impl<S: StorageClient> InsertEdgeExecutor<S> {
    /// 创建新的 InsertEdgeExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, insert_info: InsertEdgeInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertEdgeExecutor".to_string(), storage),
            insert_info,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for InsertEdgeExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.insert_edge_data("default", &self.insert_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Edge already exists from {} to {} with rank {}",
                self.insert_info.src_vertex_id, self.insert_info.dst_vertex_id, self.insert_info.rank))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to insert edge: {}", e))),
        }
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
        "InsertEdgeExecutor"
    }

    fn description(&self) -> &str {
        "Inserts an edge"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for InsertEdgeExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
