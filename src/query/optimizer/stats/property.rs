//! 属性统计信息模块
//!
//! 提供属性级别的统计信息，用于查询优化器估算选择性

use std::time::SystemTime;
use crate::core::value::Value;

/// 属性统计信息
#[derive(Debug, Clone)]
pub struct PropertyStatistics {
    /// 属性名称
    pub property_name: String,
    /// 所属标签（可选）
    pub tag_name: Option<String>,
    /// 不同值数量
    pub distinct_values: u64,
    /// 空值比例
    pub null_fraction: f64,
    /// 最小值
    pub min_value: Option<Value>,
    /// 最大值
    pub max_value: Option<Value>,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl PropertyStatistics {
    /// 创建新的属性统计信息
    pub fn new(property_name: String, tag_name: Option<String>) -> Self {
        Self {
            property_name,
            tag_name,
            distinct_values: 0,
            null_fraction: 0.0,
            min_value: None,
            max_value: None,
            last_analyzed: SystemTime::now(),
        }
    }

    /// 估算等值条件选择性
    pub fn estimate_equality_selectivity(&self) -> f64 {
        if self.distinct_values == 0 {
            0.1
        } else {
            1.0 / self.distinct_values as f64
        }
    }
}

impl Default for PropertyStatistics {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}
