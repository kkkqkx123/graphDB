//! 内连接执行器实现
//!
//! 实现基于哈希的内连接算法，支持单键和多键连接

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Expression, Value};
use crate::expression::context::row_context::RowExpressionContext;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::data_processing::join::base_join::BaseJoinExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageClient;

/// 内连接执行器
pub struct InnerJoinExecutor<S: StorageClient> {
    base_executor: BaseJoinExecutor<S>,
    single_key_hash_table: Option<HashMap<Value, Vec<Vec<Value>>>>,
    multi_key_hash_table: Option<HashMap<Vec<Value>, Vec<Vec<Value>>>>,
    use_multi_key: bool,
}

impl<S: StorageClient> std::fmt::Debug for InnerJoinExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerJoinExecutor")
            .field("base_executor", &"BaseJoinExecutor<S>")
            .field("single_key_hash_table", &self.single_key_hash_table.is_some())
            .field("multi_key_hash_table", &self.multi_key_hash_table.is_some())
            .field("use_multi_key", &self.use_multi_key)
            .finish()
    }
}

impl<S: StorageClient> InnerJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        let use_multi_key = hash_keys.len() > 1;
        Self {
            base_executor: BaseJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
            single_key_hash_table: None,
            multi_key_hash_table: None,
            use_multi_key,
        }
    }

    /// 执行单键内连接（使用表达式求值）
    fn execute_single_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        self.base_executor.optimize_join_order(left_dataset, right_dataset);
        let exchange = self.base_executor.is_exchanged();

        let hash_keys = self.base_executor.get_hash_keys().clone();
        let probe_keys = self.base_executor.get_probe_keys().clone();

        if hash_keys.is_empty() || probe_keys.is_empty() {
            return Err(QueryError::ExecutionError(
                "哈希键或探测键为空".to_string(),
            ));
        }

        let hash_key = hash_keys[0].clone();
        let probe_key = probe_keys[0].clone();

        let (build_dataset, probe_dataset, build_col_names, probe_col_names) = if exchange {
            (
                right_dataset,
                left_dataset,
                &right_dataset.col_names,
                &left_dataset.col_names,
            )
        } else {
            (
                left_dataset,
                right_dataset,
                &left_dataset.col_names,
                &right_dataset.col_names,
            )
        };

        let mut hash_table: HashMap<Value, Vec<Vec<Value>>> = HashMap::new();

        for row in &build_dataset.rows {
            let mut context = RowExpressionContext::from_dataset(row, build_col_names);
            let key = ExpressionEvaluator::evaluate(&hash_key, &mut context)
                .map_err(|e| QueryError::ExecutionError(format!("键求值失败: {}", e)))?;

            hash_table
                .entry(key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();
        let output_col_names = result.col_names.clone();

        for probe_row in &probe_dataset.rows {
            let mut context = RowExpressionContext::from_dataset(probe_row, probe_col_names);
            let probe_key_val = match ExpressionEvaluator::evaluate(&probe_key, &mut context) {
                Ok(k) => k,
                Err(_) => continue,
            };

            if let Some(matching_rows) = hash_table.get(&probe_key_val) {
                for build_row in matching_rows {
                    let new_row = Self::build_join_result_row(
                        build_row,
                        probe_row,
                        build_col_names,
                        probe_col_names,
                        &output_col_names,
                    );
                    result.rows.push(new_row);
                }
            }
        }

        Ok(result)
    }

    /// 根据输出列名构建连接结果行
    fn build_join_result_row(
        left_row: &[Value],
        right_row: &[Value],
        left_col_names: &[String],
        right_col_names: &[String],
        output_col_names: &[String],
    ) -> Vec<Value> {
        let mut result = Vec::with_capacity(output_col_names.len());

        for col_name in output_col_names {
            if let Some(idx) = left_col_names.iter().position(|c| c == col_name) {
                if let Some(val) = left_row.get(idx) {
                    result.push(val.clone());
                }
            } else if let Some(idx) = right_col_names.iter().position(|c| c == col_name) {
                if let Some(val) = right_row.get(idx) {
                    result.push(val.clone());
                }
            }
        }

        result
    }

    /// 执行多键内连接（使用表达式求值）
    fn execute_multi_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        self.base_executor.optimize_join_order(left_dataset, right_dataset);
        let exchange = self.base_executor.is_exchanged();

        let hash_keys = self.base_executor.get_hash_keys().clone();
        let probe_keys = self.base_executor.get_probe_keys().clone();

        if hash_keys.is_empty() || probe_keys.is_empty() {
            return Err(QueryError::ExecutionError(
                "哈希键或探测键为空".to_string(),
            ));
        }

        let (build_dataset, probe_dataset, build_col_names, probe_col_names) = if exchange {
            (
                right_dataset,
                left_dataset,
                &right_dataset.col_names,
                &left_dataset.col_names,
            )
        } else {
            (
                left_dataset,
                right_dataset,
                &left_dataset.col_names,
                &right_dataset.col_names,
            )
        };

        let mut hash_table: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();

        for row in &build_dataset.rows {
            let mut context = RowExpressionContext::from_dataset(row, build_col_names);
            let mut key_values = Vec::with_capacity(hash_keys.len());

            for hash_key in &hash_keys {
                let key = ExpressionEvaluator::evaluate(hash_key, &mut context)
                    .map_err(|e| QueryError::ExecutionError(format!("键求值失败: {}", e)))?;
                key_values.push(key);
            }

            hash_table
                .entry(key_values)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();
        let output_col_names = result.col_names.clone();

        for probe_row in &probe_dataset.rows {
            let mut context = RowExpressionContext::from_dataset(probe_row, probe_col_names);
            let mut key_values = Vec::with_capacity(probe_keys.len());

            for probe_key in &probe_keys {
                let key = ExpressionEvaluator::evaluate(probe_key, &mut context)
                    .map_err(|e| QueryError::ExecutionError(format!("键求值失败: {}", e)))?;
                key_values.push(key);
            }

            if let Some(matching_rows) = hash_table.get(&key_values) {
                for build_row in matching_rows {
                    let new_row = Self::build_join_result_row(
                        build_row,
                        probe_row,
                        build_col_names,
                        probe_col_names,
                        &output_col_names,
                    );
                    result.rows.push(new_row);
                }
            }
        }

        Ok(result)
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for InnerJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let (left_dataset, right_dataset) = self
            .base_executor
            .check_input_datasets()
            .map_err(DBError::from)?;

        if left_dataset.rows.is_empty() || right_dataset.rows.is_empty() {
            let empty_result = DataSet {
                col_names: self.base_executor.get_col_names().clone(),
                rows: Vec::new(),
            };
            return Ok(ExecutionResult::Values(vec![Value::DataSet(empty_result)]));
        }

        let result = if self.use_multi_key {
            self.execute_multi_key_join(&left_dataset, &right_dataset)
                .map_err(DBError::from)?
        } else {
            self.execute_single_key_join(&left_dataset, &right_dataset)
                .map_err(DBError::from)?
        };

        self.base_executor
            .get_base_mut()
            .get_stats_mut()
            .add_row(result.rows.len());

        Ok(ExecutionResult::Values(vec![Value::DataSet(result)]))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.single_key_hash_table = None;
        self.multi_key_hash_table = None;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base_executor.get_base().is_open()
    }

    fn id(&self) -> i64 {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        &self.base_executor.get_base().name
    }

    fn description(&self) -> &str {
        &self.base_executor.get_base().description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base_executor.get_base().get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base_executor.get_base_mut().get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for InnerJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base_executor
            .get_base()
            .storage
            .as_ref()
            .expect("InnerJoinExecutor storage should be set")
    }
}

