//! ShowSpacesExecutor - 列出图空间执行器
//!
//! 负责列出所有已创建的图空间。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Row, Value};
use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 列出图空间执行器
///
/// 该执行器负责返回所有已创建图空间的列表。
#[derive(Debug)]
pub struct ShowSpacesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
}

impl<S: StorageEngine> ShowSpacesExecutor<S> {
    /// 创建新的 ShowSpacesExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowSpacesExecutor".to_string(), storage),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ShowSpacesExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.list_spaces();

        match result {
            Ok(space_descs) => {
                let rows: Vec<Row> = space_descs
                    .iter()
                    .map(|desc| {
                        Row::new(vec![
                            Value::Int(desc.id as i64),
                            Value::String(desc.name.clone()),
                            Value::Int(desc.partition_num as i64),
                            Value::Int(desc.replica_factor as i64),
                            Value::String(desc.vid_type.clone()),
                            Value::String(desc.charset.clone()),
                            Value::String(desc.collate.clone()),
                        ])
                    })
                    .collect();

                let dataset = DataSet {
                    columns: vec![
                        "ID".to_string(),
                        "Name".to_string(),
                        "Partition Number".to_string(),
                        "Replica Factor".to_string(),
                        "Vid Type".to_string(),
                        "Charset".to_string(),
                        "Collate".to_string(),
                    ],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to show spaces: {}", e))),
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
        "ShowSpacesExecutor"
    }

    fn description(&self) -> &str {
        "Shows all graph spaces"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}
