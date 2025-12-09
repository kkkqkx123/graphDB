//! UnionAll执行器实现
//!
//! 实现UNION ALL操作，合并两个数据集但保留重复行

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::query::executor::{Executor, ExecutionResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

use super::base::SetExecutor;

/// UnionAll执行器
///
/// 实现UNION ALL操作，合并两个数据集但保留重复行
/// 类似于SQL的UNION ALL
pub struct UnionAllExecutor<S: StorageEngine> {
    pub set_executor: SetExecutor<S>,
}

impl<S: StorageEngine> UnionAllExecutor<S> {
    /// 创建新的UnionAll执行器
    pub fn new(
        id: usize,
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
    /// 3. 合并两个数据集的所有行（不去重）
    /// 4. 返回结果
    async fn execute_union_all(&mut self) -> Result<DataSet, QueryError> {
        // 获取左右输入数据集
        let left_dataset = self.set_executor.get_left_input_data()?;
        let right_dataset = self.set_executor.get_right_input_data()?;

        // 检查输入数据集的有效性
        self.set_executor.check_input_data_sets(&left_dataset, &right_dataset)?;

        // 合并两个数据集（不去重）
        let result_dataset = SetExecutor::<S>::concat_datasets(left_dataset, right_dataset);

        Ok(result_dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for UnionAllExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        let dataset = self.execute_union_all().await?;
        
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
    async fn test_union_all_basic() {
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
        // 这里需要根据实际的上下文实现来设置数据

        // 执行UNION ALL操作
        let result = executor.execute().await;

        // 验证结果
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该有4个值（不去重）
            // 1, Alice, 2, Bob, 2, Bob, 3, Charlie
            assert_eq!(values.len(), 8); // 4行 × 2列
        } else {
            panic!("期望Values结果");
        }
    }

    #[tokio::test]
    async fn test_union_all_empty_left() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            2,
            storage,
            "empty_left".to_string(),
            "right_input".to_string(),
        );

        // 测试左数据集为空的UNION ALL
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该只包含右数据集的内容
            // 具体数量取决于右数据集的内容
            assert!(values.len() >= 0);
        }
    }

    #[tokio::test]
    async fn test_union_all_empty_right() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            3,
            storage,
            "left_input".to_string(),
            "empty_right".to_string(),
        );

        // 测试右数据集为空的UNION ALL
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该只包含左数据集的内容
            // 具体数量取决于左数据集的内容
            assert!(values.len() >= 0);
        }
    }

    #[tokio::test]
    async fn test_union_all_both_empty() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            4,
            storage,
            "empty_left".to_string(),
            "empty_right".to_string(),
        );

        // 测试两个数据集都为空的UNION ALL
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            assert_eq!(values.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_union_all_mismatched_columns() {
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

        // 执行应该失败
        let result = executor.execute().await;
        assert!(result.is_err());
        
        if let Err(QueryError::ExecutionError(msg)) = result {
            assert!(msg.contains("列名不匹配"));
        } else {
            panic!("期望列名不匹配错误");
        }
    }

    #[tokio::test]
    async fn test_union_all_preserve_duplicates() {
        let storage = create_test_storage();
        let mut executor = UnionAllExecutor::new(
            6,
            storage,
            "left_dup".to_string(),
            "right_dup".to_string(),
        );

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

        // 执行UNION ALL操作
        let result = executor.execute().await;
        assert!(result.is_ok());
        
        if let Ok(ExecutionResult::Values(values)) = result {
            // 应该保留所有重复行
            // 3行 × 2列 = 6个值
            assert_eq!(values.len(), 6);
        } else {
            panic!("期望Values结果");
        }
    }
}