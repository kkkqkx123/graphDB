use crate::query::context::QueryContext;
use crate::query::executor::factory::BaseExecutorFactory;
use crate::query::executor::Executor;
use crate::query::planner::plan::{ExecutionPlan, PlanNode};
use crate::query::types::{QueryError, QueryResult};
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

/// 执行器工厂 - 负责根据执行计划创建执行器
///
/// 这个类取代了原来的QueryExecutor，现在负责：
/// 1. 根据执行计划创建对应的执行器
/// 2. 协调执行器间的数据流
/// 3. 执行完整的执行计划
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    storage: Arc<Mutex<S>>,
    base_factory: BaseExecutorFactory<S>,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage: storage.clone(),
            base_factory: BaseExecutorFactory::new(),
        }
    }

    /// 执行完整的执行计划
    ///
    /// # 参数
    /// * `query_context` - 查询上下文
    /// * `plan` - 优化后的执行计划
    ///
    /// # 返回
    /// * `Ok(QueryResult)` - 查询执行结果
    /// * `Err(QueryError)` - 执行过程中的错误
    pub async fn execute_plan(
        &mut self,
        _query_context: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> Result<QueryResult, QueryError> {
        // 获取执行计划的根节点
        let root_node = plan
            .root()
            .as_ref()
            .ok_or_else(|| QueryError::ExecutionError("执行计划为空".to_string()))?;

        // 创建根执行器
        let mut root_executor = self.create_executor(&**root_node)?;

        // 执行查询
        match root_executor.execute().await {
            Ok(execution_result) => {
                // 将 ExecutionResult 转换为 QueryResult
                match execution_result {
                    crate::query::executor::traits::ExecutionResult::Success => {
                        Ok(QueryResult::Success)
                    }
                    crate::query::executor::traits::ExecutionResult::Count(count) => {
                        Ok(QueryResult::Count(count))
                    }
                    crate::query::executor::traits::ExecutionResult::Values(values) => {
                        // 这里需要根据实际需求处理 Values 类型
                        Ok(QueryResult::Count(values.len()))
                    }
                    crate::query::executor::traits::ExecutionResult::Vertices(vertices) => {
                        // 这里需要根据实际需求处理 Vertices 类型
                        Ok(QueryResult::Count(vertices.len()))
                    }
                    crate::query::executor::traits::ExecutionResult::Edges(edges) => {
                        // 这里需要根据实际需求处理 Edges 类型
                        Ok(QueryResult::Count(edges.len()))
                    }
                    crate::query::executor::traits::ExecutionResult::DataSet(dataset) => {
                        // 这里需要根据实际需求处理 DataSet 类型
                        Ok(QueryResult::Count(dataset.rows.len()))
                    }
                    crate::query::executor::traits::ExecutionResult::Paths(paths) => {
                        // 这里需要根据实际需求处理 Paths 类型
                        Ok(QueryResult::Count(paths.len()))
                    }
                    crate::query::executor::traits::ExecutionResult::Error(error_msg) => {
                        Err(QueryError::ExecutionError(error_msg))
                    }
                }
            }
            Err(db_error) => Err(QueryError::ExecutionError(db_error.to_string())),
        }
    }

    /// 根据计划节点创建对应的执行器
    ///
    /// # 参数
    /// * `plan_node` - 计划节点
    ///
    /// # 返回
    /// * `Ok(Box<dyn Executor<S>>)` - 创建的执行器实例
    /// * `Err(QueryError)` - 创建执行器时的错误
    pub fn create_executor(
        &mut self,
        plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 使用重构后的基础工厂创建执行器
        self.base_factory
            .create_executor(plan_node, self.storage.clone())
    }

    /// 验证执行计划的有效性
    ///
    /// # 参数
    /// * `plan` - 执行计划
    ///
    /// # 返回
    /// * `Ok(())` - 计划有效
    /// * `Err(QueryError)` - 计划无效
    pub fn validate_plan(&self, plan: &ExecutionPlan) -> Result<(), QueryError> {
        if plan.root().is_none() {
            return Err(QueryError::ExecutionError("执行计划根节点为空".to_string()));
        }

        // TODO: 添加更多验证逻辑
        // 1. 验证计划节点的依赖关系
        // 2. 验证执行器的兼容性
        // 3. 验证资源需求

        Ok(())
    }
}
