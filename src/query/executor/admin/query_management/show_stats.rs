//! ShowStatsExecutor - 显示统计执行器
//!
//! 负责显示数据库的统计信息。

use std::sync::{Arc, Mutex};

use crate::api::session::GLOBAL_QUERY_MANAGER;
use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 显示统计执行器
///
/// 该执行器负责显示数据库的统计信息。
#[derive(Debug)]
pub struct ShowStatsExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    stats_type: Option<String>,
}

impl<S: StorageClient> ShowStatsExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage),
            stats_type: None,
        }
    }

    pub fn with_type(id: i64, storage: Arc<Mutex<S>>, stats_type: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage),
            stats_type: Some(stats_type),
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowStatsExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let dataset = match &self.stats_type {
            None => self.show_all_stats(&*storage_guard),
            Some(stats_type) => match stats_type.as_str() {
                "query" => self.show_query_stats(),
                "storage" => self.show_storage_stats(&*storage_guard),
                "space" => self.show_space_stats(&*storage_guard),
                _ => self.show_all_stats(&*storage_guard),
            },
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
        "ShowStatsExecutor"
    }

    fn description(&self) -> &str {
        "Shows database statistics"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> ShowStatsExecutor<S> {
    fn show_all_stats(&self, storage: &S) -> DataSet {
        let query_stats = GLOBAL_QUERY_MANAGER.get().map(|qm| qm.get_query_stats()).unwrap_or_default();
        let storage_stats = storage.get_storage_stats();

        let rows = vec![
            vec![
                Value::String("Total Queries".to_string()),
                Value::Int(query_stats.total_queries as i64),
            ],
            vec![
                Value::String("Running Queries".to_string()),
                Value::Int(query_stats.running_queries as i64),
            ],
            vec![
                Value::String("Finished Queries".to_string()),
                Value::Int(query_stats.finished_queries as i64),
            ],
            vec![
                Value::String("Failed Queries".to_string()),
                Value::Int(query_stats.failed_queries as i64),
            ],
            vec![
                Value::String("Killed Queries".to_string()),
                Value::Int(query_stats.killed_queries as i64),
            ],
            vec![
                Value::String("Total Vertices".to_string()),
                Value::Int(storage_stats.total_vertices as i64),
            ],
            vec![
                Value::String("Total Edges".to_string()),
                Value::Int(storage_stats.total_edges as i64),
            ],
            vec![
                Value::String("Total Spaces".to_string()),
                Value::Int(storage_stats.total_spaces as i64),
            ],
            vec![
                Value::String("Total Tags".to_string()),
                Value::Int(storage_stats.total_tags as i64),
            ],
            vec![
                Value::String("Total Edge Types".to_string()),
                Value::Int(storage_stats.total_edge_types as i64),
            ],
        ];

        DataSet {
            col_names: vec!["Statistic".to_string(), "Value".to_string()],
            rows,
        }
    }

    fn show_query_stats(&self) -> DataSet {
        let query_stats = GLOBAL_QUERY_MANAGER.get().map(|qm| qm.get_query_stats()).unwrap_or_default();

        let rows = vec![
            vec![
                Value::String("Total Queries".to_string()),
                Value::Int(query_stats.total_queries as i64),
            ],
            vec![
                Value::String("Running Queries".to_string()),
                Value::Int(query_stats.running_queries as i64),
            ],
            vec![
                Value::String("Finished Queries".to_string()),
                Value::Int(query_stats.finished_queries as i64),
            ],
            vec![
                Value::String("Failed Queries".to_string()),
                Value::Int(query_stats.failed_queries as i64),
            ],
            vec![
                Value::String("Killed Queries".to_string()),
                Value::Int(query_stats.killed_queries as i64),
            ],
        ];

        DataSet {
            col_names: vec!["Statistic".to_string(), "Value".to_string()],
            rows,
        }
    }

    fn show_storage_stats(&self, storage: &S) -> DataSet {
        let storage_stats = storage.get_storage_stats();

        let rows = vec![
            vec![
                Value::String("Total Vertices".to_string()),
                Value::Int(storage_stats.total_vertices as i64),
            ],
            vec![
                Value::String("Total Edges".to_string()),
                Value::Int(storage_stats.total_edges as i64),
            ],
            vec![
                Value::String("Total Spaces".to_string()),
                Value::Int(storage_stats.total_spaces as i64),
            ],
            vec![
                Value::String("Total Tags".to_string()),
                Value::Int(storage_stats.total_tags as i64),
            ],
            vec![
                Value::String("Total Edge Types".to_string()),
                Value::Int(storage_stats.total_edge_types as i64),
            ],
        ];

        DataSet {
            col_names: vec!["Statistic".to_string(), "Value".to_string()],
            rows,
        }
    }

    fn show_space_stats(&self, storage: &S) -> DataSet {
        let spaces = storage.list_spaces().unwrap_or_default();

        let rows: Vec<Vec<Value>> = spaces
            .iter()
            .map(|space| {
                vec![
                    Value::String(space.space_name.clone()),
                    Value::Int(space.space_id as i64),
                    Value::Int(space.partition_num as i64),
                    Value::Int(space.replica_factor as i64),
                    Value::Int(space.tags.len() as i64),
                    Value::Int(space.edge_types.len() as i64),
                ]
            })
            .collect();

        DataSet {
            col_names: vec![
                "Space Name".to_string(),
                "Space ID".to_string(),
                "Partition Num".to_string(),
                "Replica Factor".to_string(),
                "Tags".to_string(),
                "Edge Types".to_string(),
            ],
            rows,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for ShowStatsExecutor<S> {
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
    async fn test_show_stats_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowStatsExecutor::new(1, storage);

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_stats_executor_with_type() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowStatsExecutor::with_type(2, storage, "query".to_string());

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowStatsExecutor::new(3, storage);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let executor = ShowStatsExecutor::new(4, storage);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "ShowStatsExecutor");
        assert_eq!(executor.description(), "Shows database statistics");
        assert!(executor.stats().num_rows == 0);
    }
}
