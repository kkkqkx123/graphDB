//! 执行器统计信息
//!
//! 用于记录执行器执行过程中的各种统计信息，包括处理行数、执行时间等。

use std::collections::HashMap;
use std::time::Duration;

/// 执行器统计信息
///
/// 记录执行器执行过程中的统计数据，用于性能分析和查询优化。
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    /// 处理的行数
    pub num_rows: usize,
    /// 执行时间（微秒）
    pub exec_time_us: u64,
    /// 总时间（微秒）
    pub total_time_us: u64,
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

    /// 添加自定义统计信息
    pub fn add_stat(&mut self, key: String, value: String) {
        self.other_stats.insert(key, value);
    }

    /// 获取自定义统计信息
    pub fn get_stat(&self, key: &str) -> Option<&String> {
        self.other_stats.get(key)
    }
}
