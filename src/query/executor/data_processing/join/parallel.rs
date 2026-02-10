//! Join操作的并行处理实现
//!
//! 提供并行化的join算法，利用多核CPU提升大表连接的性能

use rayon::prelude::*;
use std::collections::HashMap;

use crate::core::{Value, DataSet};
use crate::query::executor::data_processing::join::hash_table::{JoinKey, SingleKeyHashTable, MultiKeyHashTable};

/// 并行处理的配置
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub parallelism: usize,
    pub min_rows_per_thread: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            parallelism: rayon::current_num_threads(),
            min_rows_per_thread: 1000,
        }
    }
}

impl ParallelConfig {
    pub fn should_use_parallel(&self, data_size: usize) -> bool {
        data_size >= self.min_rows_per_thread && self.parallelism > 1
    }
}

/// 并行哈希表构建器
pub struct ParallelHashTableBuilder;

impl ParallelHashTableBuilder {
    /// 并行构建单键哈希表
    pub fn build_single_key_table_parallel(
        dataset: &DataSet,
        key_index: usize,
        config: &ParallelConfig,
    ) -> Result<SingleKeyHashTable, String> {
        if !config.should_use_parallel(dataset.rows.len()) {
            return crate::query::executor::data_processing::join::hash_table::HashTableBuilder::build_single_key_table(dataset, key_index);
        }

        let rows: Vec<Vec<Value>> = dataset.rows.clone();
        
        let local_tables: Vec<HashMap<Value, Vec<Vec<Value>>>> = rows
            .par_chunks(config.parallelism)
            .map(|chunk| {
                let mut local_hash_table = HashMap::new();
                for row in chunk {
                    if key_index < row.len() {
                        let key = row[key_index].clone();
                        local_hash_table.entry(key).or_insert_with(Vec::new).push(row.clone());
                    }
                }
                local_hash_table
            })
            .collect();

        let mut global_hash_table = HashMap::new();
        for local_table in local_tables {
            for (key, rows) in local_table {
                global_hash_table.entry(key).or_insert_with(Vec::new).extend(rows);
            }
        }

        Ok(global_hash_table)
    }

    /// 并行构建多键哈希表
    pub fn build_multi_key_table_parallel(
        dataset: &DataSet,
        key_indices: &[usize],
        config: &ParallelConfig,
    ) -> Result<MultiKeyHashTable, String> {
        if !config.should_use_parallel(dataset.rows.len()) {
            return crate::query::executor::data_processing::join::hash_table::HashTableBuilder::build_multi_key_table(dataset, key_indices);
        }

        let rows: Vec<Vec<Value>> = dataset.rows.clone();
        let key_indices = key_indices.to_vec();
        
        let local_tables: Vec<HashMap<JoinKey, Vec<Vec<Value>>>> = rows
            .par_chunks(config.parallelism)
            .map(|chunk| {
                let mut local_hash_table = HashMap::new();
                for row in chunk {
                    let mut key_values = Vec::new();
                    let mut valid_row = true;
                    
                    for &key_idx in &key_indices {
                        if key_idx < row.len() {
                            key_values.push(row[key_idx].clone());
                        } else {
                            valid_row = false;
                            break;
                        }
                    }
                    
                    if valid_row {
                        let join_key = JoinKey::new(key_values);
                        local_hash_table.entry(join_key).or_insert_with(Vec::new).push(row.clone());
                    }
                }
                local_hash_table
            })
            .collect();

        let mut global_hash_table = HashMap::new();
        for local_table in local_tables {
            for (key, rows) in local_table {
                global_hash_table.entry(key).or_insert_with(Vec::new).extend(rows);
            }
        }

        Ok(global_hash_table)
    }
}

/// 并行哈希表探测器
pub struct ParallelHashTableProbe;

