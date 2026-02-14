//! ShowEdgesExecutor - 列出边类型执行器
//!
//! 负责列出指定图空间中的所有边类型。

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::{DataSet, Value};
use crate::storage::iterator::Row;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 列出边类型执行器
///
/// 该执行器负责返回指定图空间中所有边类型的列表。
#[derive(Debug)]
pub struct ShowEdgesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> ShowEdgesExecutor<S> {
    /// 创建新的 ShowEdgesExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowEdgesExecutor".to_string(), storage),
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowEdgesExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock();

        let result = storage_guard.list_edge_types(&self.space_name);

        match result {
            Ok(edge_schemas) => {
                let rows: Vec<Row> = edge_schemas
                    .iter()
                    .map(|schema| {
                        vec![Value::String(schema.edge_type_name.clone())]
                    })
                    .collect();

                let dataset = DataSet {
                    col_names: vec!["Edge Type".to_string()],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to show edge types: {}", e))),
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
        "ShowEdgesExecutor"
    }

    fn description(&self) -> &str {
        "Shows all edge types"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for ShowEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
