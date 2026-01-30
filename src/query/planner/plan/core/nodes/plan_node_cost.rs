//! 计划节点成本和统计结构
//!
//! 提供成本估计和统计信息相关的结构体，用于查询优化

use std::collections::HashMap;

/// 节点执行统计
#[derive(Debug, Clone, Default)]
pub struct NodeStatistics {
    /// 估计处理的行数
    pub estimated_rows: u64,
    /// 实际处理的行数
    pub actual_rows: u64,
    /// 估计输出大小（字节）
    pub estimated_output_size: u64,
    /// 实际执行时间（微秒）
    pub exec_time_us: u64,
    /// 峰值内存使用（字节）
    pub peak_memory_bytes: u64,
    /// 磁盘读取次数
    pub disk_reads: u64,
    /// 磁盘写入次数
    pub disk_writes: u64,
    /// 网络传输字节数
    pub network_bytes: u64,
    /// 其他统计信息
    pub extra_stats: HashMap<String, String>,
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

    pub fn merge(&mut self, other: &NodeStatistics) {
        self.actual_rows += other.actual_rows;
        self.exec_time_us += other.exec_time_us;
        self.peak_memory_bytes = self.peak_memory_bytes.max(other.peak_memory_bytes);
        self.disk_reads += other.disk_reads;
        self.disk_writes += other.disk_writes;
        self.network_bytes += other.network_bytes;
    }
}

/// 成本模型配置
#[derive(Debug, Clone)]
pub struct CostModelConfig {
    /// CPU 成本系数
    pub cpu_cost_factor: f64,
    /// 内存成本系数
    pub memory_cost_factor: f64,
    /// 磁盘IO成本系数
    pub disk_io_cost_factor: f64,
    /// 网络成本系数
    pub network_cost_factor: f64,
    /// 默认过滤选择性
    pub default_selectivity: f64,
    /// 默认连接选择性
    pub default_join_selectivity: f64,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            cpu_cost_factor: 1.0,
            memory_cost_factor: 0.5,
            disk_io_cost_factor: 10.0,
            network_cost_factor: 5.0,
            default_selectivity: 0.1,
            default_join_selectivity: 0.3,
        }
    }
}

/// 成本估计
#[derive(Debug, Clone, Default)]
pub struct CostEstimate {
    /// CPU 成本
    pub cpu_cost: f64,
    /// 内存成本
    pub memory_cost: f64,
    /// IO 成本
    pub io_cost: f64,
    /// 网络成本
    pub network_cost: f64,
    /// 总成本
    pub total_cost: f64,
    /// 估计输出行数
    pub output_rows: u64,
    /// 估计输出大小（字节）
    pub output_size: u64,
}

impl CostEstimate {
    pub fn new(
        cpu_cost: f64,
        memory_cost: f64,
        io_cost: f64,
        network_cost: f64,
        output_rows: u64,
        output_size: u64,
    ) -> Self {
        let total_cost = cpu_cost + memory_cost + io_cost + network_cost;
        Self {
            cpu_cost,
            memory_cost,
            io_cost,
            network_cost,
            total_cost,
            output_rows,
            output_size,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0, 0)
    }

    pub fn add(&self, other: &CostEstimate) -> Self {
        Self::new(
            self.cpu_cost + other.cpu_cost,
            self.memory_cost + other.memory_cost,
            self.io_cost + other.io_cost,
            self.network_cost + other.network_cost,
            self.output_rows.max(other.output_rows),
            self.output_size.max(other.output_size),
        )
    }

    pub fn multiply(&self, factor: f64) -> Self {
        Self::new(
            self.cpu_cost * factor,
            self.memory_cost * factor,
            self.io_cost * factor,
            self.network_cost * factor,
            (self.output_rows as f64 * factor) as u64,
            (self.output_size as f64 * factor) as u64,
        )
    }
}

/// 选择性估计
#[derive(Debug, Clone)]
pub struct SelectivityEstimate {
    /// 过滤选择性（0.0 - 1.0）
    pub filter_selectivity: f64,
    /// 连接选择性（0.0 - 1.0）
    pub join_selectivity: f64,
    /// 聚合选择性（0.0 - 1.0）
    pub aggregate_selectivity: f64,
    /// 重复因子
    pub duplication_factor: f64,
}

impl Default for SelectivityEstimate {
    fn default() -> Self {
        Self {
            filter_selectivity: 0.1,
            join_selectivity: 0.3,
            aggregate_selectivity: 1.0,
            duplication_factor: 1.0,
        }
    }
}

impl SelectivityEstimate {
    pub fn new(
        filter_selectivity: f64,
        join_selectivity: f64,
        aggregate_selectivity: f64,
        duplication_factor: f64,
    ) -> Self {
        Self {
            filter_selectivity: filter_selectivity.clamp(0.0, 1.0),
            join_selectivity: join_selectivity.clamp(0.0, 1.0),
            aggregate_selectivity: aggregate_selectivity.clamp(0.0, 1.0),
            duplication_factor,
        }
    }

    pub fn for_equality_comparison() -> Self {
        Self::new(0.01, 0.1, 1.0, 1.0)
    }

    pub fn for_range_comparison() -> Self {
        Self::new(0.2, 0.3, 1.0, 1.0)
    }

    pub fn for_like_pattern() -> Self {
        Self::new(0.3, 0.3, 1.0, 1.0)
    }
}