impl ParallelHashTableProbe {
    /// 并行单键探测
    pub fn probe_single_key_parallel(
        hash_table: &SingleKeyHashTable,
        probe_dataset: &DataSet,
        key_index: usize,
        config: &ParallelConfig,
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        if !config.should_use_parallel(probe_dataset.rows.len()) {
            return crate::query::executor::data_processing::join::hash_table::HashTableProbe::probe_single_key(hash_table, probe_dataset, key_index);
        }

        let hash_table_ref = hash_table;
        let probe_rows: Vec<Vec<Value>> = probe_dataset.rows.clone();
        
        let local_results: Vec<Vec<(Vec<Value>, Vec<Vec<Value>>)>> = probe_rows
            .par_chunks(config.parallelism)
            .map(|chunk| {
                let mut results = Vec::new();
                for probe_row in chunk {
                    if key_index < probe_row.len() {
                        let key = probe_row[key_index].clone();
                        if let Some(matching_rows) = hash_table_ref.get(&key) {
                            results.push((probe_row.clone(), matching_rows.clone()));
                        }
                    }
                }
                results
            })
            .collect();

        let mut global_results = Vec::new();
        for local in local_results {
            global_results.extend(local);
        }

        global_results
    }

    /// 并行多键探测
    pub fn probe_multi_key_parallel(
        hash_table: &MultiKeyHashTable,
        probe_dataset: &DataSet,
        key_indices: &[usize],
        config: &ParallelConfig,
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        if !config.should_use_parallel(probe_dataset.rows.len()) {
            return crate::query::executor::data_processing::join::hash_table::HashTableProbe::probe_multi_key(hash_table, probe_dataset, key_indices);
        }

        let hash_table_ref = hash_table;
        let probe_rows: Vec<Vec<Value>> = probe_dataset.rows.clone();
        let key_indices = key_indices.to_vec();
        
        let local_results: Vec<Vec<(Vec<Value>, Vec<Vec<Value>>)>> = probe_rows
            .par_chunks(config.parallelism)
            .map(|chunk| {
                let mut results = Vec::new();
                for probe_row in chunk {
                    let mut key_values = Vec::new();
                    let mut valid_row = true;
                    
                    for &key_idx in &key_indices {
                        if key_idx < probe_row.len() {
                            key_values.push(probe_row[key_idx].clone());
                        } else {
                            valid_row = false;
                            break;
                        }
                    }
                    
                    if valid_row {
                        let join_key = JoinKey::new(key_values);
                        if let Some(matching_rows) = hash_table_ref.get(&join_key) {
                            results.push((probe_row.clone(), matching_rows.clone()));
                        }
                    }
                }
                results
            })
            .collect();

        let mut global_results = Vec::new();
        for local in local_results {
            global_results.extend(local);
        }

        global_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.parallelism > 0);
        assert_eq!(config.min_rows_per_thread, 1000);
    }

    #[test]
    fn test_parallel_hash_table_builder() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["id".to_string(), "name".to_string()];

        // 创建足够多的数据以触发并行处理
        for i in 0..2000 {
            dataset.rows.push(vec![
                Value::Int(i % 100), // 100个不同的键
                Value::String(format!("user_{}", i)),
            ]);
        }

        let config = ParallelConfig {
            parallelism: 2,
            min_rows_per_thread: 1000,
        };

        let result = ParallelHashTableBuilder::build_single_key_table_parallel(&dataset, 0, &config);
        assert!(result.is_ok());

        let hash_table = result.expect("Failed to build parallel hash table");
        assert_eq!(hash_table.len(), 100); // 100个不同的键

        // 验证每个键有20个值
        for bucket in hash_table.values() {
            assert_eq!(bucket.len(), 20);
        }
    }

    #[test]
    fn test_parallel_cartesian_product() {
        let mut left_dataset = DataSet::new();
        left_dataset.col_names = vec!["id".to_string()];
        
        let mut right_dataset = DataSet::new();
        right_dataset.col_names = vec!["value".to_string()];
        
        // 创建足够多的数据以触发并行处理
        for i in 0..100 {
            left_dataset.rows.push(vec![Value::Int(i)]);
        }
        
        for i in 0..10 {
            right_dataset.rows.push(vec![Value::Int(i * 10)]);
        }

        let config = ParallelConfig {
            parallelism: 2,
            min_rows_per_thread: 50,
            enable_work_stealing: false,
        };

        let result = ParallelCartesianProduct::execute_parallel(&left_dataset, &right_dataset, &config);
        assert_eq!(result.rows.len(), 1000); // 100 * 10 = 1000
    }
}