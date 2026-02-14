//! CreateEdgeExecutor - 创建边类型执行器
//!
//! 负责在指定图空间中创建新的边类型。

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::types::{EdgeTypeSchema, PropertyDef};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

impl EdgeTypeSchema {
    pub fn from_executor(executor_info: &ExecutorEdgeInfo) -> Self {
        let properties: Vec<PropertyDef> = executor_info.properties
            .iter()
            .map(|p| PropertyDef {
                name: p.name.clone(),
                data_type: p.data_type.clone(),
                nullable: p.nullable,
                default: None,
                comment: None,
            })
            .collect();
        
        Self {
            edge_type_id: 0,
            edge_type_name: executor_info.edge_name.clone(),
            properties,
            comment: executor_info.comment.clone(),
        }
    }
}

/// 边类型信息（执行器内部使用）
#[derive(Debug, Clone)]
pub struct ExecutorEdgeInfo {
    pub space_name: String,
    pub edge_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl ExecutorEdgeInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
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

/// 创建边类型执行器
///
/// 该执行器负责在指定图空间中创建新的边类型。
#[derive(Debug)]
pub struct CreateEdgeExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    edge_info: ExecutorEdgeInfo,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateEdgeExecutor<S> {
    /// 创建新的 CreateEdgeExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, edge_info: ExecutorEdgeInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateEdgeExecutor".to_string(), storage),
            edge_info,
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 CreateEdgeExecutor
    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, edge_info: ExecutorEdgeInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateEdgeExecutor".to_string(), storage),
            edge_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateEdgeExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let metadata_edge_info = EdgeTypeSchema::from_executor(&self.edge_info);
        let result = storage_guard.create_edge_type("default", &metadata_edge_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Edge type '{}' already exists in space '{}'",
                        self.edge_info.edge_name, self.edge_info.space_name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to create edge type: {}", e))),
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
        "CreateEdgeExecutor"
    }

    fn description(&self) -> &str {
        "Creates a new edge type"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateEdgeExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
