//! MaterializeExecutor - 物化执行器
//!
//! 将输入数据物化（缓存）到内存中，用于优化多次引用的CTE或子查询
//! 避免重复计算，提高查询性能

use parking_lot::Mutex;
use std::sync::Arc;

use crate::query::executor::base::InputExecutor;
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageClient;

/// 物化状态
#[derive(Debug, Clone, PartialEq)]
pub enum MaterializeState {
    /// 尚未物化
    Uninitialized,
    /// 已物化，数据可用
    Materialized,
    /// 物化失败
    Failed(String),
}

/// MaterializeExecutor - 物化执行器
///
/// 将输入数据缓存到内存中，支持多次读取
/// 主要用于优化被多次引用的CTE或子查询
pub struct MaterializeExecutor<S: StorageClient + Send + 'static> {
    /// 基础执行器
    base: BaseExecutor<S>,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 物化状态
    state: MaterializeState,
    /// 物化的数据
    materialized_data: Option<ExecutionResult>,
    /// 内存限制（字节）
    memory_limit: usize,
    /// 当前内存使用量
    current_memory_usage: usize,
    /// 是否已消耗（用于单次消费模式）
    consumed: bool,
}

impl<S: StorageClient + Send + 'static> MaterializeExecutor<S> {
    /// 创建新的物化执行器
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        memory_limit: Option<usize>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
    ) -> Self {
        let base = BaseExecutor::new(
            id,
            "MaterializeExecutor".to_string(),
            storage,
            expr_context,
        );

        Self {
            base,
            input_executor: None,
            state: MaterializeState::Uninitialized,
            materialized_data: None,
            memory_limit: memory_limit.unwrap_or(100 * 1024 * 1024), // 默认100MB
            current_memory_usage: 0,
            consumed: false,
        }
    }

    /// 设置内存限制
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// 获取物化状态
    pub fn state(&self) -> &MaterializeState {
        &self.state
    }

    /// 检查是否已物化
    pub fn is_materialized(&self) -> bool {
        matches!(self.state, MaterializeState::Materialized)
    }

    /// 获取物化数据（如果已物化）
    pub fn get_materialized_data(&self) -> Option<&ExecutionResult> {
        self.materialized_data.as_ref()
    }

    /// 重置消费状态，允许重新读取物化数据
    pub fn reset_consumed(&mut self) {
        self.consumed = false;
    }

    /// 物化输入数据
    fn materialize_input(&mut self) -> DBResult<()> {
        if self.is_materialized() {
            return Ok(());
        }

        let input = self.input_executor.as_mut()
            .ok_or_else(|| crate::core::DBError::Query(
                crate::core::QueryError::ExecutionError("物化执行器缺少输入".to_string())
            ))?;

        let result = input.execute()?;
        
        // 估算内存使用量
        self.current_memory_usage = self.estimate_memory_usage(&result);
        
        if self.current_memory_usage > self.memory_limit {
            self.state = MaterializeState::Failed(
                format!("物化数据大小({} bytes)超过内存限制({} bytes)",
                    self.current_memory_usage, self.memory_limit)
            );
            return Err(crate::core::DBError::Query(
                crate::core::QueryError::ExecutionError("物化数据超过内存限制".to_string())
            ));
        }

        self.materialized_data = Some(result);
        self.state = MaterializeState::Materialized;
        
        Ok(())
    }

    /// 估算执行结果的内存使用量
    fn estimate_memory_usage(&self, result: &ExecutionResult) -> usize {
        match result {
            ExecutionResult::Empty => 0,
            ExecutionResult::Values(values) => {
                values.iter()
                    .map(|v| std::mem::size_of_val(v))
                    .sum()
            }
            ExecutionResult::Vertices(vertices) => {
                vertices.iter()
                    .map(|v| std::mem::size_of_val(v))
                    .sum()
            }
            ExecutionResult::Edges(edges) => {
                edges.iter()
                    .map(|e| std::mem::size_of_val(e))
                    .sum()
            }
            ExecutionResult::Paths(paths) => {
                paths.iter()
                    .map(|p| std::mem::size_of_val(p))
                    .sum()
            }
            ExecutionResult::DataSet(_dataset) => {
                // 估算数据集大小
                1024 // 简化估算
            }
            _ => 1024, // 默认估算
        }
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for MaterializeExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 如果尚未物化，先物化数据
        if !self.is_materialized() {
            self.materialize_input()?;
        }

        // 返回物化数据的克隆
        self.materialized_data.clone()
            .ok_or_else(|| crate::core::DBError::Query(
                crate::core::QueryError::ExecutionError("物化数据不可用".to_string())
            ))
    }

    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input) = self.input_executor {
            input.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input) = self.input_executor {
            input.close()?;
        }
        // 清理物化数据以释放内存
        self.materialized_data = None;
        self.state = MaterializeState::Uninitialized;
        self.current_memory_usage = 0;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.input_executor.as_ref()
            .map(|input| input.is_open())
            .unwrap_or(false)
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        "MaterializeExecutor"
    }

    fn description(&self) -> &str {
        "Materializes input data to memory for reuse"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for MaterializeExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_ref()
            .map(|boxed| boxed.as_ref())
    }
}

#[cfg(test)]
mod tests {
    // 由于需要 StorageClient，这里只进行编译时检查
    // 实际测试应该在集成测试中进行
}