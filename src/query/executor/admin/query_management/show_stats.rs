//! ShowStatsExecutor - 显示统计执行器
//!
//! 负责显示数据库的统计信息。

use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// 显示统计类型
#[derive(Debug, Clone)]
pub enum ShowStatsType {
    /// 显示存储统计（顶点、边、空间、标签、边类型数量）
    Storage,
    /// 显示空间统计（空间列表）
    Space,
}

/// 显示统计执行器
///
/// 该执行器负责显示数据库的统计信息。
#[derive(Debug)]
pub struct ShowStatsExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    stats_type: ShowStatsType,
}

impl<S: StorageClient> ShowStatsExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        stats_type: ShowStatsType,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage, expr_context),
            stats_type,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowStatsExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock();

        let dataset = match &self.stats_type {
            ShowStatsType::Storage => self.show_storage_stats(&*storage_guard),
            ShowStatsType::Space => self.show_space_stats(&*storage_guard),
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
                    Value::Int(space.tags.len() as i64),
                    Value::Int(space.edge_types.len() as i64),
                ]
            })
            .collect();

        DataSet {
            col_names: vec![
                "Space Name".to_string(),
                "Space ID".to_string(),
                "Tags".to_string(),
                "Edge Types".to_string(),
            ],
            rows,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for ShowStatsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not available")
    }
}

#[cfg(test)]
mod tests {
    use crate::query::executor::admin::query_management::show_stats::{
        ShowStatsExecutor, ShowStatsType,
    };
    use crate::query::executor::Executor;
    use crate::query::validator::context::ExpressionAnalysisContext;
    use crate::storage::test_mock::MockStorage;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn test_show_stats_executor_storage() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowStatsExecutor::new(1, storage, ShowStatsType::Storage, expr_context);

        let result = executor.execute();
        assert!(result.is_ok());

        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::DataSet(dataset) => {
                assert_eq!(
                    dataset.col_names,
                    vec!["Statistic".to_string(), "Value".to_string()]
                );
                assert_eq!(dataset.rows.len(), 5);

                let stats_map: std::collections::HashMap<String, i64> = dataset
                    .rows
                    .iter()
                    .filter_map(|row| {
                        if row.len() >= 2 {
                            if let (
                                crate::core::Value::String(key),
                                crate::core::Value::Int(value),
                            ) = (&row[0], &row[1])
                            {
                                Some((key.clone(), *value))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                assert_eq!(stats_map.get("Total Vertices"), Some(&0));
                assert_eq!(stats_map.get("Total Edges"), Some(&0));
                assert_eq!(stats_map.get("Total Spaces"), Some(&0));
                assert_eq!(stats_map.get("Total Tags"), Some(&0));
                assert_eq!(stats_map.get("Total Edge Types"), Some(&0));
            }
            _ => panic!("Expected DataSet result"),
        }
    }

    #[test]
    fn test_show_stats_executor_space() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowStatsExecutor::new(2, storage, ShowStatsType::Space, expr_context);

        let result = executor.execute();
        assert!(result.is_ok());

        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::DataSet(dataset) => {
                assert_eq!(
                    dataset.col_names,
                    vec![
                        "Space Name".to_string(),
                        "Space ID".to_string(),
                        "Tags".to_string(),
                        "Edge Types".to_string(),
                    ]
                );
            }
            _ => panic!("Expected DataSet result"),
        }
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowStatsExecutor::new(3, storage, ShowStatsType::Storage, expr_context);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_metadata() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let executor = ShowStatsExecutor::new(4, storage, ShowStatsType::Space, expr_context);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "ShowStatsExecutor");
        assert_eq!(executor.description(), "Shows database statistics");
        assert!(executor.stats().num_rows == 0);
    }

    #[test]
    fn test_show_stats_type_storage() {
        let stats_type = ShowStatsType::Storage;
        assert!(matches!(stats_type, ShowStatsType::Storage));
    }

    #[test]
    fn test_show_stats_type_space() {
        let stats_type = ShowStatsType::Space;
        assert!(matches!(stats_type, ShowStatsType::Space));
    }

    #[test]
    fn test_show_stats_type_clone() {
        let stats_type = ShowStatsType::Storage;
        let cloned = stats_type.clone();
        assert!(matches!(cloned, ShowStatsType::Storage));
    }

    #[test]
    fn test_show_stats_type_debug() {
        let stats_type = ShowStatsType::Space;
        let debug_str = format!("{:?}", stats_type);
        assert!(debug_str.contains("Space"));
    }

    #[test]
    fn test_executor_with_different_ids() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let executor1 = ShowStatsExecutor::new(
            10,
            storage.clone(),
            ShowStatsType::Storage,
            expr_context.clone(),
        );
        let executor2 = ShowStatsExecutor::new(
            20,
            storage.clone(),
            ShowStatsType::Space,
            expr_context.clone(),
        );

        assert_eq!(executor1.id(), 10);
        assert_eq!(executor2.id(), 20);
    }

    #[test]
    fn test_executor_stats_mutable() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowStatsExecutor::new(5, storage, ShowStatsType::Storage, expr_context);

        let stats = executor.stats();
        assert_eq!(stats.num_rows, 0);

        let stats_mut = executor.stats_mut();
        stats_mut.num_rows = 100;

        assert_eq!(executor.stats().num_rows, 100);
    }

    #[test]
    fn test_multiple_executions() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowStatsExecutor::new(6, storage, ShowStatsType::Storage, expr_context);

        let result1 = executor.execute();
        assert!(result1.is_ok());

        let result2 = executor.execute();
        assert!(result2.is_ok());

        match (result1, result2) {
            (
                Ok(crate::query::executor::base::ExecutionResult::DataSet(dataset1)),
                Ok(crate::query::executor::base::ExecutionResult::DataSet(dataset2)),
            ) => {
                assert_eq!(dataset1.col_names, dataset2.col_names);
                assert_eq!(dataset1.rows.len(), dataset2.rows.len());
            }
            _ => panic!("Expected DataSet results"),
        }
    }
}
