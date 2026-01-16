//! 集合操作执行器基类
//!
//! 提供所有集合操作执行器的通用功能和接口

use async_trait::async_trait;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Value};
use crate::query::executor::{BaseExecutor, ExecutionResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 集合操作执行器基类
///
/// 提供所有集合操作（Union、Intersect、Minus等）的通用功能
#[derive(Debug)]
pub struct SetExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    col_names: Vec<String>,
}

impl<S: StorageEngine> SetExecutor<S> {
    /// 创建新的集合操作执行器
    pub fn new(
        id: i64,
        name: String,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, name, storage),
            left_input_var,
            right_input_var,
            col_names: Vec::new(),
        }
    }

    /// 获取对内部base执行器的可变引用
    pub fn base_mut(&mut self) -> &mut BaseExecutor<S> {
        &mut self.base
    }

    /// 获取对内部base执行器的不可变引用
    pub fn base(&self) -> &BaseExecutor<S> {
        &self.base
    }

    /// 获取左输入数据集
    pub fn get_left_input_data(&self) -> Result<DataSet, QueryError> {
        match self.base.context.get_result(&self.left_input_var) {
            Some(ExecutionResult::Values(values)) => {
                // 检查Values中是否包含DataSet
                if values.len() == 1 {
                    if let Value::DataSet(dataset) = &values[0] {
                        return Ok(dataset.clone());
                    }
                }
                // 如果不是DataSet，尝试将Values转换为DataSet
                Ok(DataSet {
                    col_names: self.col_names.clone(),
                    rows: vec![values.clone()],
                })
            }
            Some(_result) => {
                // 其他类型的结果需要转换为DataSet
                Err(QueryError::ExecutionError(format!(
                    "左输入变量 {} 不是有效的数据集",
                    self.left_input_var
                )))
            }
            None => Err(QueryError::ExecutionError(format!(
                "左输入变量 {} 不存在",
                self.left_input_var
            ))),
        }
    }

    /// 获取右输入数据集
    pub fn get_right_input_data(&self) -> Result<DataSet, QueryError> {
        match self.base.context.get_result(&self.right_input_var) {
            Some(ExecutionResult::Values(values)) => {
                // 检查Values中是否包含DataSet
                if values.len() == 1 {
                    if let Value::DataSet(dataset) = &values[0] {
                        return Ok(dataset.clone());
                    }
                }
                // 如果不是DataSet，尝试将Values转换为DataSet
                Ok(DataSet {
                    col_names: self.col_names.clone(),
                    rows: vec![values.clone()],
                })
            }
            Some(_result) => {
                // 其他类型的结果需要转换为DataSet
                Err(QueryError::ExecutionError(format!(
                    "右输入变量 {} 不是有效的数据集",
                    self.right_input_var
                )))
            }
            None => Err(QueryError::ExecutionError(format!(
                "右输入变量 {} 不存在",
                self.right_input_var
            ))),
        }
    }

    /// 检查输入数据集的有效性
    ///
    /// 验证两个输入数据集的列名是否一致
    pub fn check_input_data_sets(
        &mut self,
        left: &DataSet,
        right: &DataSet,
    ) -> Result<(), QueryError> {
        if left.col_names != right.col_names {
            let left_cols = left.col_names.join(",");
            let right_cols = right.col_names.join(",");
            return Err(QueryError::ExecutionError(format!(
                "数据集列名不匹配: <{}> vs <{}>",
                left_cols, right_cols
            )));
        }

        // 保存列名供后续使用
        self.col_names = left.col_names.clone();
        Ok(())
    }

    /// 获取列名
    pub fn get_col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    /// 设置列名
    pub fn set_col_names(&mut self, col_names: Vec<String>) {
        self.col_names = col_names;
    }

    /// 创建行的哈希值用于去重和比较
    pub fn hash_row(row: &[Value]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        for value in row {
            value.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// 创建行集合用于快速查找
    pub fn create_row_set(rows: &[Vec<Value>]) -> HashSet<u64> {
        let mut row_set = HashSet::new();
        for row in rows {
            row_set.insert(Self::hash_row(row));
        }
        row_set
    }

    /// 检查行是否在集合中
    pub fn row_in_set(row: &[Value], row_set: &HashSet<u64>) -> bool {
        let hash = Self::hash_row(row);
        row_set.contains(&hash)
    }

    /// 去重数据集的行
    pub fn dedup_rows(rows: Vec<Vec<Value>>) -> Vec<Vec<Value>> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for row in rows {
            let hash = Self::hash_row(&row);
            if seen.insert(hash) {
                result.push(row);
            }
        }

        result
    }

    /// 合并两个数据集的行（不去重）
    pub fn concat_datasets(left: DataSet, right: DataSet) -> DataSet {
        let mut rows = left.rows;
        rows.extend(right.rows);

        DataSet {
            col_names: left.col_names,
            rows,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S>
    for SetExecutor<S>
{
    async fn execute(
        &mut self,
    ) -> crate::query::executor::traits::DBResult<crate::query::executor::traits::ExecutionResult>
    {
        Err(crate::core::error::DBError::Query(
            crate::core::error::QueryError::ExecutionError(
                "SetExecutor是抽象基类，不能直接执行".to_string(),
            ),
        ))
    }

    fn open(&mut self) -> crate::query::executor::traits::DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> crate::query::executor::traits::DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Set executor base class"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_hash_row() {
        let row1 = vec![Value::Int(1), Value::String("test".to_string())];
        let row2 = vec![Value::Int(1), Value::String("test".to_string())];
        let row3 = vec![Value::Int(2), Value::String("test".to_string())];

        assert_eq!(
            SetExecutor::<crate::storage::MemoryStorage>::hash_row(&row1),
            SetExecutor::<crate::storage::MemoryStorage>::hash_row(&row2)
        );
        assert_ne!(
            SetExecutor::<crate::storage::MemoryStorage>::hash_row(&row1),
            SetExecutor::<crate::storage::MemoryStorage>::hash_row(&row3)
        );
    }

    #[test]
    fn test_create_row_set() {
        let rows = vec![
            vec![Value::Int(1), Value::String("a".to_string())],
            vec![Value::Int(2), Value::String("b".to_string())],
            vec![Value::Int(1), Value::String("a".to_string())], // 重复行
        ];

        let row_set = SetExecutor::<crate::storage::MemoryStorage>::create_row_set(&rows);
        assert_eq!(row_set.len(), 2); // 应该只有2个唯一的哈希值
    }

    #[test]
    fn test_dedup_rows() {
        let rows = vec![
            vec![Value::Int(1), Value::String("a".to_string())],
            vec![Value::Int(2), Value::String("b".to_string())],
            vec![Value::Int(1), Value::String("a".to_string())], // 重复行
            vec![Value::Int(3), Value::String("c".to_string())],
        ];

        let deduped = SetExecutor::<crate::storage::MemoryStorage>::dedup_rows(rows);
        assert_eq!(deduped.len(), 3); // 应该去重为3行
    }

    #[test]
    fn test_concat_datasets() {
        let left = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![Value::Int(1), Value::String("Alice".to_string())]],
        };

        let right = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![Value::Int(2), Value::String("Bob".to_string())]],
        };

        let result = SetExecutor::<crate::storage::MemoryStorage>::concat_datasets(left, right);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.col_names.len(), 2);
    }
}
