use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, Vertex, Edge};
use crate::storage::StorageEngine;
use crate::query::QueryError;
use crate::query::executor::base::{Executor, ExecutionResult, ExecutionContext, BaseExecutor, InputExecutor, EdgeDirection};

/// ExpandExecutor - 单步路径扩展执行器
///
/// 从当前节点按照指定的边类型和方向扩展一步，获取相邻节点
/// 通常用于图遍历和路径查询
pub struct ExpandExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    edge_direction: EdgeDirection,
    edge_types: Option<Vec<String>>,
    max_depth: Option<usize>,  // 最大扩展深度
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> ExpandExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ExpandExecutor".to_string(), storage),
            edge_direction,
            edge_types,
            max_depth,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for ExpandExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ExpandExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 在实际实现中，这将根据边的方向和类型扩展图遍历
        // 现在返回输入结果不变
        Ok(input_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化扩展所需的任何资源
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
