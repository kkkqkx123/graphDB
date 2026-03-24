//! 哈希表实现，用于join操作
//!
//! 提供高效的哈希表用于join操作

use crate::core::types::expr::Expression;
use crate::core::{DBError, DBResult, DataSet, Value};
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::expression::DefaultExpressionContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Join键，支持高效的哈希和序列化
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinKey {
    values: Vec<Value>,
    /// 预计算的哈希值，避免重复计算
    cached_hash: u64,
}

impl JoinKey {
    pub fn new(values: Vec<Value>) -> Self {
        let cached_hash = Self::calculate_hash(&values);
        Self {
            values,
            cached_hash,
        }
    }

    fn calculate_hash(values: &[Value]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        for value in values {
            value.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

impl Hash for JoinKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.cached_hash);
    }
}

/// 哈希表条目，包含行数据和元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashTableEntry {
    /// 行数据
    pub row: Vec<Value>,
    /// 原始行索引（用于调试和重复数据处理）
    pub original_index: usize,
}

impl HashTableEntry {
    pub fn new(row: Vec<Value>, original_index: usize) -> Self {
        Self {
            row,
            original_index,
        }
    }
}

/// 哈希表
pub struct HashTable {
    /// 哈希表
    table: HashMap<JoinKey, Vec<HashTableEntry>>,
}

impl HashTable {
    /// 创建新的哈希表
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            table: HashMap::with_capacity(initial_capacity),
        }
    }

    /// 插入条目
    pub fn insert(&mut self, key: JoinKey, entry: HashTableEntry) -> DBResult<()> {
        self.table.entry(key).or_default().push(entry);
        Ok(())
    }

    /// 探测哈希表
    pub fn probe(&self, key: &JoinKey) -> Vec<HashTableEntry> {
        self.table
            .get(key)
            .map_or_else(Vec::new, |entries| entries.clone())
    }

    /// 获取条目数量
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// 清空哈希表
    pub fn clear(&mut self) {
        self.table.clear();
    }
}

/// 哈希表构建器
pub struct HashTableBuilder;

impl HashTableBuilder {
    /// 从数据集构建哈希表
    pub fn build_from_dataset(
        dataset: &DataSet,
        key_indices: &[usize],
        initial_capacity: usize,
    ) -> DBResult<HashTable> {
        let mut hash_table = HashTable::new(initial_capacity);

        for (idx, row) in dataset.rows.iter().enumerate() {
            let mut key_values = Vec::new();
            for &key_index in key_indices {
                if key_index < row.len() {
                    key_values.push(row[key_index].clone());
                } else {
                    return Err(DBError::Validation(format!(
                        "Key index {} out of bounds for row with {} columns",
                        key_index,
                        row.len()
                    )));
                }
            }

            let key = JoinKey::new(key_values);
            let entry = HashTableEntry::new(row.clone(), idx);

            hash_table.insert(key, entry)?;
        }

        Ok(hash_table)
    }
}

/// 构建哈希表函数（接受表达式）
pub fn build_hash_table(
    dataset: &DataSet,
    key_exprs: &[Expression],
) -> Result<HashMap<JoinKey, Vec<usize>>, String> {
    let mut hash_table = HashMap::new();

    for (idx, row) in dataset.rows.iter().enumerate() {
        let mut expr_context = DefaultExpressionContext::new();
        for (i, col_name) in dataset.col_names.iter().enumerate() {
            if i < row.len() {
                expr_context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let mut key_values = Vec::new();
        for key_expression in key_exprs {
            match ExpressionEvaluator::evaluate(key_expression, &mut expr_context) {
                Ok(value) => key_values.push(value),
                Err(e) => return Err(format!("键表达式求值失败: {}", e)),
            }
        }

        let key = JoinKey::new(key_values);
        hash_table.entry(key).or_insert_with(Vec::new).push(idx);
    }

    Ok(hash_table)
}

/// 提取键值
pub fn extract_key_values(
    row: &[Value],
    _col_names: &[String],
    key_exprs: &[Expression],
    col_map: &std::collections::HashMap<&str, usize>,
) -> Vec<Value> {
    let mut key_values = Vec::new();
    for key_expression in key_exprs {
        let mut expr_context = DefaultExpressionContext::new();
        for (col_name, &col_idx) in col_map.iter() {
            if col_idx < row.len() {
                expr_context.set_variable(col_name.to_string(), row[col_idx].clone());
            }
        }
        if let Ok(value) = ExpressionEvaluator::evaluate(key_expression, &mut expr_context) {
            key_values.push(value);
        }
    }
    key_values
}

impl std::fmt::Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashTable")
            .field("len", &self.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_key() {
        let key1 = JoinKey::new(vec![Value::Int(1), Value::String("test".to_string())]);
        let key2 = JoinKey::new(vec![Value::Int(1), Value::String("test".to_string())]);

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 2);
        assert!(!key1.is_empty());
    }

    #[test]
    fn test_hash_table_entry() {
        let entry = HashTableEntry::new(vec![Value::Int(1), Value::String("test".to_string())], 0);

        assert_eq!(entry.original_index, 0);
        assert_eq!(entry.row.len(), 2);
    }

    #[test]
    fn test_hash_table_basic() {
        let mut hash_table = HashTable::new(100);

        let key = JoinKey::new(vec![Value::Int(1)]);
        let entry = HashTableEntry::new(vec![Value::String("test".to_string())], 0);

        hash_table
            .insert(key.clone(), entry)
            .expect("insert should succeed");

        let results = hash_table.probe(&key);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].row[0], Value::String("test".to_string()));

        assert_eq!(hash_table.len(), 1);
        assert!(!hash_table.is_empty());

        hash_table.clear();
        assert_eq!(hash_table.len(), 0);
        assert!(hash_table.is_empty());
    }

    #[test]
    fn test_build_hash_table() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["id".to_string(), "name".to_string()];
        dataset
            .rows
            .push(vec![Value::Int(1), Value::String("Alice".to_string())]);
        dataset
            .rows
            .push(vec![Value::Int(2), Value::String("Bob".to_string())]);

        let hash_table =
            HashTableBuilder::build_from_dataset(&dataset, &[0], 10).expect("build should succeed");

        assert_eq!(hash_table.len(), 2);

        let key = JoinKey::new(vec![Value::Int(1)]);
        let results = hash_table.probe(&key);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_build_hash_table_with_expr() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["id".to_string(), "name".to_string()];
        dataset
            .rows
            .push(vec![Value::Int(1), Value::String("Alice".to_string())]);
        dataset
            .rows
            .push(vec![Value::Int(2), Value::String("Bob".to_string())]);

        let id_expr = Expression::Variable("id".to_string());
        let hash_table = build_hash_table(&dataset, &[id_expr]).expect("build should succeed");

        assert_eq!(hash_table.len(), 2);

        let key = JoinKey::new(vec![Value::Int(1)]);
        let indices = hash_table.get(&key);
        assert!(indices.is_some());
        assert_eq!(indices.expect("索引应存在"), &vec![0]);
    }
}
