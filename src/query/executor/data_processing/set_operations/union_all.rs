//! UnionAll执行器实现
//!
//! 实现UNION ALL操作，合并两个数据集但保留重复行

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::{DataSet, Value};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageClient;

use super::base::SetExecutor;

/// UnionAll执行器
///
/// 实现UNION ALL操作，合并两个数据集但保留重复行
/// 类似于SQL的UNION ALL
#[derive(Debug)]
pub struct UnionAllExecutor<S: StorageClient> {
    pub set_executor: SetExecutor<S>,
}

impl<S: StorageClient> UnionAllExecutor<S> {
    /// 创建新的UnionAll执行器
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
    ) -> Self {
        Self {
            set_executor: SetExecutor::new(
                id,
                "UnionAllExecutor".to_string(),
                storage,
                left_input_var,
                right_input_var,
            ),
        }
    }

    /// 执行UNION ALL操作
    ///
    /// 算法步骤：
    /// 1. 获取左右两个输入数据集
    /// 2. 验证列名是否一致
    fn execute_union_all(&mut self) -> Result<DataSet, QueryError> {
        // 获取左右输入数据集
        let left_dataset = self.set_executor.get_left_input_data()?;
        let right_dataset = self.set_executor.get_right_input_data()?;

        // 检查输入数据集的有效性
        self.set_executor
            .check_input_data_sets(&left_dataset, &right_dataset)?;

        // 合并两个数据集（不去重）
        let result_dataset = SetExecutor::<S>::concat_datasets(left_dataset, right_dataset);

        Ok(result_dataset)
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for UnionAllExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_union_all().map_err(|e| {
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.set_executor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
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

    #[test]
    fn test_union_all_basic() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
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

        // 执行UNION ALL操作
        let result = executor.execute();

        // 验证结果
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该包含所有8个值（不去重）：左表4个值 + 右表4个值
            assert_eq!(values.len(), 8);
        } else {
            panic!("期望Values结果");
        }
    }

    #[test]
    fn test_union_all_empty_left() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            2,
            storage,
            "empty_left".to_string(),
            "right_input".to_string(),
        );

        // 设置空的左数据集和非空的右数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "empty_left".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "right_input".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 测试左数据集为空的UNION ALL
        let result = executor.execute();
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            assert_eq!(values.len(), 4);
        }
    }

    #[test]
    fn test_union_all_empty_right() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            3,
            storage,
            "left_input".to_string(),
            "empty_right".to_string(),
        );

        // 设置非空的左数据集和空的右数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "left_input".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "empty_right".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 测试右数据集为空的UNION ALL
        let result = executor.execute();
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该只包含左数据集的内容
            // 2行 × 2列 = 4个值
            assert_eq!(values.len(), 4);
        }
    }

    #[test]
    fn test_union_all_both_empty() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            4,
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

        // 测试两个数据集都为空的UNION ALL
        let result = executor.execute();
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            assert_eq!(values.len(), 0);
        }
    }

    #[test]
    fn test_union_all_mismatched_columns() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            5,
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

    #[test]
    fn test_union_all_preserve_duplicates() {
        let storage = create_test_storage();
        let mut executor =
            UnionAllExecutor::new(6, storage, "left_dup".to_string(), "right_dup".to_string());

        // 设置包含重复行的数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "value".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("same".to_string())],
                vec![Value::Int(1), Value::String("same".to_string())], // 左数据集中的重复行
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "value".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("same".to_string())], // 与左数据集重复的行
                vec![Value::Int(2), Value::String("different".to_string())],
            ],
        };

        // 将数据集设置到执行器上下文中
        executor.set_executor.base_mut().context.set_result(
            "left_dup".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(left_dataset)]),
        );
        executor.set_executor.base_mut().context.set_result(
            "right_dup".to_string(),
            ExecutionResult::Values(vec![Value::DataSet(right_dataset)]),
        );

        // 执行UNION ALL操作
        let result = executor.execute();
        assert!(result.is_ok());

        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该保留所有重复行
            // 左数据集有2行（其中1行重复），右数据集有2行，总共4行
            // 4行 × 2列 = 8个值
            assert_eq!(values.len(), 8);
        } else {
            panic!("期望Values结果");
        }
    }
}
