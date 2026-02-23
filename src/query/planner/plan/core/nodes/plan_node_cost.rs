//! 计划节点成本和统计结构
//!
//! 定义计划节点的代价估算和统计信息结构

/// 计划节点代价估算
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlanNodeCost {
    /// 启动成本（获取第一行前的成本）
    pub startup_cost: f64,
    /// 总成本
    pub total_cost: f64,
    /// 估计输出行数
    pub estimated_rows: u64,
    /// 估计输出宽度（字节）
    pub estimated_width: u64,
}

impl PlanNodeCost {
    /// 创建新的代价估算
    pub fn new(startup_cost: f64, total_cost: f64, estimated_rows: u64, estimated_width: u64) -> Self {
        Self {
            startup_cost,
            total_cost,
            estimated_rows,
            estimated_width,
        }
    }

    /// 创建零成本
    pub fn zero() -> Self {
        Self::default()
    }

    /// 创建具有指定行数和宽度的代价
    pub fn with_rows_and_width(rows: u64, width: u64) -> Self {
        Self {
            estimated_rows: rows,
            estimated_width: width,
            ..Default::default()
        }
    }

    /// 计算每行的平均成本
    pub fn cost_per_row(&self) -> f64 {
        if self.estimated_rows == 0 {
            0.0
        } else {
            self.total_cost / self.estimated_rows as f64
        }
    }

    /// 合并两个代价（用于连接等操作）
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            startup_cost: self.startup_cost + other.startup_cost,
            total_cost: self.total_cost + other.total_cost,
            estimated_rows: self.estimated_rows.max(other.estimated_rows),
            estimated_width: self.estimated_width + other.estimated_width,
        }
    }

    /// 添加成本
    pub fn add_cost(&mut self, cost: f64) {
        self.total_cost += cost;
    }

    /// 添加启动成本
    pub fn add_startup_cost(&mut self, cost: f64) {
        self.startup_cost += cost;
    }

    /// 设置估计行数
    pub fn set_estimated_rows(&mut self, rows: u64) {
        self.estimated_rows = rows;
    }

    /// 设置估计宽度
    pub fn set_estimated_width(&mut self, width: u64) {
        self.estimated_width = width;
    }
}

/// 计划节点统计信息
#[derive(Debug, Clone, Default)]
pub struct PlanNodeStats {
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

impl PlanNodeStats {
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

/// 代价比较结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostComparison {
    /// 更便宜
    Cheaper,
    /// 相等
    Equal,
    /// 更贵
    MoreExpensive,
}

impl PlanNodeCost {
    /// 比较两个代价
    pub fn compare(&self, other: &Self) -> CostComparison {
        if self.total_cost < other.total_cost {
            CostComparison::Cheaper
        } else if self.total_cost > other.total_cost {
            CostComparison::MoreExpensive
        } else {
            CostComparison::Equal
        }
    }

    /// 是否比另一个更便宜
    pub fn is_cheaper_than(&self, other: &Self) -> bool {
        matches!(self.compare(other), CostComparison::Cheaper)
    }

    /// 是否比另一个更贵
    pub fn is_more_expensive_than(&self, other: &Self) -> bool {
        matches!(self.compare(other), CostComparison::MoreExpensive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_cost_default() {
        let cost = PlanNodeCost::default();
        assert_eq!(cost.startup_cost, 0.0);
        assert_eq!(cost.total_cost, 0.0);
        assert_eq!(cost.estimated_rows, 0);
        assert_eq!(cost.estimated_width, 0);
    }

    #[test]
    fn test_plan_node_cost_new() {
        let cost = PlanNodeCost::new(10.0, 100.0, 1000, 50);
        assert_eq!(cost.startup_cost, 10.0);
        assert_eq!(cost.total_cost, 100.0);
        assert_eq!(cost.estimated_rows, 1000);
        assert_eq!(cost.estimated_width, 50);
    }

    #[test]
    fn test_cost_per_row() {
        let cost = PlanNodeCost::new(0.0, 100.0, 10, 0);
        assert_eq!(cost.cost_per_row(), 10.0);
    }

    #[test]
    fn test_cost_merge() {
        let cost1 = PlanNodeCost::new(10.0, 100.0, 100, 50);
        let cost2 = PlanNodeCost::new(5.0, 50.0, 50, 25);
        let merged = cost1.merge(&cost2);
        assert_eq!(merged.startup_cost, 15.0);
        assert_eq!(merged.total_cost, 150.0);
        assert_eq!(merged.estimated_rows, 100); // max
        assert_eq!(merged.estimated_width, 75); // sum
    }

    #[test]
    fn test_cost_comparison() {
        let cost1 = PlanNodeCost::new(0.0, 100.0, 0, 0);
        let cost2 = PlanNodeCost::new(0.0, 200.0, 0, 0);
        assert!(cost1.is_cheaper_than(&cost2));
        assert!(cost2.is_more_expensive_than(&cost1));
    }

    #[test]
    fn test_plan_node_stats() {
        let mut stats = PlanNodeStats::new();
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
}
