use crate::query::context::QueryContext;
use crate::query::executor::{Executor, ExecutorFactory as BaseExecutorFactory};
use crate::query::planner::plan::{ExecutionPlan, PlanNode};
use crate::query::types::{QueryError, QueryResult};
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

/// 执行器工厂 - 负责根据执行计划创建和管理执行器
///
/// 这个类取代了原来的QueryExecutor，现在负责：
/// 1. 根据执行计划创建对应的执行器
/// 2. 管理执行器实例池
/// 3. 协调执行器间的数据流
/// 4. 执行完整的执行计划
pub struct ExecutorFactory<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    base_factory: BaseExecutorFactory,
}

impl<S: StorageEngine> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
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
        &self,
        query_context: &mut QueryContext,
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
    /// * `Ok(Box<dyn Executor>)` - 创建的执行器实例
    /// * `Err(QueryError)` - 创建执行器时的错误
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 基础工厂只接受 plan_node 参数
        let any_executor = self.base_factory.create_executor(plan_node)?;

        // 尝试将 Any 类型转换为 Executor<S>
        match any_executor.downcast::<Box<dyn Executor<S>>>() {
            Ok(executor) => Ok(*executor),
            Err(_) => Err(QueryError::ExecutionError(
                "无法将执行器转换为正确的类型".to_string(),
            )),
        }
    }
}
