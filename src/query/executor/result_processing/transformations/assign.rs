//! AssignExecutor实现
//!
//! 负责处理变量赋值操作，将表达式的结果赋值给变量

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageEngine;

/// Assign执行器
/// 用于将表达式的结果赋值给变量
pub struct AssignExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 赋值项列表 (变量名, 表达式)
    assign_items: Vec<(String, Expression)>,
}

impl<S: StorageEngine + Send + 'static> AssignExecutor<S> {
    /// 创建新的AssignExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, assign_items: Vec<(String, Expression)>) -> Self {
        Self {
            base: BaseExecutor::with_description(
                id,
                "AssignExecutor".to_string(),
                "Assign executor - assigns expression results to variables".to_string(),
                storage,
            ),
            assign_items,
        }
    }

    /// 带上下文创建AssignExecutor
    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        assign_items: Vec<(String, Expression)>,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context_and_description(
                id,
                "AssignExecutor".to_string(),
                "Assign executor - assigns expression results to variables".to_string(),
                storage,
                context,
            ),
            assign_items,
        }
    }

    /// 执行赋值操作
    fn execute_assign(&mut self) -> DBResult<()> {
        let mut expr_context = DefaultExpressionContext::new();

        // 执行每个赋值项
        for (var_name, expression) in &self.assign_items {
            // 计算表达式的值
            let value = ExpressionEvaluator::evaluate(expression, &mut expr_context).map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })?;

            // 根据值的类型设置到执行上下文中
            match &value {
                Value::DataSet(dataset) => {
                    // 如果是数据集，创建一个Values结果
                    let values: Vec<Value> = dataset
                        .rows
                        .iter()
                        .flat_map(|row| row.iter().cloned())
                        .collect();
                    self.base
                        .context
                        .set_result(var_name.clone(), ExecutionResult::Values(values));
                }
                _ => {
                    // 其他类型直接设置为结果
                    self.base
                        .context
                        .set_result(var_name.clone(), ExecutionResult::Values(vec![value.clone()]));
                }
            }

            // 同时设置变量以便后续使用
            self.base.context.set_variable(var_name.clone(), value.clone());

            // 同时更新表达式上下文，以便后续表达式可以使用这个变量
            expr_context.set_variable(var_name.clone(), value);
        }

        Ok(())
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AssignExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.execute_assign()?;
        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for AssignExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("AssignExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config::test_config;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::storage::MockStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_assign_executor() {
        let _config = test_config();
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建赋值项
        let assign_items = vec![
            ("var1".to_string(), Expression::literal(42i64)),
            ("var2".to_string(), Expression::literal("hello")),
        ];

        let mut executor = AssignExecutor::new(1, storage, assign_items);

        // 执行赋值
        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");
        assert!(matches!(result, ExecutionResult::Success));

        // 检查变量是否正确设置
        assert_eq!(
            executor.base.context.get_variable("var1"),
            Some(&Value::Int(42))
        );
        assert_eq!(
            executor.base.context.get_variable("var2"),
            Some(&Value::String("hello".to_string()))
        );
    }
}
