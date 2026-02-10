//! CreateSpaceExecutor - 创建图空间执行器
//!
//! 负责创建新的图空间，配置分片数和副本数（单节点简化版）。

use std::sync::{Arc, Mutex};

use crate::core::types::DataType;
use crate::core::types::metadata::SpaceInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

impl SpaceInfo {
    pub fn from_executor(executor_info: &ExecutorSpaceInfo) -> Self {
        let vid_type = match executor_info.vid_type.as_str() {
            "INT64" => DataType::Int64,
            "INT32" => DataType::Int32,
            "INT16" => DataType::Int16,
            "INT8" => DataType::Int8,
            _ => DataType::String,
        };
        
        Self {
            space_id: 0,
            space_name: executor_info.space_name.clone(),
            partition_num: executor_info.partition_num as i32,
            replica_factor: executor_info.replica_factor as i32,
            vid_type,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        }
    }
}

/// 图空间信息（执行器内部使用）
#[derive(Debug, Clone)]
pub struct ExecutorSpaceInfo {
    pub space_name: String,
    pub partition_num: usize,
    pub replica_factor: usize,
    pub vid_type: String,
}

impl ExecutorSpaceInfo {
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
pub struct CreateSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_info: ExecutorSpaceInfo,
}

impl<S: StorageClient> CreateSpaceExecutor<S> {
    /// 创建新的 CreateSpaceExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_info: ExecutorSpaceInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage),
            space_info,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let metadata_space_info = SpaceInfo::from_executor(&self.space_info);
        let result = storage_guard.create_space(&metadata_space_info);

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

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateSpaceExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
