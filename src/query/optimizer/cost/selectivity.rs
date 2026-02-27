//! 选择性估计器模块
//!
//! 用于估算查询条件的选择性

use std::sync::Arc;

use crate::query::optimizer::stats::StatisticsManager;
use crate::core::types::Expression;
use crate::core::types::BinaryOperator;

/// 选择性估计器
#[derive(Debug)]
pub struct SelectivityEstimator {
    stats_manager: Arc<StatisticsManager>,
}

impl SelectivityEstimator {
    /// 创建新的选择性估计器
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self { stats_manager }
    }

    /// 估计等值条件选择性
    pub fn estimate_equality_selectivity(
        &self,
        tag_name: Option<&str>,
        property_name: &str,
    ) -> f64 {
        let stats = self.stats_manager.get_property_stats(tag_name, property_name);

        match stats {
            Some(s) if s.distinct_values > 0 => {
                1.0 / s.distinct_values as f64
            }
            _ => 0.1,
        }
    }

    /// 估计范围条件选择性
    pub fn estimate_range_selectivity(&self) -> f64 {
        0.333
    }

    /// 估计小于条件选择性
    pub fn estimate_less_than_selectivity(&self) -> f64 {
        0.333
    }

    /// 估计大于条件选择性
    pub fn estimate_greater_than_selectivity(&self) -> f64 {
        0.333
    }

    /// 估计 LIKE 条件选择性
    pub fn estimate_like_selectivity(&self, pattern: &str) -> f64 {
        // 根据模式复杂度调整选择性
        if pattern.starts_with('%') && pattern.ends_with('%') {
            // %xxx% 模式选择性较低
            0.5
        } else if pattern.ends_with('%') {
            // xxx% 模式选择性较高
            0.1
        } else if pattern.starts_with('%') {
            // %xxx 模式选择性中等
            0.2
        } else {
            // 精确匹配
            0.05
        }
    }

    /// 从表达式估计选择性
    pub fn estimate_from_expression(
        &self,
        expr: &Expression,
        tag_name: Option<&str>,
    ) -> f64 {
        // 根据表达式类型估计选择性
        match expr {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOperator::Equal => {
                        // 尝试从表达式中提取属性名
                        if let Expression::Property { property, .. } = left.as_ref() {
                            self.estimate_equality_selectivity(tag_name, property)
                        } else if let Expression::Property { property, .. } = right.as_ref() {
                            self.estimate_equality_selectivity(tag_name, property)
                        } else {
                            0.1
                        }
                    }
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual => {
                        self.estimate_less_than_selectivity()
                    }
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual => {
                        self.estimate_greater_than_selectivity()
                    }
                    BinaryOperator::And => {
                        let left_sel = self.estimate_from_expression(left, tag_name);
                        let right_sel = self.estimate_from_expression(right, tag_name);
                        left_sel * right_sel
                    }
                    BinaryOperator::Or => {
                        let left_sel = self.estimate_from_expression(left, tag_name);
                        let right_sel = self.estimate_from_expression(right, tag_name);
                        left_sel + right_sel - left_sel * right_sel
                    }
                    _ => 0.1,
                }
            }
            Expression::Function { name, args } => {
                if name.eq_ignore_ascii_case("like") && args.len() >= 2 {
                    // 尝试提取 LIKE 模式
                    if let Expression::Literal(value) = &args[1] {
                        if let crate::core::value::Value::String(pattern) = value {
                            return self.estimate_like_selectivity(pattern);
                        }
                    }
                }
                0.1
            }
            _ => 0.1,
        }
    }
}

impl Clone for SelectivityEstimator {
    fn clone(&self) -> Self {
        Self {
            stats_manager: self.stats_manager.clone(),
        }
    }
}
