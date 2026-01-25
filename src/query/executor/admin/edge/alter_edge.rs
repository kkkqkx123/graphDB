//! AlterEdgeExecutor - 修改边类型执行器
//!
//! 负责修改已存在边类型的属性定义。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::types::metadata::PropertyDef;
use crate::core::types::graph_schema::PropertyType;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 边类型修改操作类型
#[derive(Debug, Clone)]
pub enum AlterEdgeOp {
    Add,
    Drop,
    Change,
}

/// 边类型修改项
#[derive(Debug, Clone)]
pub struct AlterEdgeItem {
    pub op: AlterEdgeOp,
    pub property: Option<PropertyDef>,
    pub property_name: Option<String>,
}

impl AlterEdgeItem {
    pub fn add_property(property: PropertyDef) -> Self {
        Self {
            op: AlterEdgeOp::Add,
            property: Some(property),
            property_name: None,
        }
    }

    pub fn drop_property(property_name: String) -> Self {
        Self {
            op: AlterEdgeOp::Drop,
            property: None,
            property_name: Some(property_name),
        }
    }
}

/// 边类型修改信息
#[derive(Debug, Clone)]
pub struct AlterEdgeInfo {
    pub space_name: String,
    pub edge_name: String,
    pub items: Vec<AlterEdgeItem>,
    pub comment: Option<String>,
}

impl AlterEdgeInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
            items: Vec::new(),
            comment: None,
        }
    }

    pub fn with_items(mut self, items: Vec<AlterEdgeItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 修改边类型执行器
///
/// 该执行器负责修改已存在边类型的属性定义。
#[derive(Debug)]
pub struct AlterEdgeExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    alter_info: AlterEdgeInfo,
}

impl<S: StorageEngine> AlterEdgeExecutor<S> {
    /// 创建新的 AlterEdgeExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, alter_info: AlterEdgeInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "AlterEdgeExecutor".to_string(), storage),
            alter_info,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AlterEdgeExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let items: Vec<String> = self.alter_info.items.iter().filter_map(|item| {
            match item.op {
                AlterEdgeOp::Add => item.property.as_ref().map(|p| p.name.clone()),
                AlterEdgeOp::Drop => item.property_name.clone(),
                AlterEdgeOp::Change => item.property_name.clone(),
            }
        }).collect();
        
        let result = storage_guard.alter_edge_type(&self.alter_info.space_name, &self.alter_info.edge_name, Vec::new(), items);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Edge type '{}' not found in space '{}'",
                self.alter_info.edge_name, self.alter_info.space_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to alter edge type: {}", e))),
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
        "AlterEdgeExecutor"
    }

    fn description(&self) -> &str {
        "Alters an edge type"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> crate::query::executor::base::HasStorage<S> for AlterEdgeExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
