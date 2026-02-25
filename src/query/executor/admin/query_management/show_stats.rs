//! ShowStatsExecutor - 显示统计执行器
//!
//! 负责显示数据库的统计信息。

use std::sync::Arc;
use parking_lot::Mutex;

use crate::query::{GLOBAL_QUERY_MANAGER, QueryStatus};
use crate::core::StatsManager;
use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 显示统计类型
#[derive(Debug, Clone)]
pub enum ShowStatsType {
    /// 显示所有统计
    All,
    /// 显示查询统计
    Query,
    /// 显示存储统计
    Storage,
    /// 显示空间统计
    Space,
    /// 显示最近查询
    RecentQueries { limit: usize },
    /// 显示慢查询
    SlowQueries { limit: usize },
    /// 显示执行器统计
    Executors,
    /// 显示指定查询详情
    QueryDetail { trace_id: String },
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
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage),
            stats_type: ShowStatsType::All,
        }
    }

    pub fn with_type(id: i64, storage: Arc<Mutex<S>>, stats_type: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage),
            stats_type: Self::parse_stats_type(&stats_type),
        }
    }

    pub fn with_stats_type(id: i64, storage: Arc<Mutex<S>>, stats_type: ShowStatsType) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowStatsExecutor".to_string(), storage),
            stats_type,
        }
    }

    /// 解析统计类型字符串
    fn parse_stats_type(stats_type: &str) -> ShowStatsType {
        let parts: Vec<&str> = stats_type.split_whitespace().collect();
        
        match parts.as_slice() {
            ["queries"] | ["queries", "recent"] => ShowStatsType::RecentQueries { limit: 10 },
            ["queries", "recent", limit] => ShowStatsType::RecentQueries { 
                limit: limit.parse().unwrap_or(10) 
            },
            ["slow", "queries"] => ShowStatsType::SlowQueries { limit: 10 },
            ["slow", "queries", limit] => ShowStatsType::SlowQueries { 
                limit: limit.parse().unwrap_or(10) 
            },
            ["executors"] => ShowStatsType::Executors,
            ["query", trace_id] => ShowStatsType::QueryDetail { 
                trace_id: trace_id.to_string() 
            },
            ["query"] => ShowStatsType::Query,
            ["storage"] => ShowStatsType::Storage,
            ["space"] => ShowStatsType::Space,
            _ => ShowStatsType::All,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowStatsExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock();

        let dataset = match &self.stats_type {
            ShowStatsType::All => self.show_all_stats(&*storage_guard),
            ShowStatsType::Query => self.show_query_stats(),
            ShowStatsType::Storage => self.show_storage_stats(&*storage_guard),
            ShowStatsType::Space => self.show_space_stats(&*storage_guard),
            ShowStatsType::RecentQueries { limit } => self.show_recent_queries(*limit),
            ShowStatsType::SlowQueries { limit } => self.show_slow_queries(*limit),
            ShowStatsType::Executors => self.show_executor_stats(),
            ShowStatsType::QueryDetail { trace_id } => self.show_query_detail(trace_id),
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
        let query_stats = GLOBAL_QUERY_MANAGER
            .get()
            .and_then(|qm| qm.get_query_stats().ok())
            .unwrap_or_default();
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
        let query_stats = GLOBAL_QUERY_MANAGER
            .get()
            .and_then(|qm| qm.get_query_stats().ok())
            .unwrap_or_default();

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

    fn show_recent_queries(&self, limit: usize) -> DataSet {
        let rows = if let Some(stats_manager) = Self::get_stats_manager() {
            stats_manager
                .get_recent_queries(limit)
                .into_iter()
                .map(|profile| {
                    vec![
                        Value::String(profile.trace_id),
                        Value::Int(profile.session_id),
                        Value::String(Self::truncate_query(&profile.query_text, 50)),
                        Value::Int(profile.total_duration_ms as i64),
                        Value::Int(profile.result_count as i64),
                        Value::String(match profile.status {
                            QueryStatus::Success => "SUCCESS".to_string(),
                            QueryStatus::Failed => "FAILED".to_string(),
                        }),
                    ]
                })
                .collect()
        } else {
            vec![]
        };

        DataSet {
            col_names: vec![
                "Trace ID".to_string(),
                "Session ID".to_string(),
                "Query".to_string(),
                "Duration (ms)".to_string(),
                "Rows".to_string(),
                "Status".to_string(),
            ],
            rows,
        }
    }

    fn show_slow_queries(&self, limit: usize) -> DataSet {
        let rows = if let Some(stats_manager) = Self::get_stats_manager() {
            stats_manager
                .get_slow_queries(limit)
                .into_iter()
                .map(|profile| {
                    vec![
                        Value::String(profile.trace_id),
                        Value::Int(profile.session_id),
                        Value::String(Self::truncate_query(&profile.query_text, 50)),
                        Value::Int(profile.total_duration_ms as i64),
                        Value::Int(profile.stages.parse_ms as i64),
                        Value::Int(profile.stages.execute_ms as i64),
                    ]
                })
                .collect()
        } else {
            vec![]
        };

        DataSet {
            col_names: vec![
                "Trace ID".to_string(),
                "Session ID".to_string(),
                "Query".to_string(),
                "Total (ms)".to_string(),
                "Parse (ms)".to_string(),
                "Execute (ms)".to_string(),
            ],
            rows,
        }
    }

    fn show_executor_stats(&self) -> DataSet {
        let rows = if let Some(stats_manager) = Self::get_stats_manager() {
            stats_manager
                .get_executor_stats_summary()
                .into_iter()
                .map(|(executor_type, (total_time, total_rows, count))| {
                    let avg_time = if count > 0 { total_time / count as u64 } else { 0 };
                    vec![
                        Value::String(executor_type),
                        Value::Int(count as i64),
                        Value::Int(total_time as i64),
                        Value::Int(avg_time as i64),
                        Value::Int(total_rows as i64),
                    ]
                })
                .collect()
        } else {
            vec![]
        };

        DataSet {
            col_names: vec![
                "Executor Type".to_string(),
                "Count".to_string(),
                "Total Time (ms)".to_string(),
                "Avg Time (ms)".to_string(),
                "Total Rows".to_string(),
            ],
            rows,
        }
    }

    fn show_query_detail(&self, trace_id: &str) -> DataSet {
        if let Some(stats_manager) = Self::get_stats_manager() {
            if let Some(profile) = stats_manager.get_query_profile(trace_id) {
                let mut rows = vec![
                    vec![
                        Value::String("Trace ID".to_string()),
                        Value::String(profile.trace_id),
                    ],
                    vec![
                        Value::String("Session ID".to_string()),
                        Value::Int(profile.session_id),
                    ],
                    vec![
                        Value::String("Query".to_string()),
                        Value::String(profile.query_text),
                    ],
                    vec![
                        Value::String("Total Duration".to_string()),
                        Value::Int(profile.total_duration_ms as i64),
                    ],
                    vec![
                        Value::String("Parse Time".to_string()),
                        Value::Int(profile.stages.parse_ms as i64),
                    ],
                    vec![
                        Value::String("Validate Time".to_string()),
                        Value::Int(profile.stages.validate_ms as i64),
                    ],
                    vec![
                        Value::String("Plan Time".to_string()),
                        Value::Int(profile.stages.plan_ms as i64),
                    ],
                    vec![
                        Value::String("Optimize Time".to_string()),
                        Value::Int(profile.stages.optimize_ms as i64),
                    ],
                    vec![
                        Value::String("Execute Time".to_string()),
                        Value::Int(profile.stages.execute_ms as i64),
                    ],
                    vec![
                        Value::String("Result Count".to_string()),
                        Value::Int(profile.result_count as i64),
                    ],
                    vec![
                        Value::String("Status".to_string()),
                        Value::String(match profile.status {
                            QueryStatus::Success => "SUCCESS".to_string(),
                            QueryStatus::Failed => "FAILED".to_string(),
                        }),
                    ],
                ];

                // 添加执行器统计
                for (i, exec_stat) in profile.executor_stats.iter().enumerate() {
                    rows.push(vec![
                        Value::String(format!("Executor {}", i + 1)),
                        Value::String(format!(
                            "{} (id={}) {}ms rows={}",
                            exec_stat.executor_type,
                            exec_stat.executor_id,
                            exec_stat.duration_ms,
                            exec_stat.rows_processed
                        )),
                    ]);
                }

                if let Some(error) = profile.error_message {
                    rows.push(vec![
                        Value::String("Error".to_string()),
                        Value::String(error),
                    ]);
                }

                return DataSet {
                    col_names: vec!["Property".to_string(), "Value".to_string()],
                    rows,
                };
            }
        }

        // 查询未找到
        DataSet {
            col_names: vec!["Message".to_string()],
            rows: vec![vec![Value::String(format!("Query {} not found in cache", trace_id))]],
        }
    }

    /// 获取 StatsManager 实例
    fn get_stats_manager() -> Option<Arc<StatsManager>> {
        // 从全局服务管理器获取，这里简化处理
        // 实际实现中应该从全局状态获取
        None
    }

    /// 截断查询文本
    fn truncate_query(query: &str, max_len: usize) -> String {
        if query.len() <= max_len {
            query.to_string()
        } else {
            format!("{}...", &query[..max_len])
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
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[test]
    fn test_show_stats_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = ShowStatsExecutor::new(1, storage);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_stats_executor_with_type() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = ShowStatsExecutor::with_type(2, storage, "query".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = ShowStatsExecutor::new(3, storage);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let executor = ShowStatsExecutor::new(4, storage);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "ShowStatsExecutor");
        assert_eq!(executor.description(), "Shows database statistics");
        assert!(executor.stats().num_rows == 0);
    }
}
