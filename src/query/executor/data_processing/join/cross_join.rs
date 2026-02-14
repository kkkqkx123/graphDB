//! 笛卡尔积执行器实现
//!
//! 实现笛卡尔积（交叉连接）算法，支持多表连接

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Value};
use crate::query::executor::data_processing::join::base_join::BaseJoinExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageClient;

/// 笛卡尔积执行器
pub struct CrossJoinExecutor<S: StorageClient> {
    base_executor: BaseJoinExecutor<S>,
    /// 输入变量列表（支持多表）
    input_vars: Vec<String>,
}

// Manual Debug implementation for CrossJoinExecutor to avoid requiring Debug trait for BaseJoinExecutor
impl<S: StorageClient> std::fmt::Debug for CrossJoinExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossJoinExecutor")
            .field("base_executor", &"BaseJoinExecutor<S>")
            .field("input_vars", &self.input_vars)
            .finish()
    }
}

impl<S: StorageClient> CrossJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_vars: Vec<String>,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            base_executor: BaseJoinExecutor::new(
                id,
                storage,
                String::new(), // 左变量（不使用）
                String::new(), // 右变量（不使用）
                Vec::new(),    // 哈希键（不使用）
                Vec::new(),    // 探测键（不使用）
                col_names,
            ),
            input_vars,
        }
    }

    /// 执行两表笛卡尔积
    fn execute_two_way_cartesian_product(
        &self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> Result<DataSet, QueryError> {
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        // 计算结果集大小并预分配
        let estimated_size = left_dataset.rows.len() * right_dataset.rows.len();
        if estimated_size > 0 {
            result.rows.reserve(estimated_size);
        }

        // 执行笛卡尔积
        for left_row in &left_dataset.rows {
            for right_row in &right_dataset.rows {
                let mut new_row = left_row.clone();
                new_row.extend(right_row.clone());
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }

    /// 优化的笛卡尔积实现（使用迭代器避免中间结果集）
    fn execute_optimized_cartesian_product(&self) -> Result<DataSet, QueryError> {
        if self.input_vars.len() < 2 {
            return Err(QueryError::ExecutionError(
                "笛卡尔积需要至少两个输入".to_string(),
            ));
        }

        // 获取所有输入数据集
        let mut datasets = Vec::new();
        for var in &self.input_vars {
            let result = self
                .base_executor
                .get_base()
                .context
                .get_result(var)
                .ok_or_else(|| QueryError::ExecutionError(format!("找不到输入变量: {}", var)))?;

            let dataset = match result {
                ExecutionResult::Values(values) => {
                    if let Some(Value::DataSet(dataset)) = values.first() {
                        dataset.clone()
                    } else {
                        return Err(QueryError::ExecutionError(format!(
                            "输入变量 {} 不是有效的数据集",
                            var
                        )));
                    }
                }
                _ => {
                    return Err(QueryError::ExecutionError(format!(
                        "输入变量 {} 不是有效的数据集",
                        var
                    )))
                }
            };

            datasets.push(dataset);
        }

        // 检查是否有空集
        for dataset in &datasets {
            if dataset.rows.is_empty() {
                return Ok(DataSet {
                    col_names: self.base_executor.get_col_names().clone(),
                    rows: Vec::new(),
                });
            }
        }

        // 计算结果集大小
        let total_size: usize = datasets.iter().map(|ds| ds.rows.len()).product();

        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        if total_size > 0 {
            result.rows.reserve(total_size);
        }

        // 使用递归或迭代方式生成笛卡尔积
        self.generate_cartesian_product_recursive(&datasets, 0, Vec::new(), &mut result);

        Ok(result)
    }

    /// 递归生成笛卡尔积
    fn generate_cartesian_product_recursive(
        &self,
        datasets: &[DataSet],
        current_index: usize,
        current_row: Vec<Value>,
        result: &mut DataSet,
    ) {
        if current_index >= datasets.len() {
            // 到达最后一个数据集，添加完整行到结果
            result.rows.push(current_row);
            return;
        }

        // 遍历当前数据集的每一行
        for row in &datasets[current_index].rows {
            let mut new_row = current_row.clone();
            new_row.extend(row.clone());
            self.generate_cartesian_product_recursive(datasets, current_index + 1, new_row, result);
        }
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for CrossJoinExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 根据输入数量选择实现方式
        let result = if self.input_vars.len() == 2 {
            // 两表笛卡尔积
            let left_var = &self.input_vars[0];
            let right_var = &self.input_vars[1];

            let left_result = self
                .base_executor
                .get_base()
                .context
                .get_result(left_var)
                .ok_or_else(|| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "找不到左输入变量: {}",
                        left_var
                    )))
                })?;

            let right_result = self
                .base_executor
                .get_base()
                .context
                .get_result(right_var)
                .ok_or_else(|| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                        "找不到右输入变量: {}",
                        right_var
                    )))
                })?;

            let left_dataset = match left_result {
                ExecutionResult::Values(values) => {
                    if let Some(Value::DataSet(dataset)) = values.first() {
                        dataset.clone()
                    } else {
                        return Err(DBError::Query(
                            crate::core::error::QueryError::ExecutionError(
                                "左输入不是有效的数据集".to_string(),
                            ),
                        ));
                    }
                }
                _ => {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(
                            "左输入不是有效的数据集".to_string(),
                        ),
                    ))
                }
            };

            let right_dataset = match right_result {
                ExecutionResult::Values(values) => {
                    if let Some(Value::DataSet(dataset)) = values.first() {
                        dataset.clone()
                    } else {
                        return Err(DBError::Query(
                            crate::core::error::QueryError::ExecutionError(
                                "右输入不是有效的数据集".to_string(),
                            ),
                        ));
                    }
                }
                _ => {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(
                            "右输入不是有效的数据集".to_string(),
                        ),
                    ))
                }
            };

            self.execute_two_way_cartesian_product(&left_dataset, &right_dataset)
                .map_err(DBError::from)?
        } else {
            // 多表笛卡尔积
            self.execute_optimized_cartesian_product()
                .map_err(DBError::from)?
        };

        Ok(ExecutionResult::Values(vec![Value::DataSet(result)]))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base_executor.get_base().is_open()
    }

    fn id(&self) -> i64 {
        self.base_executor.get_base().id
    }

    fn name(&self) -> &str {
        "CrossJoinExecutor"
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

impl<S: StorageClient + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for CrossJoinExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base_executor
            .get_base()
            .storage
            .as_ref()
            .expect("CrossJoinExecutor storage should be set")
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::core::Value;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_cross_join_two_tables() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = CrossJoinExecutor::new(
            1,
            storage,
            vec!["left".to_string(), "right".to_string()],
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "city".to_string(),
            ],
        );

        // 设置执行上下文
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["age".to_string(), "city".to_string()],
            rows: vec![
                vec![Value::Int(25), Value::String("New York".to_string())],
                vec![Value::Int(30), Value::String("London".to_string())],
            ],
        };

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行连接
        let result = executor.execute().expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 4); // 2 * 2 = 4 行结果

                    // 验证第一行
                    assert_eq!(
                        dataset.rows[0],
                        vec![
                            Value::Int(1),
                            Value::String("Alice".to_string()),
                            Value::Int(25),
                            Value::String("New York".to_string()),
                        ]
                    );

                    // 验证最后一行
                    assert_eq!(
                        dataset.rows[3],
                        vec![
                            Value::Int(2),
                            Value::String("Bob".to_string()),
                            Value::Int(30),
                            Value::String("London".to_string()),
                        ]
                    );
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_cross_join_empty_table() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = CrossJoinExecutor::new(
            1,
            storage,
            vec!["left".to_string(), "right".to_string()],
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        // 设置执行上下文
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![Value::Int(1), Value::String("Alice".to_string())]],
        };

        let right_dataset = DataSet {
            col_names: vec!["age".to_string()],
            rows: Vec::new(), // 空右表
        };

        executor.base_executor.get_base_mut().context.set_result(
            "left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行连接
        let result = executor.execute().expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 0); // 空结果
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }

    #[tokio::test]
    async fn test_cross_join_three_tables() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建执行器
        let mut executor = CrossJoinExecutor::new(
            1,
            storage,
            vec![
                "table1".to_string(),
                "table2".to_string(),
                "table3".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "f".to_string(),
            ],
        );

        // 设置执行上下文
        let table1 = DataSet {
            col_names: vec!["a".to_string()],
            rows: vec![vec![Value::Int(1)]],
        };

        let table2 = DataSet {
            col_names: vec!["b".to_string(), "c".to_string()],
            rows: vec![vec![Value::Int(2), Value::Int(3)]],
        };

        let table3 = DataSet {
            col_names: vec!["d".to_string(), "e".to_string(), "f".to_string()],
            rows: vec![vec![Value::Int(4), Value::Int(5), Value::Int(6)]],
        };

        executor.base_executor.get_base_mut().context.set_result(
            "table1".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(table1)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "table2".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(table2)]),
        );

        executor.base_executor.get_base_mut().context.set_result(
            "table3".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(table3)]),
        );

        // 执行连接
        let result = executor.execute().expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                if let Some(Value::DataSet(dataset)) = values.first() {
                    assert_eq!(dataset.rows.len(), 1); // 1 * 1 * 1 = 1 行结果
                    assert_eq!(
                        dataset.rows[0],
                        vec![
                            Value::Int(1),
                            Value::Int(2),
                            Value::Int(3),
                            Value::Int(4),
                            Value::Int(5),
                            Value::Int(6)
                        ]
                    );
                } else {
                    panic!("期望DataSet结果");
                }
            }
            _ => panic!("期望Values结果"),
        }
    }
}
