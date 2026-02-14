//! UnwindExecutor实现
//!
//! 负责处理列表展开操作，将列表中的每个元素展开为单独的行

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

#[cfg(test)]
use crate::config::test_config::test_config;

/// Unwind执行器
/// 用于将列表中的每个元素展开为单独的行
pub struct UnwindExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    /// 输入变量名
    input_var: String,
    /// 要展开的表达式
    unwind_expression: Expression,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否来自管道
    from_pipe: bool,
}

impl<S: StorageClient + Send + 'static> UnwindExecutor<S> {
    /// 创建新的UnwindExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expression: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UnwindExecutor".to_string(), storage),
            input_var,
            unwind_expression,
            col_names,
            from_pipe,
        }
    }

    /// 带上下文创建UnwindExecutor
    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expression: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "UnwindExecutor".to_string(), storage, context),
            input_var,
            unwind_expression,
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
    fn execute_unwind(&mut self) -> DBResult<DataSet> {
        // 获取输入结果
        let input_result = self
            .base
            .context
            .get_result(&self.input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Input variable '{}' not found",
                    self.input_var
                )))
            })?;

        // 创建表达式上下文
        let mut expr_context = DefaultExpressionContext::new();

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
                    let unwind_value =
                        ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                            .map_err(|e| {
                                DBError::Query(crate::core::error::QueryError::ExecutionError(
                                    e.to_string(),
                                ))
                            })?;

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

                    let unwind_value =
                        ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                            .map_err(|e| {
                                DBError::Query(crate::core::error::QueryError::ExecutionError(
                                    e.to_string(),
                                ))
                            })?;

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

                    let unwind_value =
                        ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                            .map_err(|e| {
                                DBError::Query(crate::core::error::QueryError::ExecutionError(
                                    e.to_string(),
                                ))
                            })?;

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

                let unwind_value =
                    ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context).map_err(
                        |e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        },
                    )?;

                let list_values = self.extract_list(&unwind_value);

                for list_item in list_values {
                    let mut row = Vec::new();
                    row.push(list_item);
                    dataset.rows.push(row);
                }
            }
            ExecutionResult::Empty => {}
            ExecutionResult::Paths(paths) => {
                // 处理路径列表
                for path in paths {
                    let path_value = Value::Path(path.clone());
                    expr_context.set_variable("_".to_string(), path_value.clone());

                    let unwind_value =
                        ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                            .map_err(|e| {
                                DBError::Query(crate::core::error::QueryError::ExecutionError(
                                    e.to_string(),
                                ))
                            })?;

                    let list_values = self.extract_list(&unwind_value);

                    for list_item in list_values {
                        let mut row = Vec::new();

                        if !self.from_pipe {
                            row.push(path_value.clone());
                        }

                        row.push(list_item);

                        dataset.rows.push(row);
                    }
                }
            }
            ExecutionResult::DataSet(ds) => {
                // 处理数据集
                for row in &ds.rows {
                    for value in row {
                        expr_context.set_variable("_".to_string(), value.clone());

                        let unwind_value =
                            ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                                .map_err(|e| {
                                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                                        e.to_string(),
                                    ))
                                })?;

                        let list_values = self.extract_list(&unwind_value);

                        for list_item in list_values {
                            let mut new_row = Vec::new();

                            if !self.from_pipe {
                                new_row.push(value.clone());
                            }

                            new_row.push(list_item);

                            dataset.rows.push(new_row);
                        }
                    }
                }
            }
            ExecutionResult::Count(_) => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Cannot unwind count result".to_string(),
                    ),
                ));
            }
            ExecutionResult::Error(e) => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!(
                        "Error in input result: {}",
                        e
                    )),
                ));
            }
            ExecutionResult::Result(_) => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Cannot unwind Result object".to_string(),
                    ),
                ));
            }
        }

        Ok(dataset)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UnwindExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_unwind()?;
        Ok(ExecutionResult::Values(
            dataset.rows.into_iter().flatten().collect(),
        ))
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

impl<S: StorageClient + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for UnwindExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("UnwindExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::storage::MockStorage;
    use std::sync::Arc;
use parking_lot::Mutex;

    #[tokio::test]
    async fn test_unwind_executor() {
        let _config = test_config();
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建输入数据
        let list_value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let input_result = ExecutionResult::Values(vec![list_value]);

        // 创建执行上下文
        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        // 创建UnwindExecutor
        let unwind_expression = Expression::Variable("_".to_string());
        let mut executor = UnwindExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            unwind_expression,
            vec!["unwound".to_string()],
            false,
            context,
        );

        // 执行展开
        let result = executor
            .execute()
            .expect("Executor should execute successfully");

        // 检查结果
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 6);
            assert_eq!(
                values[0],
                Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
            );
            assert_eq!(values[1], Value::Int(1));
            assert_eq!(
                values[2],
                Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
            );
            assert_eq!(values[3], Value::Int(2));
            assert_eq!(
                values[4],
                Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
            );
            assert_eq!(values[5], Value::Int(3));
        } else {
            panic!("Expected Values result");
        }
    }
}
