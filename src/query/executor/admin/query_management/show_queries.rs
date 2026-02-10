//! ShowQueriesExecutor - 显示查询执行器
//!
//! 负责显示正在运行的查询列表。

use std::sync::{Arc, Mutex};

use crate::api::session::GLOBAL_QUERY_MANAGER;
use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 显示查询执行器
///
/// 该执行器负责显示正在运行的查询列表。
#[derive(Debug)]
pub struct ShowQueriesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    show_all: bool,
}

impl<S: StorageClient> ShowQueriesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowQueriesExecutor".to_string(), storage),
            show_all: false,
        }
    }

    pub fn with_show_all(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowQueriesExecutor".to_string(), storage),
            show_all: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowQueriesExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let queries = if self.show_all {
            GLOBAL_QUERY_MANAGER.get().map(|qm| qm.get_all_queries()).unwrap_or_default()
        } else {
            GLOBAL_QUERY_MANAGER.get().map(|qm| qm.get_running_queries()).unwrap_or_default()
        };

        let rows: Vec<Vec<Value>> = queries
            .iter()
            .map(|q| {
                vec![
                    Value::Int(q.query_id),
                    Value::Int(q.session_id),
                    Value::String(q.user_name.clone()),
                    Value::String(q.space_name.clone().unwrap_or_else(|| "NULL".to_string())),
                    Value::String(q.query_text.clone()),
                    Value::String(format!("{:?}", q.status)),
                    Value::Int(q.duration()),
                ]
            })
            .collect();

        let dataset = DataSet {
            col_names: vec![
                "QueryID".to_string(),
                "SessionID".to_string(),
                "User".to_string(),
                "Space".to_string(),
                "QueryText".to_string(),
                "Status".to_string(),
                "Duration(ms)".to_string(),
            ],
            rows,
        };

        Ok(ExecutionResult::DataSet(dataset))
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
        "ShowQueriesExecutor"
    }

    fn description(&self) -> &str {
        "Shows running queries"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for ShowQueriesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[tokio::test]
    async fn test_show_queries_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowQueriesExecutor::new(1, storage);

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_queries_executor_show_all() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowQueriesExecutor::with_show_all(2, storage);

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowQueriesExecutor::new(3, storage);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let executor = ShowQueriesExecutor::new(4, storage);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "ShowQueriesExecutor");
        assert_eq!(executor.description(), "Shows running queries");
        assert!(executor.stats().num_rows == 0);
    }
}
