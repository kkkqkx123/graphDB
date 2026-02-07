//! 更新执行器
//!
//! 提供顶点和边属性的更新功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::types::metadata::UpdateInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 更新执行器
///
/// 该执行器负责更新图中顶点或边的属性。
#[derive(Debug)]
pub struct UpdateExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    update_info: UpdateInfo,
}

impl<S: StorageClient> UpdateExecutor<S> {
    /// 创建新的 UpdateExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, update_info: UpdateInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "UpdateExecutor".to_string(), storage),
            update_info,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UpdateExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.update_data("default", &self.update_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Update target not found"))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to update: {}", e))),
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
        "UpdateExecutor"
    }

    fn description(&self) -> &str {
        "Updates vertex or edge properties"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
