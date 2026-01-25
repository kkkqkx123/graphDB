//! 重建索引执行器
//!
//! 提供标签索引和边索引的重建功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 重建标签索引执行器
///
/// 该执行器负责重建指定的标签索引。
#[derive(Debug)]
pub struct RebuildTagIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageEngine> RebuildTagIndexExecutor<S> {
    /// 创建新的 RebuildTagIndexExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "RebuildTagIndexExecutor".to_string(), storage),
            space_name,
            index_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for RebuildTagIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.rebuild_tag_index(&self.space_name, &self.index_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to rebuild tag index: {}", e))),
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
        "RebuildTagIndexExecutor"
    }

    fn description(&self) -> &str {
        "Rebuilds a tag index"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> crate::query::executor::base::HasStorage<S> for RebuildTagIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 重建边索引执行器
///
/// 该执行器负责重建指定的边索引。
#[derive(Debug)]
pub struct RebuildEdgeIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageEngine> RebuildEdgeIndexExecutor<S> {
    /// 创建新的 RebuildEdgeIndexExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "RebuildEdgeIndexExecutor".to_string(), storage),
            space_name,
            index_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for RebuildEdgeIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.rebuild_edge_index(&self.space_name, &self.index_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to rebuild edge index: {}", e))),
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
        "RebuildEdgeIndexExecutor"
    }

    fn description(&self) -> &str {
        "Rebuilds an edge index"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> crate::query::executor::base::HasStorage<S> for RebuildEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
