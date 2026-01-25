//! DescEdgeExecutor - 描述边类型执行器
//!
//! 负责查看指定边类型的详细信息。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, PropertyType, Row, Value};
use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 边类型描述信息
#[derive(Debug, Clone)]
pub struct EdgeTypeDesc {
    pub space_name: String,
    pub edge_name: String,
    pub field_id: i32,
    pub field_name: String,
    pub field_type: PropertyType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub comment: Option<String>,
}

/// 描述边类型执行器
///
/// 该执行器负责返回指定边类型的详细信息。
#[derive(Debug)]
pub struct DescEdgeExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
    edge_name: String,
}

impl<S: StorageEngine> DescEdgeExecutor<S> {
    /// 创建新的 DescEdgeExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, edge_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DescEdgeExecutor".to_string(), storage),
            space_name,
            edge_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for DescEdgeExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.get_edge_type_desc(&self.space_name, &self.edge_name);

        match result {
            Ok(Some(edge_descs)) => {
                let rows: Vec<Row> = edge_descs
                    .iter()
                    .map(|desc| {
                        Row::new(vec![
                            Value::String(desc.field_name.clone()),
                            Value::String(format!("{:?}", desc.field_type)),
                            Value::Bool(desc.nullable),
                            Value::String(desc.default_value
                                .as_ref()
                                .map(|v| format!("{}", v))
                                .unwrap_or_else(|| "".to_string())),
                            Value::String(desc.comment.clone().unwrap_or_else(|| "".to_string())),
                        ])
                    })
                    .collect();

                let dataset = DataSet {
                    columns: vec![
                        "Field".to_string(),
                        "Type".to_string(),
                        "Nullable".to_string(),
                        "Default".to_string(),
                        "Comment".to_string(),
                    ],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Ok(None) => Ok(ExecutionResult::Error(format!("Edge type '{}' not found in space '{}'",
                self.edge_name, self.space_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to describe edge type: {}", e))),
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
        "DescEdgeExecutor"
    }

    fn description(&self) -> &str {
        "Describes an edge type"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}
