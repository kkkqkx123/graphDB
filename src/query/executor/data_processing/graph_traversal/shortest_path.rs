use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, Vertex, Edge};
use crate::storage::StorageEngine;
use crate::query::QueryError;
use crate::query::executor::base::{Executor, ExecutionResult, ExecutionContext, BaseExecutor, InputExecutor, EdgeDirection};

/// 最短路径算法枚举
#[derive(Debug, Clone)]
pub enum ShortestPathAlgorithm {
    /// Dijkstra 算法
    Dijkstra,
    /// BFS 广度优先搜索
    BFS,
    /// A* 算法
    AStar,
}

/// ShortestPathExecutor - 最短路径执行器
///
/// 计算两个节点之间的最短路径，支持多种算法
/// 适用于社交网络、路线规划等场景
pub struct ShortestPathExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    start_vertex_ids: Vec<Value>,
    end_vertex_ids: Vec<Value>,
    edge_direction: EdgeDirection,
    edge_types: Option<Vec<String>>,
    algorithm: ShortestPathAlgorithm,  // 使用的算法
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> ShortestPathExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        start_vertex_ids: Vec<Value>,
        end_vertex_ids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        algorithm: ShortestPathAlgorithm,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShortestPathExecutor".to_string(), storage),
            start_vertex_ids,
            end_vertex_ids,
            edge_direction,
            edge_types,
            algorithm,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for ShortestPathExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ShortestPathExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 在实际实现中，这将根据选定的算法计算最短路径
        // 现在返回输入结果不变
        Ok(input_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化最短路径计算所需的任何资源
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
