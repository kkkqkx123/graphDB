//! 优化的哈希表实现，支持内存管理和溢出处理
//!
//! 提供高效的哈希表用于join操作，支持内存限制和磁盘溢出

use crate::core::types::expression::Expr;
use crate::core::{DBError, DBResult, DataSet, Value};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::common::memory::{MemoryConfig, MemoryTracker};
use bincode::{decode_from_slice, encode_to_vec, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Join键，支持高效的哈希和序列化
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
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

    /// 估算内存大小（字节）
    pub fn estimated_size(&self) -> usize {
        // 基础结构大小 + 向量开销 + 值大小估算
        std::mem::size_of::<Self>()
            + self.values.capacity() * std::mem::size_of::<Value>()
            + self
                .values
                .iter()
                .map(|v| Self::estimate_value_size(v))
                .sum::<usize>()
    }

    fn estimate_value_size(value: &Value) -> usize {
        match value {
            Value::Int(_) | Value::Float(_) | Value::Bool(_) => 8,
            Value::String(s) => s.len(),
            Value::List(l) => l.iter().map(Self::estimate_value_size).sum::<usize>() + 24,
            Value::Map(m) => {
                m.iter()
                    .map(|(k, v)| k.len() + Self::estimate_value_size(v))
                    .sum::<usize>()
                    + 48
            }
            _ => 16, // 其他类型的估算
        }
    }
}

impl Hash for JoinKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.cached_hash);
    }
}

/// 哈希表条目，包含行数据和元信息
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HashTableEntry {
    /// 行数据
    pub row: Vec<Value>,
    /// 原始行索引（用于调试和重复数据处理）
    pub original_index: usize,
    /// 条目大小估算
    pub estimated_size: usize,
}

impl HashTableEntry {
    pub fn new(row: Vec<Value>, original_index: usize) -> Self {
        let estimated_size = Self::estimate_row_size(&row);
        Self {
            row,
            original_index,
            estimated_size,
        }
    }

    fn estimate_row_size(row: &[Value]) -> usize {
        std::mem::size_of::<Self>()
            + row
                .iter()
                .map(|v| JoinKey::estimate_value_size(v))
                .sum::<usize>()
            + row.len() * std::mem::size_of::<Value>()
    }
}

/// LRU追踪器，用于管理内存中的键访问顺序
#[derive(Debug)]
struct LruTracker {
    /// 访问顺序队列，最新的在前
    access_order: VecDeque<JoinKey>,
    /// 最大容量
    max_capacity: usize,
}

