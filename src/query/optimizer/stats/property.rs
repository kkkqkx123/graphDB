//! 属性统计信息模块
//!
//! 提供属性级别的统计信息，用于查询优化器估算选择性

use super::histogram::Histogram;

/// 属性统计信息
#[derive(Debug, Clone)]
pub struct PropertyStatistics {
    /// 属性名称
    pub property_name: String,
    /// 所属标签（可选）
    pub tag_name: Option<String>,
    /// 不同值数量
    pub distinct_values: u64,
    /// 可选的直方图（高基数属性启用）
    pub histogram: Option<Histogram>,
    /// 是否适合使用直方图（低基数属性不需要）
    pub use_histogram: bool,
}

impl PropertyStatistics {
    /// 创建新的属性统计信息
    pub fn new(property_name: String, tag_name: Option<String>) -> Self {
        Self {
            property_name,
            tag_name,
            distinct_values: 0,
            histogram: None,
            use_histogram: false,
        }
    }

    /// 设置直方图
    pub fn with_histogram(mut self, histogram: Histogram) -> Self {
        self.histogram = Some(histogram);
        self.use_histogram = true;
        self
    }

    /// 判断是否使用直方图
    pub fn should_use_histogram(&self) -> bool {
        self.use_histogram && self.histogram.is_some()
    }
}

impl Default for PropertyStatistics {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}
