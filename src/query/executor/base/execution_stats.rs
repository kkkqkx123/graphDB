//! 执行统计模块
//!
//! 提供执行过程中的统计信息收集

/// 节点执行统计信息
#[derive(Debug, Clone, Default)]
pub struct NodeExecutionStats {
    /// 实际执行时间（毫秒）
    pub actual_time_ms: f64,
    /// 实际输出行数
    pub actual_rows: u64,
    /// 实际循环次数
    pub actual_loops: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
}

impl NodeExecutionStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取总执行时间
    pub fn total_time_ms(&self) -> f64 {
        self.actual_time_ms * self.actual_loops as f64
    }

    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// 记录执行
    pub fn record_execution(&mut self, time_ms: f64, rows: u64) {
        self.actual_time_ms = time_ms;
        self.actual_rows = rows;
        self.actual_loops += 1;
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_execution_stats() {
        let mut stats = NodeExecutionStats::new();
        stats.record_execution(10.0, 100);
        stats.record_cache_hit();
        stats.record_cache_miss();

        assert_eq!(stats.actual_time_ms, 10.0);
        assert_eq!(stats.actual_rows, 100);
        assert_eq!(stats.actual_loops, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_total_time_ms() {
        let mut stats = NodeExecutionStats::new();
        stats.record_execution(10.0, 100);
        stats.record_execution(20.0, 200);

        assert_eq!(stats.actual_loops, 2);
        assert_eq!(stats.total_time_ms(), 40.0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut stats = NodeExecutionStats::new();
        stats.record_cache_hit();
        stats.record_cache_hit();
        stats.record_cache_miss();

        assert_eq!(stats.cache_hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_cache_hit_rate_no_access() {
        let stats = NodeExecutionStats::new();
        assert_eq!(stats.cache_hit_rate(), 0.0);
    }
}
