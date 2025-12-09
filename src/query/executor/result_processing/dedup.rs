use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use crate::core::Value;
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, InputExecutor};

/// DistinctExecutor - 去重执行器
///
/// 移除结果集中的重复项，通常用于 DISTINCT 子句
pub struct DistinctExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> DistinctExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "DistinctExecutor".to_string(), storage),
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for DistinctExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for DistinctExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 从结果中移除重复项
        let distinct_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                // 在实际实现中，我们会使用更复杂的比较
                // 现在用顶点 ID 作为唯一性标准
                let mut seen_ids = std::collections::HashSet::new();
                let distinct_vertices = vertices
                    .into_iter()
                    .filter(|v| seen_ids.insert(v.vid.clone()))
                    .collect::<Vec<_>>();
                ExecutionResult::Vertices(distinct_vertices)
            }
            ExecutionResult::Edges(edges) => {
                // 在实际实现中，我们会使用更复杂的比较
                // 现在用边的字段组合作为唯一性标准
                let mut seen_edges = std::collections::HashSet::new();
                let distinct_edges = edges
                    .into_iter()
                    .filter(|e| {
                        let edge_key =
                            (e.src.clone(), e.dst.clone(), e.edge_type.clone(), e.ranking);
                        seen_edges.insert(edge_key)
                    })
                    .collect::<Vec<_>>();
                ExecutionResult::Edges(distinct_edges)
            }
            ExecutionResult::Values(values) => {
                // 移除重复值
                let mut seen_values = std::collections::HashSet::new();
                let distinct_values = values
                    .into_iter()
                    .filter(|v| seen_values.insert(v.clone()))
                    .collect::<Vec<_>>();
                ExecutionResult::Values(distinct_values)
            }
            ExecutionResult::Count(count) => {
                // Count 已经是独特的
                ExecutionResult::Count(count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(distinct_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化去重所需的任何资源
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

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}
