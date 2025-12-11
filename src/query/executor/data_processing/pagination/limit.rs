use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 限制执行器 - 实现LIMIT和OFFSET功能
pub struct LimitExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 限制数量
    limit: Option<usize>,
    /// 偏移量
    offset: usize,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine + Send + 'static> LimitExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        limit: Option<usize>,
        offset: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "LimitExecutor".to_string(), storage),
            limit,
            offset,
            input_executor: None,
        }
    }

    /// 仅设置LIMIT
    pub fn with_limit(
        id: usize,
        storage: Arc<Mutex<S>>,
        limit: usize,
    ) -> Self {
        Self::new(id, storage, Some(limit), 0)
    }

    /// 仅设置OFFSET
    pub fn with_offset(
        id: usize,
        storage: Arc<Mutex<S>>,
        offset: usize,
    ) -> Self {
        Self::new(id, storage, None, offset)
    }

    /// 处理输入数据并应用限制
    async fn process_input(&mut self) -> Result<DataSet, QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;
            
            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    // 应用偏移量
                    if self.offset > 0 {
                        if self.offset < data_set.rows.len() {
                            data_set.rows.drain(0..self.offset);
                        } else {
                            data_set.rows.clear();
                        }
                    }
                    
                    // 应用限制
                    if let Some(limit) = self.limit {
                        data_set.rows.truncate(limit);
                    }
                    
                    Ok(data_set)
                }
                ExecutionResult::Values(mut values) => {
                    // 对值列表应用限制
                    if self.offset > 0 {
                        if self.offset < values.len() {
                            values.drain(0..self.offset);
                        } else {
                            values.clear();
                        }
                    }
                    
                    if let Some(limit) = self.limit {
                        values.truncate(limit);
                    }
                    
                    Ok(DataSet {
                        col_names: vec!["_value".to_string()], // 单列数据
                        rows: values.into_iter().map(|v| vec![v]).collect(),
                    })
                }
                ExecutionResult::Vertices(mut vertices) => {
                    // 对顶点列表应用限制
                    if self.offset > 0 {
                        if self.offset < vertices.len() {
                            vertices.drain(0..self.offset);
                        } else {
                            vertices.clear();
                        }
                    }
                    
                    if let Some(limit) = self.limit {
                        vertices.truncate(limit);
                    }
                    
                    // 将顶点转换为数据集
                    let rows: Vec<Vec<Value>> = vertices.into_iter()
                        .map(|v| vec![Value::Vertex(Box::new(v))])
                        .collect();
                    
                    Ok(DataSet {
                        col_names: vec!["_vertex".to_string()],
                        rows,
                    })
                }
                ExecutionResult::Edges(mut edges) => {
                    // 对边列表应用限制
                    if self.offset > 0 {
                        if self.offset < edges.len() {
                            edges.drain(0..self.offset);
                        } else {
                            edges.clear();
                        }
                    }
                    
                    if let Some(limit) = self.limit {
                        edges.truncate(limit);
                    }
                    
                    // 将边转换为数据集
                    let rows: Vec<Vec<Value>> = edges.into_iter()
                        .map(|e| vec![Value::Edge(e)])
                        .collect();
                    
                    Ok(DataSet {
                        col_names: vec!["_edge".to_string()],
                        rows,
                    })
                }
                _ => Err(QueryError::ExecutionError("Limit executor expects DataSet, Values, Vertices, or Edges input".to_string())),
            }
        } else {
            Err(QueryError::ExecutionError("Limit executor requires input executor".to_string()))
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for LimitExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn open(&mut self) -> Result<(), QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
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

impl<S: StorageEngine + Send + 'static> crate::query::executor::base::InputExecutor<S> for LimitExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}