impl LruTracker {
    fn new(max_capacity: usize) -> Self {
        Self {
            access_order: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// 记录键访问
    fn record_access(&mut self, key: &JoinKey) {
        // 移除已存在的键（如果存在）
        self.access_order.retain(|k| k != key);

        // 添加到队列前端
        self.access_order.push_front(key.clone());

        // 如果超过容量，移除最老的
        if self.access_order.len() > self.max_capacity {
            self.access_order.pop_back();
        }
    }

    /// 获取最老的键（LRU）
    fn get_lru_keys(&self, count: usize) -> Vec<JoinKey> {
        self.access_order
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// 移除指定的键
    fn remove_key(&mut self, key: &JoinKey) {
        self.access_order.retain(|k| k != key);
    }

    /// 清空追踪器
    fn clear(&mut self) {
        self.access_order.clear();
    }
}

/// 溢出文件管理器
pub struct SpillManager {
    /// 溢出文件目录
    spill_dir: PathBuf,
    /// 当前溢出文件
    current_file: Option<BufWriter<File>>,
    /// 溢出文件计数
    file_counter: usize,
    /// 每个溢出文件的最大大小
    max_file_size: usize,
    /// 当前文件大小
    current_file_size: usize,
}

impl SpillManager {
    pub fn new(spill_dir: impl AsRef<Path>, max_file_size: usize) -> DBResult<Self> {
        let spill_dir = spill_dir.as_ref().to_path_buf();

        // 创建溢出目录
        std::fs::create_dir_all(&spill_dir).map_err(|e| DBError::Io(e))?;

        Ok(Self {
            spill_dir,
            current_file: None,
            file_counter: 0,
            max_file_size,
            current_file_size: 0,
        })
    }

    /// 写入溢出数据
    pub fn spill_entry(&mut self, key: &JoinKey, entries: &[HashTableEntry]) -> DBResult<()> {
        // 检查是否需要新文件
        if self.current_file.is_none() || self.current_file_size >= self.max_file_size {
            self.rotate_file()?;
        }

        if let Some(ref mut writer) = self.current_file {
            // 序列化并写入数据
            let serialized =
                encode_to_vec(&(key, entries), bincode::config::standard()).map_err(|e| {
                    DBError::Serialization(format!("Failed to serialize spill data: {}", e))
                })?;

            let data_size = serialized.len() as usize;

            // 写入大小前缀
            writer
                .write_all(&(data_size as u32).to_le_bytes())
                .map_err(|e| DBError::Io(e))?;

            // 写入数据
            writer.write_all(&serialized).map_err(|e| DBError::Io(e))?;

            self.current_file_size += data_size + 4; // +4 for size prefix
        }

        Ok(())
    }

    /// 旋转到新文件
    fn rotate_file(&mut self) -> DBResult<()> {
        // 关闭当前文件
        if let Some(mut writer) = self.current_file.take() {
            writer.flush().map_err(|e| DBError::Io(e))?;
        }

        // 创建新文件
        let file_path = self
            .spill_dir
            .join(format!("spill_{}.dat", self.file_counter));
        self.file_counter += 1;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| DBError::Io(e))?;

        self.current_file = Some(BufWriter::new(file));
        self.current_file_size = 0;

        Ok(())
    }

    /// 读取所有溢出数据
    pub fn read_all_spills(&self) -> DBResult<Vec<(JoinKey, Vec<HashTableEntry>)>> {
        let mut results = Vec::new();

        for i in 0..self.file_counter {
            let file_path = self.spill_dir.join(format!("spill_{}.dat", i));
            if file_path.exists() {
                let file = File::open(&file_path).map_err(|e| DBError::Io(e))?;
                let mut reader = BufReader::new(file);

                loop {
                    // 读取大小前缀
                    let mut size_bytes = [0u8; 4];
                    match reader.read_exact(&mut size_bytes) {
                        Ok(_) => {
                            let data_size = u32::from_le_bytes(size_bytes) as usize;

                            // 读取数据
                            let mut data = vec![0u8; data_size];
                            reader.read_exact(&mut data).map_err(|e| DBError::Io(e))?;

                            // 反序列化
                            let (key, entries): (JoinKey, Vec<HashTableEntry>) =
                                decode_from_slice(&data, bincode::config::standard())
                                    .map_err(|e| {
                                        DBError::Serialization(format!(
                                            "Failed to deserialize spill data: {}",
                                            e
                                        ))
                                    })?
                                    .0;

                            results.push((key, entries));
                        }
                        Err(_) => break, // 文件结束
                    }
                }
            }
        }

        Ok(results)
    }

    /// 清理溢出文件
    pub fn cleanup(&self) -> DBResult<()> {
        for i in 0..self.file_counter {
            let file_path = self.spill_dir.join(format!("spill_{}.dat", i));
            if file_path.exists() {
                std::fs::remove_file(&file_path).map_err(|e| DBError::Io(e))?;
            }
        }

        Ok(())
    }
}

impl Drop for SpillManager {
    fn drop(&mut self) {
        // 刷新当前文件
        if let Some(ref mut writer) = self.current_file {
            let _ = writer.flush();
        }
    }
}

/// 哈希表，支持内存管理和溢出
pub struct HashTable {
    /// 内存中的哈希表
    memory_table: HashMap<JoinKey, Vec<HashTableEntry>>,
    /// 内存跟踪器
    memory_tracker: Arc<MemoryTracker>,
    /// 溢出管理器（可选）
    spill_manager: Option<SpillManager>,
    /// 配置
    config: HashTableConfig,
    /// 统计信息
    stats: Arc<Mutex<HashTableStats>>,
    /// LRU访问追踪器
    lru_tracker: Arc<Mutex<LruTracker>>,
}

/// 哈希表配置
#[derive(Debug, Clone)]
pub struct HashTableConfig {
    /// 内存配置
    pub memory_config: MemoryConfig,
    /// 溢出目录
    pub spill_dir: Option<PathBuf>,
    /// 每个溢出文件的最大大小
    pub max_spill_file_size: usize,
    /// 初始容量
    pub initial_capacity: usize,
}

impl Default for HashTableConfig {
    fn default() -> Self {
        Self {
            memory_config: MemoryConfig {
                spill_enabled: false, // 禁用溢出，避免测试需要配置 spill_dir
                ..Default::default()
            },
            spill_dir: None,
            max_spill_file_size: 50 * 1024 * 1024, // 50MB
            initial_capacity: 10000,
        }
    }
}

/// 哈希表统计信息
#[derive(Debug, Clone, Default)]
pub struct HashTableStats {
    pub total_entries: usize,
    pub memory_entries: usize,
    pub spilled_entries: usize,
    pub memory_usage: usize,
    pub spill_file_count: usize,
    pub probe_count: usize,
    pub hit_count: usize,
}

impl HashTable {
    /// 创建新的哈希表
    pub fn new(memory_tracker: Arc<MemoryTracker>, config: HashTableConfig) -> DBResult<Self> {
        let initial_capacity = config.initial_capacity;

        let spill_manager = if config.spill_dir.is_some() && config.memory_config.spill_enabled {
            let spill_dir = config.spill_dir.as_ref().expect("Spill directory should exist when spill is enabled");
            Some(SpillManager::new(
                spill_dir,
                config.max_spill_file_size,
            )?)
        } else {
            None
        };

        Ok(Self {
            memory_table: HashMap::with_capacity(initial_capacity),
            memory_tracker,
            spill_manager,
            config,
            stats: Arc::new(Mutex::new(HashTableStats::default())),
            lru_tracker: Arc::new(Mutex::new(LruTracker::new(initial_capacity))),
        })
    }

    /// 插入条目
    pub fn insert(&mut self, key: JoinKey, entry: HashTableEntry) -> DBResult<()> {
        // 记录内存使用
        let entry_size = entry.estimated_size;
        self.memory_tracker.record_allocation(entry_size);

        // 检查是否需要溢出
        if self.should_spill() {
            self.spill_to_disk()?;
        }

        // 插入到内存表
        let entries = self.memory_table.entry(key.clone()).or_insert_with(Vec::new);

        entries.push(entry);

        // 记录LRU访问
        if let Ok(mut tracker) = self.lru_tracker.lock() {
            tracker.record_access(&key);
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.total_entries += 1;
            stats.memory_entries += 1;
            stats.memory_usage += entry_size;
        }

        Ok(())
    }

    /// 检查是否应该溢出
    fn should_spill(&self) -> bool {
        self.memory_tracker.current_allocated() > self.config.memory_config.max_query_memory as usize 
            && self.spill_manager.is_some()
    }

    /// 溢出到磁盘
    fn spill_to_disk(&mut self) -> DBResult<()> {
        if let Some(ref mut spill_manager) = self.spill_manager {
            // 使用LRU策略选择要溢出的键（溢出25%的键）
            let spill_count = std::cmp::max(1, self.memory_table.len() / 4);
            let mut spilled_count = 0;

            // 从LRU追踪器获取最久未使用的键
            let keys_to_spill = {
                if let Ok(tracker) = self.lru_tracker.lock() {
                    tracker.get_lru_keys(spill_count)
                } else {
                    Vec::new()
                }
            };

            for key in keys_to_spill {
                if let Some(entries) = self.memory_table.remove(&key) {
                    let entries_vec: Vec<_> = entries.as_slice().to_vec();
                    let total_size: usize = entries_vec.iter().map(|e| e.estimated_size).sum();

                    spill_manager.spill_entry(&key, &entries_vec)?;

                    // 从LRU追踪器中移除键
                    if let Ok(mut tracker) = self.lru_tracker.lock() {
                        tracker.remove_key(&key);
                    }

                    if let Ok(mut stats) = self.stats.lock() {
                        stats.memory_entries -= entries_vec.len();
                        stats.spilled_entries += entries_vec.len();
                        stats.memory_usage -= total_size;
                        stats.spill_file_count += 1;
                    }

                    spilled_count += 1;
                }
            }

            if spilled_count > 0 {
                // 重置内存计数器
                self.memory_tracker.reset();
            }
        }

        Ok(())
    }

    /// 探测哈希表
    pub fn probe(&self, key: &JoinKey) -> Vec<HashTableEntry> {
        if let Ok(mut stats) = self.stats.lock() {
            stats.probe_count += 1;
        }

        // 记录LRU访问
        if let Ok(mut tracker) = self.lru_tracker.lock() {
            tracker.record_access(key);
        }

        // 先在内存中查找
        if let Some(entries) = self.memory_table.get(key) {
            if let Ok(mut stats) = self.stats.lock() {
                stats.hit_count += 1;
            }
            return entries.as_slice().to_vec();
        }

        // 如果启用了溢出，需要在溢出文件中查找
        if self.spill_manager.is_some() {
            self.probe_spilled_data(key)
        } else {
            Vec::new()
        }
    }

    /// 在溢出数据中探测
    fn probe_spilled_data(&self, key: &JoinKey) -> Vec<HashTableEntry> {
        if let Some(ref spill_manager) = self.spill_manager {
            match spill_manager.read_all_spills() {
                Ok(spill_data) => {
                    for (spill_key, entries) in spill_data {
                        if &spill_key == key {
                            return entries;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取溢出数据失败: {}", e);
                }
            }
        }
        Vec::new()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> HashTableStats {
        self.stats.lock().expect("Failed to acquire lock on hash table stats").clone()
    }

    /// 获取内存使用量
    pub fn memory_usage(&self) -> usize {
        self.stats.lock().expect("Failed to acquire lock on hash table stats").memory_usage
    }

    /// 清理资源
    pub fn cleanup(&mut self) -> DBResult<()> {
        if let Some(ref mut spill_manager) = self.spill_manager {
            spill_manager.cleanup()?;
        }

        self.memory_table.clear();
        self.memory_tracker.reset();

        Ok(())
    }
}

/// 哈希表构建器
pub struct HashTableBuilder;

impl HashTableBuilder {
    /// 从数据集构建哈希表
    pub fn build_from_dataset(
        dataset: &DataSet,
        key_indices: &[usize],
        memory_tracker: Arc<MemoryTracker>,
        config: HashTableConfig,
    ) -> DBResult<HashTable> {
        let mut hash_table = HashTable::new(memory_tracker, config)?;

        for (idx, row) in dataset.rows.iter().enumerate() {
            // 构建连接键
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

    /// 构建单键哈希表（向后兼容）
    pub fn build_single_key_table(
        dataset: &DataSet,
        key_index: usize,
    ) -> Result<SingleKeyHashTable, String> {
        let memory_tracker = Arc::new(MemoryTracker::new());

        let config = HashTableConfig::default();

        HashTableBuilder::build_from_dataset(dataset, &[key_index], memory_tracker, config)
            .map_err(|e| e.to_string())
    }

    /// 构建多键哈希表（向后兼容）
    pub fn build_multi_key_table(
        dataset: &DataSet,
        key_indices: &[usize],
    ) -> Result<MultiKeyHashTable, String> {
        let memory_tracker = Arc::new(MemoryTracker::new());

        let config = HashTableConfig::default();

        HashTableBuilder::build_from_dataset(dataset, key_indices, memory_tracker, config)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_key() {
        let key1 = JoinKey::new(vec![Value::Int(1), Value::String("test".to_string())]);
        let key2 = JoinKey::new(vec![Value::Int(1), Value::String("test".to_string())]);
        let key3 = JoinKey::new(vec![Value::Int(2), Value::String("test".to_string())]);

        assert_eq!(key1, key2); // 相同内容应该相等
        assert_ne!(key1, key3); // 不同内容不应该相等

        // 测试哈希一致性
        use std::collections::hash_map::DefaultHasher;
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_table_entry() {
        let entry = HashTableEntry::new(vec![Value::Int(1), Value::String("test".to_string())], 0);

        assert_eq!(entry.original_index, 0);
        assert_eq!(entry.row.len(), 2);
        assert!(entry.estimated_size > 0);
    }

    #[tokio::test]
    async fn test_hash_table_basic() {
        let config = HashTableConfig::default();
        let memory_tracker = Arc::new(MemoryTracker::new());

        let mut hash_table = HashTable::new(memory_tracker, config).expect("HashTable::new should succeed");

        // 插入测试数据
        let key = JoinKey::new(vec![Value::Int(1)]);
        let entry = HashTableEntry::new(vec![Value::String("test".to_string())], 0);

        hash_table.insert(key.clone(), entry).expect("insert should succeed");

        // 探测测试
        let results = hash_table.probe(&key);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].row[0], Value::String("test".to_string()));

        // 检查统计
        let stats = hash_table.get_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.memory_entries, 1);

        // 清理
        hash_table.cleanup().expect("cleanup should succeed");
    }

    #[tokio::test]
    async fn test_optimized_hash_table_memory_limit() {
        let mut config = HashTableConfig::default();
        config.memory_config.max_query_memory = 100; // 很小的内存限制
        config.memory_config.spill_enabled = false; // 禁用溢出

        let memory_tracker = Arc::new(MemoryTracker::new());

        let mut hash_table = HashTable::new(memory_tracker, config).expect("HashTable::new should succeed");

        // 这里可以添加内存限制测试逻辑
        // 例如尝试插入大量数据并验证行为

        // 清理
        hash_table.cleanup().expect("cleanup should succeed");
    }
}

impl std::fmt::Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashTable")
            .field("memory_table_len", &self.memory_table.len())
            .field("memory_usage", &self.memory_usage())
            .field("stats", &self.get_stats())
            .finish()
    }
}

// 向后兼容接口
/// 单键哈希表类型别名
pub type SingleKeyHashTable = HashTable;

/// 多键哈希表类型别名  
pub type MultiKeyHashTable = HashTable;

/// 哈希表探测器（向后兼容接口）
pub struct HashTableProbe;

impl HashTableProbe {
    /// 单键探测（向后兼容）
    pub fn probe_single_key(
        hash_table: &SingleKeyHashTable,
        probe_dataset: &DataSet,
        key_index: usize,
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        let mut results = Vec::new();

        for probe_row in &probe_dataset.rows {
            if key_index < probe_row.len() {
                let key_value = &probe_row[key_index];
                let key = JoinKey::new(vec![key_value.clone()]);

                let matching_entries = hash_table.probe(&key);
                let matching_rows: Vec<Vec<Value>> = matching_entries
                    .into_iter()
                    .map(|entry| entry.row)
                    .collect();

                if !matching_rows.is_empty() {
                    results.push((probe_row.clone(), matching_rows));
                }
            }
        }

        results
    }

    /// 多键探测（向后兼容）
    pub fn probe_multi_key(
        hash_table: &MultiKeyHashTable,
        probe_dataset: &DataSet,
        key_indices: &[usize],
    ) -> Vec<(Vec<Value>, Vec<Vec<Value>>)> {
        let mut results = Vec::new();

        for probe_row in &probe_dataset.rows {
            let mut key_values = Vec::new();

            for &key_index in key_indices {
                if key_index < probe_row.len() {
                    key_values.push(probe_row[key_index].clone());
                }
            }

            if key_values.len() == key_indices.len() {
                let key = JoinKey::new(key_values);

                let matching_entries = hash_table.probe(&key);
                let matching_rows: Vec<Vec<Value>> = matching_entries
                    .into_iter()
                    .map(|entry| entry.row)
                    .collect();

                if !matching_rows.is_empty() {
                    results.push((probe_row.clone(), matching_rows));
                }
            }
        }

        results
    }
}

/// 构建哈希表函数（向后兼容，接受表达式）
pub fn build_hash_table(
    dataset: &DataSet,
    key_exprs: &[Expression],
) -> Result<HashMap<JoinKey, Vec<usize>>, String> {
    let mut hash_table = HashMap::new();
    let _evaluator = ExpressionEvaluator;

    for (idx, row) in dataset.rows.iter().enumerate() {
        // 创建表达式上下文
        let mut expr_context = DefaultExpressionContext::new();
        for (i, col_name) in dataset.col_names.iter().enumerate() {
            if i < row.len() {
                expr_context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        // 评估表达式获取键值
        let mut key_values = Vec::new();
        for key_expr in key_exprs {
            match ExpressionEvaluator::evaluate(key_expr, &mut expr_context) {
                Ok(value) => key_values.push(value),
                Err(e) => return Err(format!("键表达式求值失败: {}", e)),
            }
        }

        let key = JoinKey::new(key_values);
        hash_table.entry(key).or_insert_with(Vec::new).push(idx);
    }

    Ok(hash_table)
}

/// 构建哈希表函数（向后兼容，接受列索引）
pub fn build_hash_table_with_indices(
    dataset: &DataSet,
    key_indices: &[usize],
) -> Result<HashMap<JoinKey, Vec<usize>>, String> {
    let mut hash_table = HashMap::new();

    for (idx, row) in dataset.rows.iter().enumerate() {
        let mut key_values = Vec::new();
        for &key_index in key_indices {
            if key_index < row.len() {
                key_values.push(row[key_index].clone());
            }
        }

        if key_values.len() == key_indices.len() {
            let key = JoinKey::new(key_values);
            hash_table.entry(key).or_insert_with(Vec::new).push(idx);
        }
    }

    Ok(hash_table)
}

/// 提取键值函数（向后兼容，接受表达式）
pub fn extract_key_values(
    row: &[Value],
    col_names: &[String],
    key_exprs: &[Expression],
    _col_map: &std::collections::HashMap<&str, usize>,
) -> Vec<Value> {
    let mut expr_context = DefaultExpressionContext::new();
    for (i, col_name) in col_names.iter().enumerate() {
        if i < row.len() {
            expr_context.set_variable(col_name.clone(), row[i].clone());
        }
    }

    let mut key_values = Vec::new();

    for key_expr in key_exprs {
        if let Ok(value) = ExpressionEvaluator::evaluate(key_expr, &mut expr_context) {
            key_values.push(value);
        }
    }

    key_values
}

/// 提取键值函数（向后兼容，接受列索引）
pub fn extract_key_values_with_indices(row: &[Value], key_indices: &[usize]) -> Vec<Value> {
    key_indices
        .iter()
        .filter_map(|&idx| row.get(idx).cloned())
        .collect()
}
