//! Join操作的并行处理实现
//!
//! 提供并行化的join算法，利用多核CPU提升大表连接的性能

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::core::{Value, DataSet};
use crate::query::executor::data_processing::join::hash_table::{JoinKey, SingleKeyHashTable, MultiKeyHashTable};

/// 并行处理的配置
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// 并行度（线程数）
    pub parallelism: usize,
    /// 每个线程处理的最小行数
    pub min_rows_per_thread: usize,
    /// 是否启用工作窃取
    pub enable_work_stealing: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        let num_cpus = num_cpus::get();
        Self {
            parallelism: num_cpus,
            min_rows_per_thread: 1000,
            enable_work_stealing: true,
        }
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
        if dataset.rows.len() < config.min_rows_per_thread || config.parallelism == 1 {
            // 数据量小，使用单线程
            return crate::query::executor::data_processing::join::hash_table::HashTableBuilder::build_single_key_table(dataset, key_index);
        }

        // 计算每个线程处理的行数
        let rows_per_thread = (dataset.rows.len() + config.parallelism - 1) / config.parallelism;
        let mut handles = Vec::new();

        // 分割数据并创建线程
        for i in 0..config.parallelism {
            let start = i * rows_per_thread;
            let end = std::cmp::min(start + rows_per_thread, dataset.rows.len());
            
            if start >= dataset.rows.len() {
                break;
            }

            let chunk = dataset.rows[start..end].to_vec();
            let key_idx = key_index;

            let handle = thread::spawn(move || {
                let mut local_hash_table = HashMap::new();
                
                for row in chunk {
                    if key_idx < row.len() {
                        let key = row[key_idx].clone();
                        local_hash_table.entry(key).or_insert_with(Vec::new).push(row);
                    }
                }
                
                local_hash_table
            });

            handles.push(handle);
        }

        // 合并结果
        let mut global_hash_table = HashMap::new();
        for handle in handles {
            let local_table = handle.join().map_err(|e| format!("线程执行失败: {:?}", e))?;
            
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
        if dataset.rows.len() < config.min_rows_per_thread || config.parallelism == 1 {
            // 数据量小，使用单线程
            return crate::query::executor::data_processing::join::hash_table::HashTableBuilder::build_multi_key_table(dataset, key_indices);
        }

        // 计算每个线程处理的行数
        let rows_per_thread = (dataset.rows.len() + config.parallelism - 1) / config.parallelism;
        let mut handles = Vec::new();

        // 分割数据并创建线程
        for i in 0..config.parallelism {
            let start = i * rows_per_thread;
            let end = std::cmp::min(start + rows_per_thread, dataset.rows.len());
            
            if start >= dataset.rows.len() {
                break;
            }

            let chunk = dataset.rows[start..end].to_vec();
            let key_idxs = key_indices.to_vec();

            let handle = thread::spawn(move || {
                let mut local_hash_table = HashMap::new();
                
                for row in chunk {
                    let mut key_values = Vec::new();
                    let mut valid_row = true;
                    
                    for &key_idx in &key_idxs {
                        if key_idx < row.len() {
                            key_values.push(row[key_idx].clone());
                        } else {
                            valid_row = false;
                            break;
                        }
                    }
                    
                    if valid_row {
                        let join_key = JoinKey::new(key_values);
                        local_hash_table.entry(join_key).or_insert_with(Vec::new).push(row);
                    }
                }
                
                local_hash_table
            });

            handles.push(handle);
        }

        // 合并结果
        let mut global_hash_table = HashMap::new();
        for handle in handles {
            let local_table = handle.join().map_err(|e| format!("线程执行失败: {:?}", e))?;
            
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
        if probe_dataset.rows.len() < config.min_rows_per_thread || config.parallelism == 1 {
            // 数据量小，使用单线程
            return crate::query::executor::data_processing::join::hash_table::HashTableProbe::probe_single_key(hash_table, probe_dataset, key_index);
        }

        // 创建共享的哈希表引用
        let hash_table_arc = Arc::new(hash_table);
        
        // 计算每个线程处理的行数
        let rows_per_thread = (probe_dataset.rows.len() + config.parallelism - 1) / config.parallelism;
        let mut handles = Vec::new();

        // 分割数据并创建线程
        for i in 0..config.parallelism {
            let start = i * rows_per_thread;
            let end = std::cmp::min(start + rows_per_thread, probe_dataset.rows.len());
            
            if start >= probe_dataset.rows.len() {
                break;
            }

            let chunk = probe_dataset.rows[start..end].to_vec();
            let key_idx = key_index;
            let hash_table_ref = Arc::clone(&hash_table_arc);

            let handle = thread::spawn(move || {
                let mut local_results = Vec::new();
                
                for probe_row in chunk {
                    if key_idx < probe_row.len() {
                        let key = probe_row[key_idx].clone();
                        if let Some(matching_rows) = hash_table_ref.get(&key) {
                            local_results.push((probe_row, matching_rows.clone()));
                        }
                    }
                }
                
                local_results
            });

            handles.push(handle);
        }

        // 合并结果
        let mut global_results = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(local_results) => global_results.extend(local_results),
                Err(e) => eprintln!("线程执行失败: {:?}", e),
            }
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
        if probe_dataset.rows.len() < config.min_rows_per_thread || config.parallelism == 1 {
            // 数据量小，使用单线程
            return crate::query::executor::data_processing::join::hash_table::HashTableProbe::probe_multi_key(hash_table, probe_dataset, key_indices);
        }

        // 创建共享的哈希表引用
        let hash_table_arc = Arc::new(hash_table);
        
        // 计算每个线程处理的行数
        let rows_per_thread = (probe_dataset.rows.len() + config.parallelism - 1) / config.parallelism;
        let mut handles = Vec::new();

        // 分割数据并创建线程
        for i in 0..config.parallelism {
            let start = i * rows_per_thread;
            let end = std::cmp::min(start + rows_per_thread, probe_dataset.rows.len());
            
            if start >= probe_dataset.rows.len() {
                break;
            }

            let chunk = probe_dataset.rows[start..end].to_vec();
            let key_idxs = key_indices.to_vec();
            let hash_table_ref = Arc::clone(&hash_table_arc);

            let handle = thread::spawn(move || {
                let mut local_results = Vec::new();
                
                for probe_row in chunk {
                    let mut key_values = Vec::new();
                    let mut valid_row = true;
                    
                    for &key_idx in &key_idxs {
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
                            local_results.push((probe_row, matching_rows.clone()));
                        }
                    }
                }
                
                local_results
            });

            handles.push(handle);
        }

        // 合并结果
        let mut global_results = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(local_results) => global_results.extend(local_results),
                Err(e) => eprintln!("线程执行失败: {:?}", e),
            }
        }

        global_results
    }
}

