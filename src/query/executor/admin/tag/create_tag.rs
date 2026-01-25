//! CreateTagExecutor - 创建标签执行器
//!
//! 负责在指定图空间中创建新的标签。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::types::metadata::{TagInfo, PropertyDef};
use crate::core::types::graph_schema::PropertyType;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

impl TagInfo {
    pub fn from_executor(executor_info: &ExecutorTagInfo) -> Self {
        let properties: Vec<PropertyType> = executor_info.properties
            .iter()
            .map(|p| PropertyType {
                name: p.name.clone(),
                type_def: p.data_type.clone(),
                is_nullable: p.nullable,
            })
            .collect();
        
        Self {
            space_name: executor_info.space_name.clone(),
            name: executor_info.tag_name.clone(),
            properties,
            comment: executor_info.comment.clone(),
        }
    }
}

/// 标签信息（执行器内部使用）
#[derive(Debug, Clone)]
pub struct ExecutorTagInfo {
    pub space_name: String,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl ExecutorTagInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 创建标签执行器
///
/// 该执行器负责在指定图空间中创建新的标签。
#[derive(Debug)]
pub struct CreateTagExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    tag_info: ExecutorTagInfo,
    if_not_exists: bool,
}

impl<S: StorageEngine> CreateTagExecutor<S> {
    /// 创建新的 CreateTagExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, tag_info: ExecutorTagInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagExecutor".to_string(), storage),
            tag_info,
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 CreateTagExecutor
    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, tag_info: ExecutorTagInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagExecutor".to_string(), storage),
            tag_info,
            if_not_exists: true,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for CreateTagExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let metadata_tag_info = TagInfo::from_executor(&self.tag_info);
        let result = storage_guard.create_tag(&metadata_tag_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Tag '{}' already exists in space '{}'",
                        self.tag_info.tag_name, self.tag_info.space_name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to create tag: {}", e))),
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
        "CreateTagExecutor"
    }

    fn description(&self) -> &str {
        "Creates a new tag"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> crate::query::executor::base::HasStorage<S> for CreateTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
