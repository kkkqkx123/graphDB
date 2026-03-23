//! 边类型统计信息模块
//!
//! 提供边类型级别的统计信息，用于查询优化器估算遍历代价

/// 热点顶点信息
#[derive(Debug, Clone)]
pub struct HotVertexInfo {
    /// 顶点ID
    pub vertex_id: i64,
    /// 出度
    pub out_degree: u64,
    /// 入度
    pub in_degree: u64,
}

/// 倾斜度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkewnessLevel {
    /// 无倾斜
    None,
    /// 轻度倾斜
    Mild,
    /// 中度倾斜
    Moderate,
    /// 严重倾斜
    Severe,
}

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
    /// 出度标准差（衡量分布离散程度）
    pub out_degree_std_dev: f64,
    /// 入度标准差
    pub in_degree_std_dev: f64,
    /// 基尼系数（0-1，越大越倾斜）
    pub degree_gini_coefficient: f64,
    /// 热点顶点列表（Top-K高度数顶点）
    pub hot_vertices: Vec<HotVertexInfo>,
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
            out_degree_std_dev: 0.0,
            in_degree_std_dev: 0.0,
            degree_gini_coefficient: 0.0,
            hot_vertices: Vec::new(),
        }
    }

    /// 估算扩展代价
    pub fn estimate_expand_cost(&self, start_nodes: u64) -> f64 {
        start_nodes as f64 * self.avg_out_degree
    }

    /// 判断是否存在严重倾斜
    pub fn is_heavily_skewed(&self) -> bool {
        // 基尼系数 > 0.5 认为存在严重倾斜
        self.degree_gini_coefficient > 0.5
            || self.max_out_degree as f64 > self.avg_out_degree * 10.0
    }

    /// 获取倾斜度等级
    pub fn skewness_level(&self) -> SkewnessLevel {
        match self.degree_gini_coefficient {
            g if g > 0.7 => SkewnessLevel::Severe,
            g if g > 0.5 => SkewnessLevel::Moderate,
            g if g > 0.3 => SkewnessLevel::Mild,
            _ => SkewnessLevel::None,
        }
    }

    /// 计算倾斜感知代价（对倾斜数据使用更保守的估计）
    pub fn calculate_skewed_expand_cost(&self, start_nodes: u64) -> f64 {
        let base_cost = self.estimate_expand_cost(start_nodes);

        // 根据倾斜度增加惩罚
        let penalty = match self.skewness_level() {
            SkewnessLevel::Severe => 2.0,
            SkewnessLevel::Moderate => 1.5,
            SkewnessLevel::Mild => 1.2,
            SkewnessLevel::None => 1.0,
        };

        base_cost * penalty
    }

    /// 判断是否包含热点顶点
    pub fn has_hot_vertices(&self) -> bool {
        !self.hot_vertices.is_empty()
    }

    /// 获取热点顶点数量
    pub fn hot_vertex_count(&self) -> usize {
        self.hot_vertices.len()
    }
}

impl Default for EdgeTypeStatistics {
    fn default() -> Self {
        Self::new(String::new())
    }
}
