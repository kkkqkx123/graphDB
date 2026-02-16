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

/// 执行器统计收集器
///
/// 用于在执行器执行过程中收集统计信息，支持嵌套执行器的统计聚合。
#[derive(Debug, Clone)]
pub struct ExecutorStatsCollector {
    /// 执行器ID
    pub executor_id: i64,
    /// 执行器类型名称
    pub executor_type: String,
    /// 开始时间
    start_time: std::time::Instant,
    /// 统计信息
    stats: ExecutorStats,
    /// 子执行器统计
    children: Vec<ExecutorStatsCollector>,
}

impl ExecutorStatsCollector {
    /// 创建新的统计收集器
    pub fn new(executor_id: i64, executor_type: impl Into<String>) -> Self {
        Self {
            executor_id,
            executor_type: executor_type.into(),
            start_time: std::time::Instant::now(),
            stats: ExecutorStats::new(),
            children: Vec::new(),
        }
    }

    /// 记录处理的行数
    pub fn record_rows(&mut self, count: usize) {
        self.stats.add_row(count);
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&mut self) {
        self.stats.record_cache_hit();
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&mut self) {
        self.stats.record_cache_miss();
    }

    /// 记录内存使用
    pub fn record_memory(&mut self, bytes: usize) {
        self.stats.update_memory_current(bytes);
        self.stats.set_memory_peak(bytes);
    }

    /// 添加子执行器统计
    pub fn add_child(&mut self, child: ExecutorStatsCollector) {
        self.children.push(child);
    }

    /// 完成统计收集，返回执行器统计
    pub fn finish(mut self) -> (ExecutorStatSnapshot, Vec<ExecutorStatSnapshot>) {
        let duration = self.start_time.elapsed();
        self.stats.add_exec_time(duration);
        self.stats.add_total_time(duration);

        let snapshot = ExecutorStatSnapshot {
            executor_id: self.executor_id,
            executor_type: self.executor_type,
            duration_ms: duration.as_millis() as u64,
            rows_processed: self.stats.num_rows,
            memory_used: self.stats.memory_peak,
            cache_hit_rate: self.stats.cache_hit_rate(),
        };

        let children_snapshots: Vec<ExecutorStatSnapshot> = self.children
            .into_iter()
            .map(|c| c.finish().0)
            .collect();

        (snapshot, children_snapshots)
    }

    /// 获取当前统计信息（不结束收集）
    pub fn current_stats(&self) -> &ExecutorStats {
        &self.stats
    }
}

/// 执行器统计快照
///
/// 执行完成后生成的不可变统计快照，用于存储和展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStatSnapshot {
    pub executor_id: i64,
    pub executor_type: String,
    pub duration_ms: u64,
    pub rows_processed: usize,
    pub memory_used: usize,
    pub cache_hit_rate: f64,
}

/// 全局执行器统计收集器
///
/// 用于在整个查询执行过程中收集所有执行器的统计信息。
#[derive(Debug, Default)]
pub struct QueryStatsCollector {
    /// 根执行器统计
    root_stats: Vec<ExecutorStatSnapshot>,
    /// 统计收集栈
    stack: Vec<ExecutorStatsCollector>,
}

impl QueryStatsCollector {
    /// 创建新的查询统计收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始收集一个执行器的统计
    pub fn start_executor(&mut self, executor_id: i64, executor_type: impl Into<String>) {
        let collector = ExecutorStatsCollector::new(executor_id, executor_type);
        self.stack.push(collector);
    }

    /// 结束当前执行器的统计收集
    pub fn end_executor(&mut self) -> Option<(ExecutorStatSnapshot, Vec<ExecutorStatSnapshot>)> {
        if let Some(collector) = self.stack.pop() {
            let (snapshot, children) = collector.finish();
            
            // 如果有父执行器，将当前统计添加到父执行器
            if let Some(parent) = self.stack.last_mut() {
                let child_collector = ExecutorStatsCollector {
                    executor_id: snapshot.executor_id,
                    executor_type: snapshot.executor_type.clone(),
                    start_time: std::time::Instant::now(),
                    stats: ExecutorStats {
                        num_rows: snapshot.rows_processed,
                        exec_time_us: snapshot.duration_ms * 1000,
                        total_time_us: snapshot.duration_ms * 1000,
                        memory_peak: snapshot.memory_used,
                        ..Default::default()
                    },
                    children: Vec::new(),
                };
                parent.add_child(child_collector);
            } else {
                // 根执行器
                self.root_stats.push(snapshot.clone());
            }
            
            Some((snapshot, children))
        } else {
            None
        }
    }

    /// 记录当前执行器处理的行数
    pub fn record_rows(&mut self, count: usize) {
        if let Some(current) = self.stack.last_mut() {
            current.record_rows(count);
        }
    }

    /// 记录当前执行器的缓存命中
    pub fn record_cache_hit(&mut self) {
        if let Some(current) = self.stack.last_mut() {
            current.record_cache_hit();
        }
    }

    /// 记录当前执行器的缓存未命中
    pub fn record_cache_miss(&mut self) {
        if let Some(current) = self.stack.last_mut() {
            current.record_cache_miss();
        }
    }

    /// 获取所有根执行器统计
    pub fn get_root_stats(&self) -> &[ExecutorStatSnapshot] {
        &self.root_stats
    }

    /// 获取所有统计（包括嵌套）
    pub fn get_all_stats(&self) -> Vec<ExecutorStatSnapshot> {
        let mut all_stats = self.root_stats.clone();
        
        for collector in &self.stack {
            let (snapshot, _) = collector.clone().finish();
            all_stats.push(snapshot);
        }
        
        all_stats
    }

    /// 清空所有统计
    pub fn clear(&mut self) {
        self.root_stats.clear();
        self.stack.clear();
    }

    /// 检查是否正在收集统计
    pub fn is_collecting(&self) -> bool {
        !self.stack.is_empty()
    }

    /// 获取当前执行器ID
    pub fn current_executor_id(&self) -> Option<i64> {
        self.stack.last().map(|c| c.executor_id)
    }
}
