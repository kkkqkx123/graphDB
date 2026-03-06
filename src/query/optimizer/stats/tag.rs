//! 标签统计信息模块
//!
//! 提供标签级别的统计信息，用于查询优化器估算代价

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
}

impl TagStatistics {
    /// 创建新的标签统计信息
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_name,
            vertex_count: 0,
            avg_out_degree: 0.0,
            avg_in_degree: 0.0,
        }
    }
}

impl Default for TagStatistics {
    fn default() -> Self {
        Self::new(String::new())
    }
}
