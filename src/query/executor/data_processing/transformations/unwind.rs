//! UnwindExecutor实现
//!
//! 负责处理列表展开操作，将列表中的每个元素展开为单独的行

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::config::test_config::test_config;
use crate::core::{DataSet, Value};
use crate::graph::expression::{Expression, ExpressionContext};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// Unwind执行器
/// 用于将列表中的每个元素展开为单独的行
pub struct UnwindExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 输入变量名
    input_var: String,
    /// 要展开的表达式
    unwind_expr: Expression,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否来自管道
    from_pipe: bool,
}

impl<S: StorageEngine + Send + 'static> UnwindExecutor<S> {
    /// 创建新的UnwindExecutor
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expr: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UnwindExecutor".to_string(), storage),
            input_var,
            unwind_expr,
            col_names,
            from_pipe,
        }
    }

    /// 带上下文创建UnwindExecutor
    pub fn with_context(
        id: usize,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expr: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "UnwindExecutor".to_string(), storage, context),
            input_var,
            unwind_expr,
            col_names,
            from_pipe,
        }
    }

    /// 从值中提取列表
    fn extract_list(&self, val: &Value) -> Vec<Value> {
        match val {
            Value::List(list) => list.clone(),
            Value::Null(_) | Value::Empty => vec![],
            _ => vec![val.clone()],
        }
    }

    /// 执行展开操作
    fn execute_unwind(&mut self) -> Result<DataSet, QueryError> {
        // 获取输入结果
        let input_result = self
            .base
            .context
            .get_result(&self.input_var)
            .ok_or_else(|| {
                QueryError::ExecutionError(format!("Input variable '{}' not found", self.input_var))
            })?;

        // 创建表达式上下文
        let mut expr_context = ExpressionContext::new();

        // 从执行上下文中设置变量
        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

        // 创建输出数据集
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        // 根据输入结果类型处理
        match input_result {
            ExecutionResult::Values(values) => {
                // 处理值列表
                for value in values {
                    // 设置当前行到表达式上下文
                    expr_context.set_variable("_".to_string(), value.clone());

                    // 计算展开表达式
                    let unwind_value = self
                        .unwind_expr
                        .evaluate(&expr_context)
                        .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

                    // 提取列表
                    let list_values = self.extract_list(&unwind_value);

                    // 为每个列表元素创建一行
                    for list_item in list_values {
                        let mut row = Vec::new();

                        // 如果不是来自管道且输入不为空，保留原始值
                        if !self.from_pipe {
                            row.push(value.clone());
                        }

                        // 添加展开的值
                        row.push(list_item);

                        dataset.rows.push(row);
                    }
                }
            }
            ExecutionResult::Vertices(vertices) => {
                // 处理顶点列表
                for vertex in vertices {
                    let vertex_value = Value::Vertex(Box::new(vertex.clone()));
                    expr_context.set_variable("_".to_string(), vertex_value.clone());

                    let unwind_value = self
                        .unwind_expr
                        .evaluate(&expr_context)
                        .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

                    let list_values = self.extract_list(&unwind_value);

                    for list_item in list_values {
                        let mut row = Vec::new();

                        if !self.from_pipe {
                            row.push(vertex_value.clone());
                        }

                        row.push(list_item);

                        dataset.rows.push(row);
                    }
                }
            }
            ExecutionResult::Edges(edges) => {
                // 处理边列表
                for edge in edges {
                    let edge_value = Value::Edge(edge.clone());
                    expr_context.set_variable("_".to_string(), edge_value.clone());

                    let unwind_value = self
                        .unwind_expr
                        .evaluate(&expr_context)
                        .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

                    let list_values = self.extract_list(&unwind_value);

                    for list_item in list_values {
                        let mut row = Vec::new();

                        if !self.from_pipe {
                            row.push(edge_value.clone());
                        }

                        row.push(list_item);

                        dataset.rows.push(row);
                    }
                }
            }
            ExecutionResult::Success => {
                // 处理空输入
                let empty_value = Value::Empty;
                expr_context.set_variable("_".to_string(), empty_value.clone());

                let unwind_value = self
                    .unwind_expr
                    .evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

                let list_values = self.extract_list(&unwind_value);

                for list_item in list_values {
                    let mut row = Vec::new();
                    row.push(list_item);
                    dataset.rows.push(row);
                }
            }
            ExecutionResult::Count(_) => {
                return Err(QueryError::ExecutionError(
                    "Cannot unwind count result".to_string(),
                ));
            }
        }

        Ok(dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for UnwindExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 执行展开操作
        let dataset = self.execute_unwind()?;

        // 返回展开后的数据集 - 平铺所有值
        Ok(ExecutionResult::Values(
            dataset.rows.into_iter().flatten().collect(),
        ))
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
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_unwind_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            NativeStorage::new(config.test_db_path("test_db_unwind")).unwrap(),
        ));

        // 创建输入数据
        let list_value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let input_result = ExecutionResult::Values(vec![list_value]);

        // 创建执行上下文
        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        // 创建UnwindExecutor
        let unwind_expr = Expression::Variable("_".to_string());
        let mut executor = UnwindExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            unwind_expr,
            vec!["unwound".to_string()],
            false,
            context,
        );

        // 执行展开
        let result = executor.execute().await.unwrap();

        // 检查结果
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 6);
            // UNWIND的模式是：[原始列表, 元素1, 原始列表, 元素2, 原始列表, 元素3]
            assert_eq!(values[0], Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
            assert_eq!(values[1], Value::Int(1));
            assert_eq!(values[2], Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
            assert_eq!(values[3], Value::Int(2));
            assert_eq!(values[4], Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
            assert_eq!(values[5], Value::Int(3));
        } else {
            panic!("Expected Values result");
        }
    }
}
