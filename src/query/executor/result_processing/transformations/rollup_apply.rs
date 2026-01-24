//! RollUpApplyExecutor实现
//!
//! 负责处理聚合操作，将右输入中的值根据左输入的键进行聚合

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, List, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageEngine;

/// RollUpApply执行器
/// 用于将右输入中的值根据左输入的键进行聚合
pub struct RollUpApplyExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    compare_cols: Vec<Expression>,
    collect_col: Expression,
    col_names: Vec<String>,
    movable: bool,
}

impl<S: StorageEngine + Send + 'static> RollUpApplyExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        compare_cols: Vec<Expression>,
        collect_col: Expression,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "RollUpApplyExecutor".to_string(), storage),
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            col_names,
            movable: false,
        }
    }

    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        compare_cols: Vec<Expression>,
        collect_col: Expression,
        col_names: Vec<String>,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(
                id,
                "RollUpApplyExecutor".to_string(),
                storage,
                context,
            ),
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            col_names,
            movable: false,
        }
    }

    fn check_bi_input_data_sets(&self) -> DBResult<()> {
        let _left_result = self
            .base
            .context
            .get_result(&self.left_input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Left input variable '{}' not found",
                    self.left_input_var
                )))
            })?;

        let _right_result = self
            .base
            .context
            .get_result(&self.right_input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Right input variable '{}' not found",
                    self.right_input_var
                )))
            })?;

        Ok(())
    }

    fn build_hash_table<C: ExpressionContext>(
        &self,
        compare_cols: &[Expression],
        collect_col: &Expression,
        iter: &[Value],
        hash_table: &mut HashMap<List, List>,
        expr_context: &mut C,
    ) -> DBResult<()> {
        for value in iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut key_list = List { values: Vec::new() };
            for col in compare_cols {
                let val = ExpressionEvaluator::evaluate(col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;
                key_list.values.push(val);
            }

            let collect_val =
                ExpressionEvaluator::evaluate(collect_col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;

            let entry = hash_table
                .entry(key_list)
                .or_insert_with(|| List { values: Vec::new() });
            entry.values.push(collect_val);
        }

        Ok(())
    }

    fn build_single_key_hash_table<C: ExpressionContext>(
        &self,
        compare_col: &Expression,
        collect_col: &Expression,
        iter: &[Value],
        hash_table: &mut HashMap<Value, List>,
        expr_context: &mut C,
    ) -> DBResult<()> {
        for value in iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let key_val =
                ExpressionEvaluator::evaluate(compare_col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;

            let collect_val =
                ExpressionEvaluator::evaluate(collect_col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;

            let entry = hash_table
                .entry(key_val)
                .or_insert_with(|| List { values: Vec::new() });
            entry.values.push(collect_val);
        }

        Ok(())
    }

    fn build_zero_key_hash_table<C: ExpressionContext>(
        &self,
        collect_col: &Expression,
        iter: &[Value],
        hash_table: &mut List,
        expr_context: &mut C,
    ) -> DBResult<()> {
        hash_table.values.reserve(iter.len());

        for value in iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let collect_val =
                ExpressionEvaluator::evaluate(collect_col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;

            hash_table.values.push(collect_val);
        }

        Ok(())
    }

    fn probe_zero_key<C: ExpressionContext>(
        &self,
        probe_iter: &[Value],
        hash_table: &List,
        expr_context: &mut C,
    ) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut row = Vec::new();

            if self.movable {
                row.push(value.clone());
            }

            row.push(Value::List(hash_table.values.clone()));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    fn probe_single_key<C: ExpressionContext>(
        &self,
        probe_key: &Expression,
        probe_iter: &[Value],
        hash_table: &HashMap<Value, List>,
        expr_context: &mut C,
    ) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let key_val = ExpressionEvaluator::evaluate(probe_key, expr_context).map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })?;

            let vals = hash_table
                .get(&key_val)
                .cloned()
                .unwrap_or(List { values: Vec::new() });

            let mut row = Vec::new();

            if self.movable {
                row.push(value.clone());
            } else {
                row.push(key_val.clone());
            }

            row.push(Value::List(vals.values));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    fn probe<C: ExpressionContext>(
        &self,
        probe_keys: &[Expression],
        probe_iter: &[Value],
        hash_table: &HashMap<List, List>,
        expr_context: &mut C,
    ) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut key_list = List { values: Vec::new() };
            for col in probe_keys {
                let val = ExpressionEvaluator::evaluate(col, expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;
                key_list.values.push(val);
            }

            let vals = hash_table
                .get(&key_list)
                .cloned()
                .unwrap_or(List { values: Vec::new() });

            let mut row = Vec::new();

            if self.movable {
                row.push(value.clone());
            }

            row.push(Value::List(vals.values));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    fn execute_rollup_apply(&mut self) -> DBResult<DataSet> {
        self.check_bi_input_data_sets()?;

        let left_result = self
            .base
            .context
            .get_result(&self.left_input_var)
            .expect("Context should have left result");
        let right_result = self
            .base
            .context
            .get_result(&self.right_input_var)
            .expect("Context should have right result");

        let left_values = match left_result {
            ExecutionResult::Values(values) => values.clone(),
            ExecutionResult::Vertices(vertices) => vertices
                .iter()
                .map(|v| Value::Vertex(Box::new(v.clone())))
                .collect::<Vec<_>>(),
            ExecutionResult::Edges(edges) => edges
                .iter()
                .map(|e| Value::Edge(e.clone()))
                .collect::<Vec<_>>(),
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid left input result type".to_string(),
                    ),
                ))
            }
        };

        let right_values = match right_result {
            ExecutionResult::Values(values) => values.clone(),
            ExecutionResult::Vertices(vertices) => vertices
                .iter()
                .map(|v| Value::Vertex(Box::new(v.clone())))
                .collect::<Vec<_>>(),
            ExecutionResult::Edges(edges) => edges
                .iter()
                .map(|e| Value::Edge(e.clone()))
                .collect::<Vec<_>>(),
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid right input result type".to_string(),
                    ),
                ))
            }
        };

        let mut expr_context = DefaultExpressionContext::new();

        let result = if self.compare_cols.is_empty() {
            let mut hash_table = List { values: Vec::new() };
            self.build_zero_key_hash_table(
                &self.collect_col,
                &right_values,
                &mut hash_table,
                &mut expr_context,
            )?;
            self.probe_zero_key(&left_values, &hash_table, &mut expr_context)?
        } else if self.compare_cols.len() == 1 {
            let mut hash_table = HashMap::new();
            self.build_single_key_hash_table(
                &self.compare_cols[0],
                &self.collect_col,
                &right_values,
                &mut hash_table,
                &mut expr_context,
            )?;
            self.probe_single_key(
                &self.compare_cols[0],
                &left_values,
                &hash_table,
                &mut expr_context,
            )?
        } else {
            let mut hash_table = HashMap::new();
            self.build_hash_table(
                &self.compare_cols,
                &self.collect_col,
                &right_values,
                &mut hash_table,
                &mut expr_context,
            )?;
            self.probe(
                &self.compare_cols,
                &left_values,
                &hash_table,
                &mut expr_context,
            )?
        };

        Ok(result)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for RollUpApplyExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_rollup_apply()?;

        let values: Vec<Value> = dataset
            .rows
            .into_iter()
            .flat_map(|row| row.into_iter())
            .collect();

        Ok(ExecutionResult::Values(values))
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
    for RollUpApplyExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("RollUpApplyExecutor storage should be set")
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
    async fn test_rollup_apply_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_values = vec![Value::Int(1), Value::Int(2)];
        let right_values = vec![Value::Int(1), Value::Int(1), Value::Int(2)];

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values.clone()),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values.clone()),
        );

        let compare_cols = vec![Expression::variable("_")];
        let collect_col = Expression::variable("_");

        let mut executor = RollUpApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["key".to_string(), "collected".to_string()],
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 4);
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_rollup_apply_zero_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_values = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let right_values = vec![Value::Int(10), Value::Int(20)];

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values.clone()),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values.clone()),
        );

        let compare_cols: Vec<Expression> = vec![];
        let collect_col = Expression::Variable("_".to_string());

        let mut executor = RollUpApplyExecutor::with_context(
            2,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["collected".to_string()],
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 3);
            assert_eq!(values.len(), 3);
            for val in &values {
                match val {
                    Value::List(list) => {
                        assert_eq!(list.len(), 2);
                    }
                    _ => panic!("Expected List value"),
                }
            }
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_rollup_apply_multi_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_values = vec![
            Value::from((1, "A")),
            Value::from((1, "B")),
            Value::from((2, "A")),
        ];
        let right_values = vec![
            Value::from((1, "A")),
            Value::from((1, "B")),
            Value::from((1, "C")),
            Value::from((2, "A")),
        ];

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values.clone()),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values.clone()),
        );

        let compare_cols = vec![
            Expression::subscript(Expression::variable("_"), Expression::literal(0i64)),
            Expression::subscript(Expression::variable("_"), Expression::literal(1i64)),
        ];
        let collect_col = Expression::Variable("_".to_string());

        let mut executor = RollUpApplyExecutor::with_context(
            3,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["key0".to_string(), "key1".to_string(), "collected".to_string()],
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 3);
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_rollup_apply_empty_right() {
        let _config = test_config();
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_values = vec![Value::Int(1), Value::Int(2)];
        let right_values: Vec<Value> = vec![];

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values.clone()),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values.clone()),
        );

        let compare_cols = vec![Expression::variable("_")];
        let collect_col = Expression::Variable("_".to_string());

        let mut executor = RollUpApplyExecutor::with_context(
            4,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["key".to_string(), "collected".to_string()],
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 4);
            assert_eq!(values[0], Value::Int(1));
            assert_eq!(values[1], Value::List(Vec::new()));
            assert_eq!(values[2], Value::Int(2));
            assert_eq!(values[3], Value::List(Vec::new()));
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_rollup_apply_empty_left() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_values: Vec<Value> = vec![];
        let right_values = vec![Value::Int(1), Value::Int(2)];

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values.clone()),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values.clone()),
        );

        let compare_cols = vec![Expression::literal(0i64)];
        let collect_col = Expression::Variable("_".to_string());

        let mut executor = RollUpApplyExecutor::with_context(
            5,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["key".to_string(), "collected".to_string()],
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert!(values.is_empty());
        } else {
            panic!("Expected Values result");
        }
    }
}
