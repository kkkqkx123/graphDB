//! 执行器统计信息
//!
//! 用于记录执行器执行过程中的各种统计信息，包括处理行数、执行时间等。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 执行器统计信息
///
/// 记录执行器执行过程中的统计数据，用于性能分析和查询优化。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutorStats {
    /// 处理的行数
    pub num_rows: usize,
    /// 执行时间（微秒）
    pub exec_time_us: u64,
    /// 总时间（微秒）
    pub total_time_us: u64,
    /// 内存使用峰值（字节）
    pub memory_peak: usize,
    /// 内存使用当前值（字节）
    pub memory_current: usize,
    /// 批量操作次数
    pub batch_count: usize,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 缓存未命中次数
    pub cache_misses: usize,
    /// 其他统计信息
    pub other_stats: HashMap<String, String>,
}

impl ExecutorStats {
    /// 创建新的统计信息实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 增加处理的行数
    pub fn add_row(&mut self, count: usize) {
        self.num_rows += count;
    }

    /// 增加执行时间
    pub fn add_exec_time(&mut self, duration: Duration) {
        self.exec_time_us += duration.as_micros() as u64;
    }

    /// 增加总时间
    pub fn add_total_time(&mut self, duration: Duration) {
        self.total_time_us += duration.as_micros() as u64;
    }

    /// 设置内存使用峰值
    pub fn set_memory_peak(&mut self, peak: usize) {
        if peak > self.memory_peak {
            self.memory_peak = peak;
        }
    }

    /// 更新内存当前使用量
    pub fn update_memory_current(&mut self, current: usize) {
        self.memory_current = current;
    }

    /// 增加批量操作次数
    pub fn add_batch(&mut self, count: usize) {
        self.batch_count += count;
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// 计算缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// 添加自定义统计信息
    pub fn add_stat(&mut self, key: String, value: String) {
        self.other_stats.insert(key, value);
    }

    /// 获取自定义统计信息
    pub fn get_stat(&self, key: &str) -> Option<&String> {
        self.other_stats.get(key)
    }

    /// 获取吞吐量（行/秒）
    pub fn throughput_rows_per_sec(&self) -> f64 {
        if self.total_time_us > 0 {
            self.num_rows as f64 * 1_000_000.0 / self.total_time_us as f64
        } else {
            0.0
        }
    }

    /// 获取执行效率（行/微秒）
    pub fn efficiency_rows_per_us(&self) -> f64 {
        if self.exec_time_us > 0 {
            self.num_rows as f64 / self.exec_time_us as f64
        } else {
            0.0
        }
    }

    /// 导出为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串导入
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 导出为格式化字符串
    pub fn to_formatted_string(&self) -> String {
        format!(
            "rows: {}, exec_time: {}us, total_time: {}us, memory_peak: {}B, \
             memory_current: {}B, batches: {}, cache_hits: {}, cache_misses: {}, \
             cache_hit_rate: {:.2}%, throughput: {:.2} rows/sec",
            self.num_rows,
            self.exec_time_us,
            self.total_time_us,
            self.memory_peak,
            self.memory_current,
            self.batch_count,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate() * 100.0,
            self.throughput_rows_per_sec()
        )
    }
}
