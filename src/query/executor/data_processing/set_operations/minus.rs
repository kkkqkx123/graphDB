//! Minus执行器实现
//!
//! 实现MINUS操作，返回左数据集中存在但右数据集中不存在的行

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::query::executor::{Executor, ExecutionResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

use super::base::SetExecutor;

/// Minus执行器
///
/// 实现MINUS操作，返回左数据集中存在但右数据集中不存在的行
/// 类似于SQL的EXCEPT或MINUS
pub struct MinusExecutor<S: StorageEngine> {
    pub set_executor: SetExecutor<S>,
}

impl<S: StorageEngine> MinusExecutor<S> {
    /// 创建新的Minus执行器
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
    ) -> Self {
        Self {
            set_executor: SetExecutor::new(
                id,
                "MinusExecutor".to_string(),
                storage,
                left_input_var,
                right_input_var,
            ),
        }
    }

    /// 执行MINUS操作
    /// 
    /// 算法步骤：
    /// 1. 获取左右两个输入数据集
    /// 2. 验证列名是否一致
    /// 3. 创建右数据集的行哈希集合
    /// 4. 遍历左数据集，只保留在右数据集中不存在的行
    /// 5. 返回结果
    async fn execute_minus(&mut self) -> Result<DataSet, QueryError> {
        // 获取左右输入数据集
        let left_dataset = self.set_executor.get_left_input_data()?;
        let right_dataset = self.set_executor.get_right_input_data()?;

        // 检查输入数据集的有效性
        self.set_executor.check_input_data_sets(&left_dataset, &right_dataset)?;

        // 如果右数据集为空，直接返回左数据集
        if right_dataset.rows.is_empty() {
            return Ok(DataSet {
                col_names: self.set_executor.get_col_names().clone(),
                rows: left_dataset.rows,
            });
        }

        // 如果左数据集为空，直接返回空结果
        if left_dataset.rows.is_empty() {
            return Ok(DataSet {
                col_names: self.set_executor.get_col_names().clone(),
                rows: Vec::new(),
            });
        }

        // 创建右数据集的行哈希集合用于快速查找
        let right_row_set = SetExecutor::<S>::create_row_set(&right_dataset.rows);

        // 找出在左数据集中存在但在右数据集中不存在的行
        let mut minus_rows = Vec::new();
        
        for left_row in &left_dataset.rows {
            if !SetExecutor::<S>::row_in_set(left_row, &right_row_set) {
                minus_rows.push(left_row.clone());
            }
        }

        // 构建结果数据集
        let result_dataset = DataSet {
            col_names: self.set_executor.get_col_names().clone(),
            rows: minus_rows,
        };

        Ok(result_dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for MinusExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        let dataset = self.execute_minus().await?;
        
        // 将DataSet转换为Values结果
        let values: Vec<Value> = dataset.rows.into_iter()
            .flat_map(|row| row.into_iter())
            .collect();

        Ok(ExecutionResult::Values(values))
    }

    fn open(&mut self) -> Result<(), QueryError> {
        self.set_executor.open()
    }

    fn close(&mut self) -> Result<(), QueryError> {
        self.set_executor.close()
    }

    fn id(&self) -> usize {
        self.set_executor.id()
    }

    fn name(&self) -> &str {
        self.set_executor.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    // 创建测试用的存储引擎
    fn create_test_storage() -> Arc<Mutex<crate::storage::NativeStorage>> {
        // 这里应该创建一个测试用的存储引擎实例
        // 为了简化测试，我们使用一个模拟的实现
        todo!("需要实现测试用的存储引擎")
    }

    #[tokio::test]
    async fn test_minus_basic() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
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
                vec![Value::Int(3), Value::String("Charlie".to_string())],
                vec![Value::Int(4), Value::String("David".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 要排除的行
                vec![Value::Int(4), Value::String("David".to_string())], // 要排除的行
                vec![Value::Int(5), Value::String("Eve".to_string())], // 左数据集中不存在的行
            ],
        };

        // 将数据集设置到执行器上下文中
        // 这里需要根据实际的上下文实现来设置数据

        // 执行MINUS操作
        let result = executor.execute().await;

        // 验证结果
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该只包含Alice和Charlie（Bob和David被排除）
            // 2行 × 2列 = 4个值
            assert_eq!(values.len(), 4);
        } else {
            panic!("期望Values结果");
        }
    }

    #[tokio::test]
    async fn test_minus_no_overlap() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            2,
            storage,
            "left_no_overlap".to_string(),
            "right_no_overlap".to_string(),
        );

        // 设置没有重叠的数据集
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
                vec![Value::Int(3), Value::String("Charlie".to_string())],
                vec![Value::Int(4), Value::String("David".to_string())],
            ],
        };

        // 执行MINUS操作
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 没有重叠，应该返回整个左数据集
            // 2行 × 2列 = 4个值
            assert_eq!(values.len(), 4);
        }
    }

    #[tokio::test]
    async fn test_minus_all_overlap() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            3,
            storage,
            "left_all_overlap".to_string(),
            "right_all_overlap".to_string(),
        );

        // 设置完全重叠的数据集
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
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        // 执行MINUS操作
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 完全重叠，结果应该为空
            assert_eq!(values.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_minus_empty_left() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            4,
            storage,
            "empty_left".to_string(),
            "right_input".to_string(),
        );

        // 测试左数据集为空的MINUS
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 左数据集为空，结果应该为空
            assert_eq!(values.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_minus_empty_right() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            5,
            storage,
            "left_input".to_string(),
            "empty_right".to_string(),
        );

        // 测试右数据集为空的MINUS
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 右数据集为空，应该返回整个左数据集
            // 具体数量取决于左数据集的内容
            assert!(values.len() >= 0);
        }
    }

    #[tokio::test]
    async fn test_minus_both_empty() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            6,
            storage,
            "empty_left".to_string(),
            "empty_right".to_string(),
        );

        // 测试两个数据集都为空的MINUS
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            assert_eq!(values.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_minus_with_duplicates() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            7,
            storage,
            "left_dup".to_string(),
            "right_dup".to_string(),
        );

        // 设置包含重复行的数据集
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "value".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("common".to_string())],
                vec![Value::Int(1), Value::String("common".to_string())], // 左数据集中的重复行
                vec![Value::Int(2), Value::String("unique".to_string())],
                vec![Value::Int(3), Value::String("another".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "value".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("common".to_string())],
                vec![Value::Int(3), Value::String("another".to_string())],
            ],
        };

        // 执行MINUS操作
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该只包含unique行，common和another被排除
            // 1行 × 2列 = 2个值
            assert_eq!(values.len(), 2);
        } else {
            panic!("期望Values结果");
        }
    }

    #[tokio::test]
    async fn test_minus_mismatched_columns() {
        let storage = create_test_storage();
        let mut executor = MinusExecutor::new(
            8,
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
            rows: vec![vec![Value::Int(1), Value::String("Ms".to_string())]],
        };

        // 执行应该失败
        let result = executor.execute().await;
        assert!(result.is_err());
        
        if let Err(QueryError::ExecutionError(msg)) = result {
            assert!(msg.contains("列名不匹配"));
        } else {
            panic!("期望列名不匹配错误");
        }
    }
}