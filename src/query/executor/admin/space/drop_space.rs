//! DropSpaceExecutor - 删除图空间执行器
//!
//! 负责删除指定的图空间及其所有数据。

use std::sync::Arc;

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 删除图空间执行器
///
/// 该执行器负责删除指定的图空间及其所有数据。
#[derive(Debug)]
pub struct DropSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropSpaceExecutor<S> {
    /// 创建新的 DropSpaceExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropSpaceExecutor".to_string(), storage),
            space_name,
            if_exists: false,
        }
    }

    /// 创建带 IF EXISTS 选项的 DropSpaceExecutor
    pub fn with_if_exists(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropSpaceExecutor".to_string(), storage),
            space_name,
            if_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let result = storage_guard.drop_space(&self.space_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Space '{}' not found", self.space_name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to drop space: {}", e))),
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
        "DropSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Drops a graph space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DropSpaceExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
