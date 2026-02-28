//! 边类型统计信息模块
//!
//! 提供边类型级别的统计信息，用于查询优化器估算遍历代价

use std::time::SystemTime;

/// 边类型统计信息
#[derive(Debug, Clone)]
pub struct EdgeTypeStatistics {
    /// 边类型名称
    pub edge_type: String,
    /// 边总数
    pub edge_count: u64,
    /// 平均出度
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 最大出度
    pub max_out_degree: u64,
    /// 最大入度
    pub max_in_degree: u64,
    /// 唯一源顶点数
    pub unique_src_vertices: u64,
    /// 唯一目标顶点数
    pub unique_dst_vertices: u64,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl EdgeTypeStatistics {
    /// 创建新的边类型统计信息
    pub fn new(edge_type: String) -> Self {
        Self {
            edge_type,
            edge_count: 0,
            avg_out_degree: 0.0,
            avg_in_degree: 0.0,
            max_out_degree: 0,
            max_in_degree: 0,
            unique_src_vertices: 0,
            unique_dst_vertices: 0,
            last_analyzed: SystemTime::now(),
        }
    }

    /// 估算扩展代价
    pub fn estimate_expand_cost(&self, start_nodes: u64) -> f64 {
        start_nodes as f64 * self.avg_out_degree
    }
}

impl Default for EdgeTypeStatistics {
    fn default() -> Self {
        Self::new(String::new())
    }
}
