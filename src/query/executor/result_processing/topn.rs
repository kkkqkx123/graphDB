use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::traits::{Executor, ExecutionResult, ExecutorCore, ExecutorLifecycle, ExecutorMetadata};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// TopNExecutor - TOP N 结果执行器
///
/// 返回排序后的前 N 个结果，是 Sort + Limit 的优化版本
pub struct TopNExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    n: usize,                  // 返回的结果数量
    #[allow(dead_code)]
    sort_columns: Vec<String>, // 排序列
    ascending: bool,           // 排序方向
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> TopNExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        n: usize,
        sort_columns: Vec<String>,
        ascending: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "TopNExecutor".to_string(), storage),
            n,
            sort_columns,
            ascending,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for TopNExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for TopNExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 排序并取前 N 个结果
        let top_n_result = match input_result {
            ExecutionResult::Vertices(mut vertices) => {
                // 在实际实现中，我们会根据指定的属性排序
                // 现在只按顶点 ID 排序作为例子
                if self.ascending {
                    vertices.sort_by(|a, b| a.vid.cmp(&b.vid));
                } else {
                    vertices.sort_by(|a, b| b.vid.cmp(&a.vid));
                }
                let top_vertices = vertices.into_iter().take(self.n).collect::<Vec<_>>();
                ExecutionResult::Vertices(top_vertices)
            }
            ExecutionResult::Edges(mut edges) => {
                // 对于边，我们可能按源顶点 ID 或其他属性排序
                if self.ascending {
                    edges.sort_by(|a, b| a.src.cmp(&b.src));
                } else {
                    edges.sort_by(|a, b| b.src.cmp(&a.src));
                }
                let top_edges = edges.into_iter().take(self.n).collect::<Vec<_>>();
                ExecutionResult::Edges(top_edges)
            }
            ExecutionResult::Values(values) => {
                // 对于值，我们可能需要进行比较（如果可能）
                // 现在只取前 N 个值
                let top_values = values.into_iter().take(self.n).collect::<Vec<_>>();
                ExecutionResult::Values(top_values)
            }
            ExecutionResult::Paths(mut paths) => {
                // 对路径进行排序（按路径长度或其他标准）
                if self.ascending {
                    paths.sort_by(|a, b| a.len().cmp(&b.len()));
                } else {
                    paths.sort_by(|a, b| b.len().cmp(&a.len()));
                }
                let top_paths = paths.into_iter().take(self.n).collect::<Vec<_>>();
                ExecutionResult::Paths(top_paths)
            }
            ExecutionResult::DataSet(mut dataset) => {
                // 对数据集的行进行排序（按第一列或其他标准）
                if !dataset.rows.is_empty() {
                    if self.ascending {
                        dataset.rows.sort_by(|a, b| a.cmp(b));
                    } else {
                        dataset.rows.sort_by(|a, b| b.cmp(a));
                    }
                }
                let top_rows = dataset.rows.into_iter().take(self.n).collect::<Vec<_>>();
                ExecutionResult::DataSet(crate::core::value::DataSet {
                    col_names: dataset.col_names,
                    rows: top_rows,
                })
            }
            ExecutionResult::Count(count) => {
                // 对于计数，返回 count 和 N 的最小值
                ExecutionResult::Count(std::cmp::min(count, self.n))
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(top_n_result)
    }
}

impl<S: StorageEngine> ExecutorLifecycle for TopNExecutor<S> {
    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化 TopN 操作所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for TopNExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for TopNExecutor<S> {
    fn storage(&self) -> &S {
        &self.base.storage
    }
}
