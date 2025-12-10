use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use crate::core::Value;
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, InputExecutor};

/// LimitExecutor - 结果数量限制执行器
///
/// 将结果集限制为指定的最大数量，通常用于 LIMIT 子句
pub struct LimitExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    limit: usize,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> LimitExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>, limit: usize) -> Self {
        Self {
            base: BaseExecutor::new(id, "LimitExecutor".to_string(), storage),
            limit,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for LimitExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for LimitExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 对结果应用限制
        let limited_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let limited_vertices = vertices.into_iter().take(self.limit).collect::<Vec<_>>();
                ExecutionResult::Vertices(limited_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let limited_edges = edges.into_iter().take(self.limit).collect::<Vec<_>>();
                ExecutionResult::Edges(limited_edges)
            }
            ExecutionResult::Values(values) => {
                let limited_values = values.into_iter().take(self.limit).collect::<Vec<_>>();
                ExecutionResult::Values(limited_values)
            }
            ExecutionResult::Paths(paths) => {
                let limited_paths = paths.into_iter().take(self.limit).collect::<Vec<_>>();
                ExecutionResult::Paths(limited_paths)
            }
            ExecutionResult::DataSet(dataset) => {
                let limited_rows = dataset.rows.into_iter().take(self.limit).collect::<Vec<_>>();
                ExecutionResult::DataSet(crate::core::value::DataSet {
                    col_names: dataset.col_names,
                    rows: limited_rows,
                })
            }
            ExecutionResult::Count(count) => {
                let limited_count = std::cmp::min(count, self.limit);
                ExecutionResult::Count(limited_count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(limited_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化限制所需的任何资源
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

/// OffsetExecutor - 结果偏移执行器
///
/// 跳过指定数量的结果，通常用于 OFFSET 子句
pub struct OffsetExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    offset: usize,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> OffsetExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>, offset: usize) -> Self {
        Self {
            base: BaseExecutor::new(id, "OffsetExecutor".to_string(), storage),
            offset,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for OffsetExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for OffsetExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 对结果应用偏移
        let offset_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let offset_vertices = vertices.into_iter().skip(self.offset).collect::<Vec<_>>();
                ExecutionResult::Vertices(offset_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let offset_edges = edges.into_iter().skip(self.offset).collect::<Vec<_>>();
                ExecutionResult::Edges(offset_edges)
            }
            ExecutionResult::Values(values) => {
                let offset_values = values.into_iter().skip(self.offset).collect::<Vec<_>>();
                ExecutionResult::Values(offset_values)
            }
            ExecutionResult::Paths(paths) => {
                let offset_paths = paths.into_iter().skip(self.offset).collect::<Vec<_>>();
                ExecutionResult::Paths(offset_paths)
            }
            ExecutionResult::DataSet(dataset) => {
                let offset_rows = dataset.rows.into_iter().skip(self.offset).collect::<Vec<_>>();
                ExecutionResult::DataSet(crate::core::value::DataSet {
                    col_names: dataset.col_names,
                    rows: offset_rows,
                })
            }
            ExecutionResult::Count(count) => {
                let offset_count = if count > self.offset {
                    count - self.offset
                } else {
                    0
                };
                ExecutionResult::Count(offset_count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(offset_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化偏移所需的任何资源
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
