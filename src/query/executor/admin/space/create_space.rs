//! CreateSpaceExecutor - 创建图空间执行器
//!
//! 负责创建新的图空间，配置分片数和副本数（单节点简化版）。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 图空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_name: String,
    pub partition_num: usize,
    pub replica_factor: usize,
    pub vid_type: String,
}

impl SpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            partition_num: 1,
            replica_factor: 1,
            vid_type: "FIXED_STRING(32)".to_string(),
        }
    }

    pub fn with_partition_num(mut self, partition_num: usize) -> Self {
        self.partition_num = partition_num;
        self
    }

    pub fn with_replica_factor(mut self, replica_factor: usize) -> Self {
        self.replica_factor = replica_factor;
        self
    }

    pub fn with_vid_type(mut self, vid_type: String) -> Self {
        self.vid_type = vid_type;
        self
    }
}

/// 创建图空间执行器
///
/// 该执行器负责在存储层创建新的图空间。
#[derive(Debug)]
pub struct CreateSpaceExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_info: SpaceInfo,
}

impl<S: StorageEngine> CreateSpaceExecutor<S> {
    /// 创建新的 CreateSpaceExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_info: SpaceInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage),
            space_info,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for CreateSpaceExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.create_space(&self.space_info);

        match result {
            Ok(_) => Ok(ExecutionResult::Success),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to create space: {}", e))),
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
        "CreateSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Creates a new graph space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}
