//! 计划节点成本和统计结构
//!
//! 提供成本估计和统计信息相关的结构体，用于查询优化
//!
//! 注意：当前为单节点数据库设计，仅保留核心统计信息，
//! 移除了CPU、磁盘IO、网络等分布式场景才需要的指标

/// 节点执行统计
#[derive(Debug, Clone, Default)]
pub struct NodeStatistics {
    /// 估计处理的行数
    pub estimated_rows: u64,
    /// 实际处理的行数
    pub actual_rows: u64,
    /// 实际执行时间（微秒）
    pub exec_time_us: u64,
    /// 峰值内存使用（字节）
    pub peak_memory_bytes: u64,
}

impl NodeStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_estimated_rows(rows: u64) -> Self {
        Self {
            estimated_rows: rows,
            ..Default::default()
        }
    }
}

/// 成本估计
/// 单节点数据库使用简化的单一成本值
#[derive(Debug, Clone, Default, Copy)]
pub struct CostEstimate {
    /// 总成本（综合CPU、内存、IO等因素的单一值）
    pub total_cost: f64,
    /// 估计输出行数
    pub output_rows: u64,
}

impl CostEstimate {
    pub fn new(total_cost: f64, output_rows: u64) -> Self {
        Self {
            total_cost,
            output_rows,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0)
    }

    pub fn add(&self, other: &CostEstimate) -> Self {
        Self::new(
            self.total_cost + other.total_cost,
            self.output_rows.max(other.output_rows),
        )
    }

    pub fn multiply(&self, factor: f64) -> Self {
        Self::new(
            self.total_cost * factor,
            (self.output_rows as f64 * factor) as u64,
        )
    }
}