/// 并行笛卡尔积处理器
pub struct ParallelCartesianProduct;

impl ParallelCartesianProduct {
    /// 并行执行笛卡尔积
    pub fn execute_parallel(
        left_dataset: &DataSet,
        right_dataset: &DataSet,
        config: &ParallelConfig,
    ) -> DataSet {
        if left_dataset.rows.len() < config.min_rows_per_thread || config.parallelism == 1 {
            // 数据量小，使用单线程
            let mut result = DataSet::new();
            for left_row in &left_dataset.rows {
                for right_row in &right_dataset.rows {
                    let mut new_row = left_row.clone();
                    new_row.extend(right_row.clone());
                    result.rows.push(new_row);
                }
            }
            return result;
        }

        // 创建共享的右表引用
        let right_dataset_arc = Arc::new(right_dataset);
        
        // 计算每个线程处理的行数
        let rows_per_thread = (left_dataset.rows.len() + config.parallelism - 1) / config.parallelism;
        let mut handles = Vec::new();

        // 分割左表数据并创建线程
        for i in 0..config.parallelism {
            let start = i * rows_per_thread;
            let end = std::cmp::min(start + rows_per_thread, left_dataset.rows.len());
            
            if start >= left_dataset.rows.len() {
                break;
            }

            let chunk = left_dataset.rows[start..end].to_vec();
            let right_ref = Arc::clone(&right_dataset_arc);

            let handle = thread::spawn(move || {
                let mut local_results = Vec::new();
                
                for left_row in chunk {
                    for right_row in &right_ref.rows {
                        let mut new_row = left_row.clone();
                        new_row.extend(right_row.clone());
                        local_results.push(new_row);
                    }
                }
                
                local_results
            });

            handles.push(handle);
        }

        // 合并结果
        let mut result = DataSet::new();
        for handle in handles {
            match handle.join() {
                Ok(local_results) => result.rows.extend(local_results),
                Err(e) => eprintln!("线程执行失败: {:?}", e),
            }
        }

        result
    }
}

/// 工作窃取队列（用于负载均衡）
pub struct WorkStealingQueue<T> {
    tasks: Arc<Mutex<Vec<T>>>,
}

impl<T> WorkStealingQueue<T> {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, task: T) {
        let mut tasks = self.tasks.lock()
            .expect("WorkStealingQueue tasks lock should not be poisoned");
        tasks.push(task);
    }

    pub fn try_steal(&self) -> Option<T> {
        let mut tasks = self.tasks.lock()
            .expect("WorkStealingQueue tasks lock should not be poisoned");
        tasks.pop()
    }

    pub fn len(&self) -> usize {
        let tasks = self.tasks.lock()
            .expect("WorkStealingQueue tasks lock should not be poisoned");
        tasks.len()
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
        assert!(config.enable_work_stealing);
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
            enable_work_stealing: false,
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