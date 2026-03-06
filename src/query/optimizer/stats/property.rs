//! 属性统计信息模块
//!
//! 提供属性级别的统计信息，用于查询优化器估算选择性

/// 属性统计信息
#[derive(Debug, Clone)]
pub struct PropertyStatistics {
    /// 属性名称
    pub property_name: String,
    /// 所属标签（可选）
    pub tag_name: Option<String>,
    /// 不同值数量
    pub distinct_values: u64,
}

impl PropertyStatistics {
    /// 创建新的属性统计信息
    pub fn new(property_name: String, tag_name: Option<String>) -> Self {
        Self {
            property_name,
            tag_name,
            distinct_values: 0,
        }
    }
}

impl Default for PropertyStatistics {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}
