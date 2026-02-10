//! PatternApplyExecutor 实现
//!
//! 负责处理模式匹配操作，支持 EXISTS 和 NOT EXISTS 语义
//! 将左输入数据与右输入数据进行键匹配

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, List, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::DefaultExpressionContext;
use crate::expression::ExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

fn execution_result_to_values(result: &ExecutionResult) -> Result<Vec<Value>, DBError> {
    match result {
        ExecutionResult::Values(values) => Ok(values.clone()),
        ExecutionResult::Vertices(vertices) => Ok(vertices
            .iter()
            .map(|v| Value::Vertex(Box::new(v.clone())))
            .collect()),
        ExecutionResult::Edges(edges) => Ok(edges
            .iter()
            .map(|e| Value::Edge(e.clone()))
            .collect()),
        _ => Err(DBError::Query(
            crate::core::error::QueryError::ExecutionError(
                "Unsupported result type for PatternApply".to_string(),
            ),
        )),
    }
}

pub struct PatternApplyExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    key_cols: Vec<Expression>,
    col_names: Vec<String>,
    is_anti_predicate: bool,
}

impl<S: StorageClient + Send + 'static> PatternApplyExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        key_cols: Vec<Expression>,
        col_names: Vec<String>,
        is_anti_predicate: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "PatternApplyExecutor".to_string(), storage),
            left_input_var,
            right_input_var,
            key_cols,
            col_names,
            is_anti_predicate,
        }
    }

    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        key_cols: Vec<Expression>,
        col_names: Vec<String>,
        is_anti_predicate: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(
                id,
                "PatternApplyExecutor".to_string(),
                storage,
                context,
            ),
            left_input_var,
            right_input_var,
            key_cols,
            col_names,
            is_anti_predicate,
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

    fn collect_valid_keys(&self, values: &[Value]) -> DBResult<HashSet<List>> {
        let mut valid_keys = HashSet::new();
        let mut expr_context = DefaultExpressionContext::new();

        for value in values {
            expr_context.set_variable("_".to_string(), value.clone());

            if self.key_cols.is_empty() {
                continue;
            }

            let mut key_list = List {
                values: Vec::with_capacity(self.key_cols.len()),
            };

            for col in &self.key_cols {
                let val = ExpressionEvaluator::evaluate(col, &mut expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string()))
                })?;
                key_list.values.push(val);
            }

            valid_keys.insert(key_list);
        }

        Ok(valid_keys)
    }

    fn collect_valid_single_key(&self, values: &[Value]) -> DBResult<HashSet<Value>> {
        let mut valid_keys = HashSet::new();
        let mut expr_context = DefaultExpressionContext::new();

        for value in values {
            expr_context.set_variable("_".to_string(), value.clone());

            if self.key_cols.is_empty() {
                continue;
            }

            let val = ExpressionEvaluator::evaluate(&self.key_cols[0], &mut expr_context)
                .map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string()))
                })?;
            valid_keys.insert(val);
        }

        Ok(valid_keys)
    }

    fn execute_pattern_apply(&mut self) -> DBResult<DataSet> {
        self.check_bi_input_data_sets()?;

        let left_result = self
            .base
            .context
            .get_result(&self.left_input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Left input variable '{}' not found",
                    self.left_input_var
                )))
            })?;

        let right_result = self
            .base
            .context
            .get_result(&self.right_input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Right input variable '{}' not found",
                    self.right_input_var
                )))
            })?;

        let left_values = execution_result_to_values(&left_result)?;
        let right_values = execution_result_to_values(&right_result)?;

        let mut expr_context = DefaultExpressionContext::new();

        let result = if self.key_cols.is_empty() {
            let all_valid = !right_values.is_empty();
            let final_valid = all_valid ^ self.is_anti_predicate;
            self.apply_zero_key(&left_values, final_valid)
        } else if self.key_cols.len() == 1 {
            let valid_keys = self.collect_valid_single_key(&right_values)?;
            self.apply_single_key(&left_values, &valid_keys, &mut expr_context)?
        } else {
            let valid_keys = self.collect_valid_keys(&right_values)?;
            self.apply_multi_key(&left_values, &valid_keys, &mut expr_context)?
        };

        Ok(result)
    }

    fn apply_zero_key(&self, left_values: &[Value], all_valid: bool) -> DataSet {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        if all_valid {
            for value in left_values {
                dataset.rows.push(vec![value.clone()]);
            }
        }

        dataset
    }

    fn apply_single_key<C: ExpressionContext + Send>(
        &self,
        left_values: &[Value],
        valid_keys: &HashSet<Value>,
        expr_context: &mut C,
    ) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        for value in left_values {
            expr_context.set_variable("_".to_string(), value.clone());

            let key_val = ExpressionEvaluator::evaluate(&self.key_cols[0], expr_context)
                .map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string()))
                })?;

            let apply_flag = (valid_keys.contains(&key_val)) ^ self.is_anti_predicate;

            if apply_flag {
                dataset.rows.push(vec![value.clone()]);
            }
        }

        Ok(dataset)
    }

    fn apply_multi_key<C: ExpressionContext + Send>(
        &self,
        left_values: &[Value],
        valid_keys: &HashSet<List>,
        expr_context: &mut C,
    ) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        for value in left_values {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut key_list = List {
                values: Vec::with_capacity(self.key_cols.len()),
            };

            for col in &self.key_cols {
                let val = ExpressionEvaluator::evaluate(col, expr_context)
                    .map_err(|e| {
                        DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string()))
                    })?;
                key_list.values.push(val);
            }

            let apply_flag = (valid_keys.contains(&key_list)) ^ self.is_anti_predicate;

            if apply_flag {
                dataset.rows.push(vec![value.clone()]);
            }
        }

        Ok(dataset)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for PatternApplyExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_pattern_apply()?;

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

