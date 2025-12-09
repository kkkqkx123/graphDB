//! AssignExecutor实现
//! 
//! 负责处理变量赋值操作，将表达式的结果赋值给变量

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::Value;
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::graph::expression::{Expression, ExpressionContext};

/// Assign执行器
/// 用于将表达式的结果赋值给变量
pub struct AssignExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 赋值项列表 (变量名, 表达式)
    assign_items: Vec<(String, Expression)>,
}

impl<S: StorageEngine + Send + 'static> AssignExecutor<S> {
    /// 创建新的AssignExecutor
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        assign_items: Vec<(String, Expression)>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AssignExecutor".to_string(), storage),
            assign_items,
        }
    }

    /// 带上下文创建AssignExecutor
    pub fn with_context(
        id: usize,
        storage: Arc<Mutex<S>>,
        assign_items: Vec<(String, Expression)>,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "AssignExecutor".to_string(), storage, context),
            assign_items,
        }
    }

    /// 执行赋值操作
    fn execute_assign(&mut self) -> Result<(), QueryError> {
        let mut expr_context = ExpressionContext::new();
        
        // 从执行上下文中设置变量
        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

        // 执行每个赋值项
        for (var_name, expr) in &self.assign_items {
            // 计算表达式的值
            let value = expr.evaluate(&expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 根据值的类型设置到执行上下文中
            match &value {
                Value::DataSet(dataset) => {
                    // 如果是数据集，创建一个Values结果
                    let values: Vec<Value> = dataset.rows.iter()
                        .flat_map(|row| row.iter().cloned())
                        .collect();
                    self.base.context.set_result(var_name.clone(), ExecutionResult::Values(values));
                },
                _ => {
                    // 其他类型直接设置为变量
                    self.base.context.set_variable(var_name.clone(), value.clone());
                }
            }

            // 同时更新表达式上下文，以便后续表达式可以使用这个变量
            expr_context.set_variable(var_name.clone(), value);
        }

        Ok(())
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for AssignExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 执行赋值操作
        self.execute_assign()?;
        
        // AssignExecutor通常返回Success，表示赋值操作完成
        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化资源
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::graph::expression::Expression;
    use crate::storage::NativeStorage;
    use crate::config::test_config::test_config;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_assign_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(NativeStorage::new(config.test_db_path("test_db_assign")).unwrap()));
        
        // 创建赋值项
        let assign_items = vec![
            ("var1".to_string(), Expression::Constant(Value::Int(42))),
            ("var2".to_string(), Expression::Constant(Value::String("hello".to_string()))),
        ];

        let mut executor = AssignExecutor::new(1, storage, assign_items);
        
        // 执行赋值
        let result = executor.execute().await.unwrap();
        assert!(matches!(result, ExecutionResult::Success));
        
        // 检查变量是否正确设置
        assert_eq!(executor.base.context.get_variable("var1"), Some(&Value::Int(42)));
        assert_eq!(executor.base.context.get_variable("var2"), Some(&Value::String("hello".to_string())));
    }
}