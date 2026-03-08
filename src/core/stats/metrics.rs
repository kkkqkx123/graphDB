//! 轻量级查询指标
//!
//! 用于返回给客户端的轻量级查询指标，使用微秒级精度。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 轻量级查询指标
///
/// 用于返回给客户端的查询执行指标，使用微秒级精度。
/// 与 QueryProfile 的区别：
/// - QueryMetrics: 轻量级，用于返回给客户端（微秒级）
/// - QueryProfile: 详细监控，用于内部分析和日志（毫秒级）
#[derive(Debug, Clone)]
pub struct QueryMetrics {
    /// 解析时间（微秒）
    pub parse_time_us: u64,
    /// 验证时间（微秒）
    pub validate_time_us: u64,
    /// 规划时间（微秒）
    pub plan_time_us: u64,
    /// 优化时间（微秒）
    pub optimize_time_us: u64,
    /// 执行时间（微秒）
    pub execute_time_us: u64,
    /// 总时间（微秒）
    pub total_time_us: u64,
    /// 计划节点数量
    pub plan_node_count: usize,
    /// 结果行数
    pub result_row_count: usize,
    /// 时间戳
    pub timestamp: Instant,
}

impl Default for QueryMetrics {
    fn default() -> Self {
        Self {
            parse_time_us: 0,
            validate_time_us: 0,
            plan_time_us: 0,
            optimize_time_us: 0,
            execute_time_us: 0,
            total_time_us: 0,
            plan_node_count: 0,
            result_row_count: 0,
            timestamp: Instant::now(),
        }
    }
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_parse_time(&mut self, duration: Duration) {
        self.parse_time_us = duration.as_micros() as u64;
    }

    pub fn record_validate_time(&mut self, duration: Duration) {
        self.validate_time_us = duration.as_micros() as u64;
    }

    pub fn record_plan_time(&mut self, duration: Duration) {
        self.plan_time_us = duration.as_micros() as u64;
    }

    pub fn record_optimize_time(&mut self, duration: Duration) {
        self.optimize_time_us = duration.as_micros() as u64;
    }

    pub fn record_execute_time(&mut self, duration: Duration) {
        self.execute_time_us = duration.as_micros() as u64;
    }

    pub fn record_total_time(&mut self, duration: Duration) {
        self.total_time_us = duration.as_micros() as u64;
    }

    pub fn set_plan_node_count(&mut self, count: usize) {
        self.plan_node_count = count;
    }

    pub fn set_result_row_count(&mut self, count: usize) {
        self.result_row_count = count;
    }

    pub fn to_map(&self) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert("parse_time_us".to_string(), self.parse_time_us);
        map.insert("validate_time_us".to_string(), self.validate_time_us);
        map.insert("plan_time_us".to_string(), self.plan_time_us);
        map.insert("optimize_time_us".to_string(), self.optimize_time_us);
        map.insert("execute_time_us".to_string(), self.execute_time_us);
        map.insert("total_time_us".to_string(), self.total_time_us);
        map.insert("plan_node_count".to_string(), self.plan_node_count as u64);
        map.insert("result_row_count".to_string(), self.result_row_count as u64);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_metrics_default() {
        let metrics = QueryMetrics::default();
        assert_eq!(metrics.parse_time_us, 0);
        assert_eq!(metrics.total_time_us, 0);
    }

    #[test]
    fn test_record_parse_time() {
        let mut metrics = QueryMetrics::new();
        metrics.record_parse_time(Duration::from_micros(100));
        assert_eq!(metrics.parse_time_us, 100);
    }

    #[test]
    fn test_to_map() {
        let mut metrics = QueryMetrics::new();
        metrics.parse_time_us = 100;
        metrics.total_time_us = 1000;

        let map = metrics.to_map();
        assert_eq!(map.get("parse_time_us"), Some(&100));
        assert_eq!(map.get("total_time_us"), Some(&1000));
    }
}