impl<S: StorageClient + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for PatternApplyExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("PatternApplyExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MockStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_pattern_apply_single_key_positive() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut context = crate::query::executor::base::ExecutionContext::new();

        let left_values = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let right_values = vec![Value::Int(2), Value::Int(4)];

        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values),
        );

        let key_cols = vec![Expression::variable("_")];
        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            key_cols,
            vec!["matched".to_string()],
            false,
            context,
        );

        let result = executor.execute().await.unwrap();
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], Value::Int(2));
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_pattern_apply_single_key_anti_predicate() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut context = crate::query::executor::base::ExecutionContext::new();

        let left_values = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let right_values = vec![Value::Int(2), Value::Int(4)];

        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values),
        );

        let key_cols = vec![Expression::variable("_")];
        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            key_cols,
            vec!["matched".to_string()],
            true,
            context,
        );

        let result = executor.execute().await.unwrap();
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 2);
            assert!(values.contains(&Value::Int(1)));
            assert!(values.contains(&Value::Int(3)));
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_pattern_apply_zero_key_exists() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut context = crate::query::executor::base::ExecutionContext::new();

        let left_values = vec![Value::Int(1), Value::Int(2)];
        let right_values = vec![Value::Int(10), Value::Int(20)];

        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values),
        );

        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![],
            vec!["matched".to_string()],
            false,
            context,
        );

        let result = executor.execute().await.unwrap();
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 2);
        } else {
            panic!("Expected Values result");
        }
    }

    #[tokio::test]
    async fn test_pattern_apply_zero_key_not_exists() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut context = crate::query::executor::base::ExecutionContext::new();

        let left_values = vec![Value::Int(1), Value::Int(2)];
        let right_values: Vec<Value> = vec![];

        context.set_result(
            "left".to_string(),
            ExecutionResult::Values(left_values),
        );
        context.set_result(
            "right".to_string(),
            ExecutionResult::Values(right_values),
        );

        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![],
            vec!["matched".to_string()],
            false,
            context,
        );

        let result = executor.execute().await.unwrap();
        if let ExecutionResult::Values(values) = result {
            assert!(values.is_empty());
        } else {
            panic!("Expected Values result");
        }
    }
}