#[derive(Debug)]
pub struct HashInnerJoinExecutor<S: StorageClient> {
    inner: InnerJoinExecutor<S>,
}

impl<S: StorageClient> HashInnerJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            inner: InnerJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
        }
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for HashInnerJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.inner.execute()
    }

    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "HashInnerJoinExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.inner.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.inner.stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for HashInnerJoinExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::storage::test_mock::MockStorage;

    fn create_test_datasets() -> (DataSet, DataSet) {
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "age".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::Int(25)],
                vec![Value::Int(2), Value::Int(30)],
                vec![Value::Int(3), Value::Int(35)],
            ],
        };

        (left_dataset, right_dataset)
    }

    #[tokio::test]
    async fn test_inner_join_single_key_with_expression() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = InnerJoinExecutor::new(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![Expression::variable("id")],
            vec![Expression::variable("id")],
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        let (left_dataset, right_dataset) = create_test_datasets();

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        let result = executor.execute().await.expect("执行失败");

        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 2);
                    assert_eq!(dataset.rows[0][0], Value::Int(1));
                    assert_eq!(
                        dataset.rows[0][1],
                        Value::String("Alice".to_string())
                    );
                    assert_eq!(dataset.rows[0][2], Value::Int(25));
                    assert_eq!(dataset.rows[1][0], Value::Int(2));
                    assert_eq!(
                        dataset.rows[1][1],
                        Value::String("Bob".to_string())
                    );
                    assert_eq!(dataset.rows[1][2], Value::Int(30));
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_inner_join_multi_key() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_dataset = DataSet {
            col_names: vec!["a".to_string(), "b".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::Int(10), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::Int(20), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["a".to_string(), "b".to_string(), "age".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::Int(10), Value::Int(25)],
                vec![Value::Int(1), Value::Int(11), Value::Int(26)],
                vec![Value::Int(2), Value::Int(20), Value::Int(30)],
            ],
        };

        let mut executor = InnerJoinExecutor::new(
            2,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![
                Expression::variable("a"),
                Expression::variable("b"),
            ],
            vec![
                Expression::variable("a"),
                Expression::variable("b"),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "name".to_string(),
                "age".to_string(),
            ],
        );

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        let result = executor.execute().await.expect("执行失败");

        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 2);
                    assert_eq!(dataset.rows[0][2], Value::String("Alice".to_string()));
                    assert_eq!(dataset.rows[0][3], Value::Int(25));
                    assert_eq!(dataset.rows[1][2], Value::String("Bob".to_string()));
                    assert_eq!(dataset.rows[1][3], Value::Int(30));
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_inner_join_empty_dataset() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "age".to_string()],
            rows: vec![vec![Value::Int(1), Value::Int(25)]],
        };

        let mut executor = InnerJoinExecutor::new(
            3,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![Expression::variable("id")],
            vec![Expression::variable("id")],
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        let result = executor.execute().await.expect("执行失败");

        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 0);
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_inner_join_with_variable_expression() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let mut executor = InnerJoinExecutor::new(
            4,
            storage,
            "left".to_string(),
            "right".to_string(),
            vec![Expression::Variable("id".to_string())],
            vec![Expression::Variable("id".to_string())],
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        let (left_dataset, right_dataset) = create_test_datasets();

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        let result = executor.execute().await.expect("执行失败");

        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 2);
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }
}
