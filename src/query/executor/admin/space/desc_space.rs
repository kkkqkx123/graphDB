//! DescSpaceExecutor - 描述图空间执行器
//!
//! 负责查看指定图空间的详细信息。

use std::sync::Arc;

use crate::core::{DataSet, Value};
use crate::storage::iterator::Row;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 图空间详情
#[derive(Debug, Clone)]
pub struct SpaceDesc {
    pub id: i32,
    pub name: String,
    pub vid_type: String,
    pub charset: String,
    pub collate: String,
}

impl SpaceDesc {
    pub fn to_row(&self) -> Row {
        vec![
            Value::Int(self.id as i64),
            Value::String(self.name.clone()),
            Value::String(self.vid_type.clone()),
            Value::String(self.charset.clone()),
            Value::String(self.collate.clone()),
        ]
    }
}

/// 描述图空间执行器
///
/// 该执行器负责返回指定图空间的详细信息。
#[derive(Debug)]
pub struct DescSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> DescSpaceExecutor<S> {
    /// 创建新的 DescSpaceExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DescSpaceExecutor".to_string(), storage),
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DescSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock();

        let result = storage_guard.get_space(&self.space_name);

        match result {
            Ok(Some(space_info)) => {
                let rows = vec![
                    vec![
                        Value::String(space_info.space_name.clone()),
                        Value::String(format!("{:?}", space_info.vid_type)),
                        Value::String(space_info.comment.clone().unwrap_or_else(|| "".to_string())),
                    ]
                ];

                let dataset = DataSet {
                    col_names: vec![
                        "Name".to_string(),
                        "Vid Type".to_string(),
                        "Comment".to_string(),
                    ],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Ok(None) => Ok(ExecutionResult::Error(format!("Space '{}' not found", self.space_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to describe space: {}", e))),
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
        "DescSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Describes a graph space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DescSpaceExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
