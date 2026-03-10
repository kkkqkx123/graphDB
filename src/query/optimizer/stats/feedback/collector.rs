//! 执行反馈收集器模块
//!
//! 提供轻量级的执行反馈收集机制，用于收集查询执行的实际统计信息。

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 执行反馈收集器
///
/// 轻量级收集器，用于收集查询执行的实际统计信息。
/// 使用原子操作确保线程安全。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::collector::ExecutionFeedbackCollector;
///
/// let collector = ExecutionFeedbackCollector::new();
/// collector.start();
/// collector.record_rows(100);
/// let time_us = collector.finish();
/// assert_eq!(collector.get_actual_rows(), 100);
/// ```
#[derive(Debug)]
pub struct ExecutionFeedbackCollector {
    /// 实际输出行数（原子计数器）
    actual_rows: AtomicU64,
    /// 执行时间（微秒）
    execution_time_us: AtomicU64,
    /// 开始时间
    start_time: RwLock<Option<Instant>>,
}

impl ExecutionFeedbackCollector {
    /// 创建新的反馈收集器
    pub fn new() -> Self {
        Self {
            actual_rows: AtomicU64::new(0),
            execution_time_us: AtomicU64::new(0),
            start_time: RwLock::new(None),
        }
    }

    /// 开始收集
    ///
    /// 记录当前时间为开始时间。
    pub fn start(&self) {
        *self.start_time.write() = Some(Instant::now());
    }

    /// 记录输出行数
    ///
    /// 原子地增加输出行数计数。
    pub fn record_rows(&self, rows: u64) {
        self.actual_rows.fetch_add(rows, Ordering::Relaxed);
    }

    /// 结束收集并返回执行时间（微秒）
    ///
    /// 计算从开始到当前的经过时间，并存储执行时间。
    pub fn finish(&self) -> u64 {
        let elapsed = self
            .start_time
            .read()
            .map(|start| start.elapsed().as_micros() as u64)
            .unwrap_or(0);
        self.execution_time_us.store(elapsed, Ordering::Relaxed);
        elapsed
    }

    /// 获取实际输出行数
    pub fn get_actual_rows(&self) -> u64 {
        self.actual_rows.load(Ordering::Relaxed)
    }

    /// 获取执行时间（微秒）
    pub fn get_execution_time_us(&self) -> u64 {
        self.execution_time_us.load(Ordering::Relaxed)
    }

    /// 重置收集器
    ///
    /// 清除所有收集的数据，恢复到初始状态。
    pub fn reset(&self) {
        self.actual_rows.store(0, Ordering::Relaxed);
        self.execution_time_us.store(0, Ordering::Relaxed);
        *self.start_time.write() = None;
    }
}

impl Default for ExecutionFeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_feedback_collector() {
        let collector = ExecutionFeedbackCollector::new();
        collector.start();
        collector.record_rows(100);
        collector.record_rows(50);

        let time = collector.finish();
        assert_eq!(collector.get_actual_rows(), 150);
        assert_eq!(collector.get_execution_time_us(), time);
        assert!(time > 0);
    }

    #[test]
    fn test_collector_reset() {
        let collector = ExecutionFeedbackCollector::new();
        collector.start();
        collector.record_rows(100);
        collector.finish();

        collector.reset();
        assert_eq!(collector.get_actual_rows(), 0);
        assert_eq!(collector.get_execution_time_us(), 0);
    }

    #[test]
    fn test_collector_without_start() {
        let collector = ExecutionFeedbackCollector::new();
        // 不调用start直接finish
        let time = collector.finish();
        assert_eq!(time, 0);
        assert_eq!(collector.get_execution_time_us(), 0);
    }
}
