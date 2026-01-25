//! AlterTagExecutor - 修改标签执行器
//!
//! 负责修改已存在标签的属性定义。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::types::metadata::PropertyDef;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 标签修改操作类型
#[derive(Debug, Clone)]
pub enum AlterTagOp {
    Add,
    Drop,
    Change,
}

/// 标签修改项
#[derive(Debug, Clone)]
pub struct AlterTagItem {
    pub op: AlterTagOp,
    pub property: Option<PropertyDef>,
    pub property_name: Option<String>,
}

impl AlterTagItem {
    pub fn add_property(property: PropertyDef) -> Self {
        Self {
            op: AlterTagOp::Add,
            property: Some(property),
            property_name: None,
        }
    }

    pub fn drop_property(property_name: String) -> Self {
        Self {
            op: AlterTagOp::Drop,
            property: None,
            property_name: Some(property_name),
        }
    }
}

/// 标签修改信息
#[derive(Debug, Clone)]
pub struct AlterTagInfo {
    pub space_name: String,
    pub tag_name: String,
    pub items: Vec<AlterTagItem>,
    pub comment: Option<String>,
}

impl AlterTagInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            items: Vec::new(),
            comment: None,
        }
    }

    pub fn with_items(mut self, items: Vec<AlterTagItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 修改标签执行器
///
/// 该执行器负责修改已存在标签的属性定义。
#[derive(Debug)]
pub struct AlterTagExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    alter_info: AlterTagInfo,
}

impl<S: StorageEngine> AlterTagExecutor<S> {
    /// 创建新的 AlterTagExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, alter_info: AlterTagInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "AlterTagExecutor".to_string(), storage),
            alter_info,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AlterTagExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let items: Vec<String> = self.alter_info.items.iter().filter_map(|item| {
            match item.op {
                AlterTagOp::Add => item.property.as_ref().map(|p| p.name.clone()),
                AlterTagOp::Drop => item.property_name.clone(),
                AlterTagOp::Change => item.property_name.clone(),
            }
        }).collect();
        
        let result = storage_guard.alter_tag(&self.alter_info.space_name, &self.alter_info.tag_name, Vec::new(), items);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Tag '{}' not found in space '{}'",
                self.alter_info.tag_name, self.alter_info.space_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to alter tag: {}", e))),
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
        "AlterTagExecutor"
    }

    fn description(&self) -> &str {
        "Alters a tag"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> crate::query::executor::base::HasStorage<S> for AlterTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
