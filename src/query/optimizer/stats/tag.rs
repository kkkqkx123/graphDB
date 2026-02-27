//! 标签统计信息模块
//!
//! 提供标签级别的统计信息，用于查询优化器估算代价

use std::time::SystemTime;

/// 标签统计信息
#[derive(Debug, Clone)]
pub struct TagStatistics {
    /// 标签名称
    pub tag_name: String,
    /// 顶点数量
    pub vertex_count: u64,
    /// 平均出度（关键指标：影响遍历代价）
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 平均顶点大小（字节）
    pub avg_vertex_size: usize,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl TagStatistics {
    /// 创建新的标签统计信息
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_name,
            vertex_count: 0,
            avg_out_degree: 0.0,
            avg_in_degree: 0.0,
            avg_vertex_size: 0,
            last_analyzed: SystemTime::now(),
        }
    }

    /// 估算遍历代价
    pub fn estimate_traversal_cost(&self, start_nodes: u64, steps: u32) -> f64 {
        let degree = (self.avg_out_degree + self.avg_in_degree) / 2.0;
        start_nodes as f64 * degree.powi(steps as i32)
    }
}

impl Default for TagStatistics {
    fn default() -> Self {
        Self::new(String::new())
    }
}
