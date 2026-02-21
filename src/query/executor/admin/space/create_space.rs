//! CreateSpaceExecutor - 创建图空间执行器
//!
//! 负责创建新的图空间（单节点简化版）。

use std::sync::Arc;

use crate::core::types::DataType;
use crate::core::types::metadata::SpaceInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

impl SpaceInfo {
    pub fn from_executor(executor_info: &ExecutorSpaceInfo) -> Self {
        let vid_type = match executor_info.vid_type.as_str() {
            "INT64" => DataType::Int64,
            "INT32" => DataType::Int32,
            "INT16" => DataType::Int16,
            "INT8" => DataType::Int8,
            _ => DataType::String,
        };

        // 使用自动生成的Space ID
        let space_id = crate::core::types::metadata::generate_space_id();

        Self {
            space_id,
            space_name: executor_info.space_name.clone(),
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
    pub vid_type: String,
}

impl ExecutorSpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            vid_type: "FIXED_STRING(32)".to_string(),
        }
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
    if_not_exists: bool,
}

impl<S: StorageClient> CreateSpaceExecutor<S> {
    /// 创建新的 CreateSpaceExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_info: ExecutorSpaceInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage),
            space_info,
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 CreateSpaceExecutor
    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, space_info: ExecutorSpaceInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage),
            space_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let metadata_space_info = SpaceInfo::from_executor(&self.space_info);
        let result = storage_guard.create_space(&metadata_space_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Space '{}' already exists", self.space_info.space_name)))
                }
            }
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
