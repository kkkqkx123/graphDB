//! Union执行器实现
//!
//! 实现UNION操作，合并两个数据集并去除重复行

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::{DataSet, Value};
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageClient;

use super::base::SetExecutor;

/// Union执行器
///
/// 实现UNION操作，合并两个数据集并去除重复行
/// 类似于SQL的UNION（不是UNION ALL）
#[derive(Debug)]
pub struct UnionExecutor<S: StorageClient> {
    pub set_executor: SetExecutor<S>,
}

impl<S: StorageClient> UnionExecutor<S> {
    /// 创建新的Union执行器
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
    ) -> Self {
        Self {
            set_executor: SetExecutor::new(
                id,
                "UnionExecutor".to_string(),
                storage,
                left_input_var,
                right_input_var,
            ),
        }
    }

    /// 执行UNION操作
    ///
    /// 算法步骤：
    /// 1. 获取左右两个输入数据集
    /// 2. 验证列名是否一致
    /// 3. 合并两个数据集的所有行
    fn execute_union(&mut self) -> Result<DataSet, QueryError> {
        // 获取左右输入数据集
        let left_dataset = self.set_executor.get_left_input_data()?;
        let right_dataset = self.set_executor.get_right_input_data()?;

        // 检查输入数据集的有效性
        self.set_executor
            .check_input_data_sets(&left_dataset, &right_dataset)?;

        // 合并两个数据集
        let combined_dataset = SetExecutor::<S>::concat_datasets(left_dataset, right_dataset);

        // 去除重复行
        let deduped_rows = SetExecutor::<S>::dedup_rows(combined_dataset.rows);

        // 构建结果数据集
        let result_dataset = DataSet {
            col_names: self.set_executor.get_col_names().clone(),
            rows: deduped_rows,
        };

        Ok(result_dataset)
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for UnionExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_union().map_err(|e| {
            crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                e.to_string(),
            ))
        })?;

        let values: Vec<Value> = dataset
            .rows
            .into_iter()
            .flat_map(|row| row.into_iter())
            .collect();

        Ok(ExecutionResult::Values(values))
    }

    fn open(&mut self) -> DBResult<()> {
        self.set_executor.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.set_executor.close()
    }

    fn is_open(&self) -> bool {
        self.set_executor.is_open()
    }

    fn id(&self) -> i64 {
        self.set_executor.id()
    }

    fn name(&self) -> &str {
        self.set_executor.name()
    }

    fn description(&self) -> &str {
        self.set_executor.description()
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.set_executor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.set_executor.stats_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    // 创建测试用的存储引擎
    fn create_test_storage() -> Arc<Mutex<crate::storage::test_mock::MockStorage>> {
        let storage = crate::storage::test_mock::MockStorage::new()
            .expect("Failed to create test storage");
        Arc::new(Mutex::new(storage))
    }

    #[tokio::test]
    async fn test_union_basic() {
        let storage = create_test_storage();
        let mut executor = UnionExecutor::new(
            1,
            storage,
            "left_input".to_string(),
            "right_input".to_string(),
        );

        // 设置测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 重复行
                vec![Value::Int(3), Value::String("Charlie".to_string())],
            ],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "left_input".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "right_input".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行UNION操作
        let result = executor.execute();

        // 验证结果
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该有3个唯一的值（去重后）
            // 1, Alice, 2, Bob, 3, Charlie
            assert_eq!(values.len(), 6); // 3行 × 2列
        } else {
            panic!("期望Values结果");
        }
    }

    #[tokio::test]
    async fn test_union_empty_datasets() {
        let storage = create_test_storage();
        let mut executor = UnionExecutor::new(
            2,
            storage,
            "empty_left".to_string(),
            "empty_right".to_string(),
        );

        // 设置两个空数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "empty_left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "empty_right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 测试空数据集的UNION
        let result = executor.execute();
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            assert_eq!(values.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_union_mismatched_columns() {
        let storage = create_test_storage();
        let mut executor = UnionExecutor::new(
            3,
            storage,
            "left_mismatch".to_string(),
            "right_mismatch".to_string(),
        );

        // 设置列名不匹配的数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![Value::Int(1), Value::String("Alice".to_string())]],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "title".to_string()], // 不同的列名
            rows: vec![vec![Value::Int(2), Value::String("Mr".to_string())]],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "left_mismatch".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "right_mismatch".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行应该失败
        let result = executor.execute();
        assert!(result.is_err());

        if let Err(crate::core::error::DBError::Query(
            crate::core::error::QueryError::ExecutionError(msg),
        )) = result
        {
            assert!(msg.contains("列名不匹配"));
        } else {
            panic!("期望列名不匹配错误");
        }
    }
}
