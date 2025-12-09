//! Join操作的哈希表实现
//!
//! 提供高效的哈希表用于join操作，支持单键和多键连接

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::core::Value;

/// Join键的表示，支持多键连接
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinKey {
    values: Vec<Value>,
}

impl JoinKey {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
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
        for value in &self.values {
            value.hash(state);
        }
    }
}

/// 单键哈希表
pub type SingleKeyHashTable = HashMap<Value, Vec<Vec<Value>>>;

/// 多键哈希表
pub type MultiKeyHashTable = HashMap<JoinKey, Vec<Vec<Value>>>;

/// 哈希表构建器
pub struct HashTableBuilder;

impl HashTableBuilder {
    /// 构建单键哈希表
    pub fn build_single_key_table(
        dataset: &crate::core::DataSet,
        key_index: usize,
    ) -> Result<SingleKeyHashTable, String> {
        let mut hash_table = HashMap::new();
        
        for row in &dataset.rows {
            if key_index < row.len() {
                let key = row[key_index].clone();
                hash_table.entry(key).or_insert_with(Vec::new).push(row.clone());
            } else {
                return Err(format!("键索引 {} 超出行长度 {}", key_index, row.len()));
            }
        }
        
        Ok(hash_table)
    }

    /// 构建多键哈希表
    pub fn build_multi_key_table(
        dataset: &crate::core::DataSet,
        key_indices: &[usize],
    ) -> Result<MultiKeyHashTable, String> {
        let mut hash_table = HashMap::new();
        
        for row in &dataset.rows {
            let mut key_values = Vec::new();
            for &key_index in key_indices {
                if key_index < row.len() {
                    key_values.push(row[key_index].clone());
                } else {
                    return Err(format!("键索引 {} 超出行长度 {}", key_index, row.len()));
                }
            }
            
            let join_key = JoinKey::new(key_values);
            hash_table.entry(join_key).or_insert_with(Vec::new).push(row.clone());
        }
        
        Ok(hash_table)
    }
}

/// 哈希表探测器
pub struct HashTableProbe;

impl HashTableProbe {
    /// 单键探测
    pub fn probe_single_key(
        hash_table: &SingleKeyHashTable,
        probe_dataset: &crate::core::DataSet,
        key_index: usize,
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        let mut results = Vec::new();
        
        for probe_row in &probe_dataset.rows {
            if key_index < probe_row.len() {
                let key = probe_row[key_index].clone();
                if let Some(matching_rows) = hash_table.get(&key) {
                    results.push((probe_row.clone(), matching_rows.clone()));
                }
            }
        }
        
        results
    }

    /// 多键探测
    pub fn probe_multi_key(
        hash_table: &MultiKeyHashTable,
        probe_dataset: &crate::core::DataSet,
        key_indices: &[usize],
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        let mut results = Vec::new();
        
        for probe_row in &probe_dataset.rows {
            let mut key_values = Vec::new();
            for &key_index in key_indices {
                if key_index < probe_row.len() {
                    key_values.push(probe_row[key_index].clone());
                } else {
                    continue; // 跳过无效行
                }
            }
            
            let join_key = JoinKey::new(key_values);
            if let Some(matching_rows) = hash_table.get(&join_key) {
                results.push((probe_row.clone(), matching_rows.clone()));
            }
        }
        
        results
    }
}

/// 哈希表统计信息
#[derive(Debug, Clone)]
pub struct HashTableStats {
    pub total_rows: usize,
    pub unique_keys: usize,
    pub avg_bucket_size: f64,
    pub max_bucket_size: usize,
}

impl HashTableStats {
    /// 计算单键哈希表统计信息
    pub fn for_single_key_table(hash_table: &SingleKeyHashTable) -> Self {
        let total_rows: usize = hash_table.values().map(|bucket| bucket.len()).sum();
        let unique_keys = hash_table.len();
        let avg_bucket_size = if unique_keys > 0 {
            total_rows as f64 / unique_keys as f64
        } else {
            0.0
        };
        let max_bucket_size = hash_table.values().map(|bucket| bucket.len()).max().unwrap_or(0);
        
        Self {
            total_rows,
            unique_keys,
            avg_bucket_size,
            max_bucket_size,
        }
    }

    /// 计算多键哈希表统计信息
    pub fn for_multi_key_table(hash_table: &MultiKeyHashTable) -> Self {
        let total_rows: usize = hash_table.values().map(|bucket| bucket.len()).sum();
        let unique_keys = hash_table.len();
        let avg_bucket_size = if unique_keys > 0 {
            total_rows as f64 / unique_keys as f64
        } else {
            0.0
        };
        let max_bucket_size = hash_table.values().map(|bucket| bucket.len()).max().unwrap_or(0);
        
        Self {
            total_rows,
            unique_keys,
            avg_bucket_size,
            max_bucket_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;

    #[test]
    fn test_single_key_hash_table() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["id".to_string(), "name".to_string()];
        dataset.rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
            vec![Value::Int(1), Value::String("Charlie".to_string())],
        ];

        let hash_table = HashTableBuilder::build_single_key_table(&dataset, 0).unwrap();
        
        assert_eq!(hash_table.len(), 2); // 两个不同的键: 1 和 2
        assert_eq!(hash_table[&Value::Int(1)].len(), 2); // 键1有两行
        assert_eq!(hash_table[&Value::Int(2)].len(), 1); // 键2有一行
    }

    #[test]
    fn test_multi_key_hash_table() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["id".to_string(), "name".to_string(), "age".to_string()];
        dataset.rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string()), Value::Int(25)],
            vec![Value::Int(2), Value::String("Bob".to_string()), Value::Int(30)],
            vec![Value::Int(1), Value::String("Alice".to_string()), Value::Int(26)],
        ];

        let hash_table = HashTableBuilder::build_multi_key_table(&dataset, &[0, 1]).unwrap();
        
        assert_eq!(hash_table.len(), 2); // 两个不同的键组合: (1,Alice) 和 (2,Bob)
        
        let key1 = JoinKey::new(vec![Value::Int(1), Value::String("Alice".to_string())]);
        assert_eq!(hash_table[&key1].len(), 2); // 键(1,Alice)有两行
    }

    #[test]
    fn test_hash_table_probe() {
        let mut build_dataset = DataSet::new();
        build_dataset.col_names = vec!["id".to_string(), "name".to_string()];
        build_dataset.rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];

        let mut probe_dataset = DataSet::new();
        probe_dataset.col_names = vec!["id".to_string(), "age".to_string()];
        probe_dataset.rows = vec![
            vec![Value::Int(1), Value::Int(25)],
            vec![Value::Int(3), Value::Int(35)],
        ];

        let hash_table = HashTableBuilder::build_single_key_table(&build_dataset, 0).unwrap();
        let results = HashTableProbe::probe_single_key(&hash_table, &probe_dataset, 0);
        
        assert_eq!(results.len(), 1); // 只有一个匹配
        assert_eq!(results[0].0, vec![Value::Int(1), Value::Int(25)]); // 探测行
        assert_eq!(results[0].1.len(), 1); // 一个匹配的构建行
        assert_eq!(results[0].1[0], vec![Value::Int(1), Value::String("Alice".to_string())]);
    }
}