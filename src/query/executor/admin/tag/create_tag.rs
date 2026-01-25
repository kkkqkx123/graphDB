//! CreateTagExecutor - 创建标签执行器
//!
//! 负责在指定图空间中创建新的标签。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::PropertyDef;
use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 标签信息
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub space_name: String,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl TagInfo {
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
    tag_info: TagInfo,
    if_not_exists: bool,
}

impl<S: StorageEngine> CreateTagExecutor<S> {
    /// 创建新的 CreateTagExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, tag_info: TagInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagExecutor".to_string(), storage),
            tag_info,
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 CreateTagExecutor
    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, tag_info: TagInfo) -> Self {
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
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.create_tag(&self.tag_info);

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
