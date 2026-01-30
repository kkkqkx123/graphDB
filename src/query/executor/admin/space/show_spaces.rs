//! ShowSpacesExecutor - 列出图空间执行器
//!
//! 负责列出所有已创建的图空间。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Value};
use crate::storage::iterator::Row;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 列出图空间执行器
///
/// 该执行器负责返回所有已创建图空间的列表。
#[derive(Debug)]
pub struct ShowSpacesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
}

impl<S: StorageClient> ShowSpacesExecutor<S> {
    /// 创建新的 ShowSpacesExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowSpacesExecutor".to_string(), storage),
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowSpacesExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.list_spaces();

        match result {
            Ok(spaces) => {
                let rows: Vec<Row> = spaces
                    .iter()
                    .map(|space| {
                        vec![
                            Value::String(space.space_name.clone()),
                            Value::Int(space.partition_num as i64),
                            Value::Int(space.replica_factor as i64),
                            Value::String(format!("{:?}", space.vid_type)),
                            Value::String(space.comment.clone().unwrap_or_else(|| "".to_string())),
                        ]
                    })
                    .collect();

                let dataset = DataSet {
                    col_names: vec![
                        "Name".to_string(),
                        "Partition Number".to_string(),
                        "Replica Factor".to_string(),
                        "Vid Type".to_string(),
                        "Comment".to_string(),
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

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for ShowSpacesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
