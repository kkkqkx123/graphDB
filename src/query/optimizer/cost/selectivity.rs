//! 选择性估计器模块
//!
//! 用于估算查询条件的选择性

use std::sync::Arc;

use crate::query::optimizer::stats::StatisticsManager;
use crate::core::types::Expression;
use crate::core::types::BinaryOperator;

/// 选择性估计器
///
/// 提供基于统计信息和启发式规则的选择性估计
#[derive(Debug)]
pub struct SelectivityEstimator {
    stats_manager: Arc<StatisticsManager>,
}

/// 默认选择性常量
pub mod defaults {
    /// 等值查询默认选择性（假设10个不同值）
    pub const EQUALITY: f64 = 0.1;
    /// 范围查询默认选择性（假设选择1/3的数据）
    pub const RANGE: f64 = 0.333;
    /// 小于/大于查询默认选择性
    pub const COMPARISON: f64 = 0.333;
    /// 不等查询默认选择性
    pub const NOT_EQUAL: f64 = 0.9;
    /// IS NULL 查询选择性（通常很少为null）
    pub const IS_NULL: f64 = 0.05;
    /// IS NOT NULL 查询选择性
    pub const IS_NOT_NULL: f64 = 0.95;
    /// IN 查询默认选择性（假设3个值）
    pub const IN_LIST: f64 = 0.3;
    /// EXISTS 查询选择性
    pub const EXISTS: f64 = 0.5;
    /// 布尔AND操作的选择性惩罚
    pub const AND_CORRELATION: f64 = 0.9;
    /// 布尔OR操作的选择性惩罚
    pub const OR_CORRELATION: f64 = 0.9;
}

impl SelectivityEstimator {
    /// 创建新的选择性估计器
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self { stats_manager }
    }

    /// 估计等值条件选择性
    ///
    /// 如果有统计信息，使用 1/不同值数量；
    /// 否则使用默认值 0.1
    pub fn estimate_equality_selectivity(
        &self,
        tag_name: Option<&str>,
        property_name: &str,
    ) -> f64 {
        let stats = self.stats_manager.get_property_stats(tag_name, property_name);

        match stats {
            Some(s) if s.distinct_values > 0 => {
                (1.0 / s.distinct_values as f64).min(1.0)
            }
            _ => defaults::EQUALITY,
        }
    }

    /// 估计范围条件选择性
    ///
    /// 如果有统计信息，基于直方图计算；
    /// 否则使用默认值 1/3
    pub fn estimate_range_selectivity(&self) -> f64 {
        defaults::RANGE
    }

    /// 估计范围条件选择性（带边界值）
    ///
    /// 根据范围大小调整选择性
    pub fn estimate_range_selectivity_with_bounds(&self, min_val: f64, max_val: f64, range_size: f64) -> f64 {
        if max_val <= min_val {
            return defaults::RANGE;
        }
        let total_range = max_val - min_val;
        let selectivity = (range_size / total_range).min(1.0).max(0.001);
        // 范围查询通常不会选择太多数据，添加一个上限
        selectivity.min(0.8)
    }

    /// 估计小于条件选择性
    ///
    /// 如果有统计信息，基于直方图计算；
    /// 否则假设数据均匀分布，返回 1/3
    pub fn estimate_less_than_selectivity(&self, value: Option<f64>) -> f64 {
        // 如果有具体值，可以尝试根据值的分布调整
        // 这里使用简单的启发式：假设数据均匀分布
        match value {
            Some(v) if v < 0.0 => 0.1, // 负值通常较少
            Some(v) if v == 0.0 => 0.05, // 零值通常很少
            _ => defaults::COMPARISON,
        }
    }

    /// 估计大于条件选择性
    pub fn estimate_greater_than_selectivity(&self, value: Option<f64>) -> f64 {
        match value {
            Some(v) if v < 0.0 => 0.9, // 大于负值通常选择大部分数据
            Some(v) if v == 0.0 => 0.95, // 大于零通常选择大部分数据
            _ => defaults::COMPARISON,
        }
    }

    /// 估计 LIKE 条件选择性
    ///
    /// 根据模式的前缀和后缀通配符调整选择性：
    /// - prefix%：选择性较高（约0.1）
    /// - %suffix：选择性中等（约0.2）
    /// - %substring%：选择性较低（约0.5）
    /// - 无通配符：精确匹配（约0.05）
    pub fn estimate_like_selectivity(&self, pattern: &str) -> f64 {
        let has_prefix = pattern.starts_with('%');
        let has_suffix = pattern.ends_with('%');
        let middle_wildcards = pattern.matches('%').count()
            + pattern.matches('_').count();

        match (has_prefix, has_suffix) {
            (true, true) => {
                // %xxx% 模式选择性很低
                0.5_f64.min(0.1 + middle_wildcards as f64 * 0.1)
            }
            (false, true) => {
                // xxx% 前缀匹配选择性较高
                0.1_f64.min(0.05 + middle_wildcards as f64 * 0.02)
            }
            (true, false) => {
                // %xxx 后缀匹配选择性中等
                0.2_f64.min(0.1 + middle_wildcards as f64 * 0.05)
            }
            (false, false) => {
                // 无通配符，接近精确匹配
                0.05
            }
        }
    }

    /// 估计 IN 列表选择性
    ///
    /// 假设每个值的选择性相同，总选择性 = 值数量 * 单个值选择性
    pub fn estimate_in_selectivity(&self, list_size: usize) -> f64 {
        let single_selectivity = defaults::EQUALITY;
        (list_size as f64 * single_selectivity).min(0.9)
    }

    /// 估计 IS NULL 选择性
    pub fn estimate_is_null_selectivity(&self) -> f64 {
        defaults::IS_NULL
    }

    /// 估计 IS NOT NULL 选择性
    pub fn estimate_is_not_null_selectivity(&self) -> f64 {
        defaults::IS_NOT_NULL
    }

    /// 估计 NOT 条件选择性
    ///
    /// NOT 条件的选择性 = 1 - 原条件选择性
    pub fn estimate_not_selectivity(&self, inner_selectivity: f64) -> f64 {
        (1.0 - inner_selectivity).min(0.99).max(0.01)
    }

    /// 从表达式估计选择性
    ///
    /// 这是主要的入口方法，根据表达式类型分发到具体的估计方法
    pub fn estimate_from_expression(
        &self,
        expr: &Expression,
        tag_name: Option<&str>,
    ) -> f64 {
        match expr {
            Expression::Binary { op, left, right } => {
                self.estimate_binary_expression(op, left, right, tag_name)
            }
            Expression::Unary { op, operand } => {
                self.estimate_unary_expression(op, operand, tag_name)
            }
            Expression::Function { name, args } => {
                self.estimate_function_expression(name, args)
            }
            Expression::Literal(_) => {
                // 字面量条件的选择性取决于值，通常认为是高选择性
                0.1
            }
            Expression::Property { .. } => {
                // 属性本身作为条件（如 WHERE n.active）
                // 假设布尔属性大约一半为真
                0.5
            }
            _ => defaults::EQUALITY,
        }
    }

    /// 估计二元表达式选择性
    fn estimate_binary_expression(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
        tag_name: Option<&str>,
    ) -> f64 {
        match op {
            BinaryOperator::Equal => {
                // 尝试从表达式中提取属性名和值
                let property_name = self.extract_property_name(left)
                    .or_else(|| self.extract_property_name(right));

                if let Some(prop) = property_name {
                    self.estimate_equality_selectivity(tag_name, &prop)
                } else {
                    defaults::EQUALITY
                }
            }
            BinaryOperator::NotEqual => {
                // 不等查询通常选择大部分数据
                defaults::NOT_EQUAL
            }
            BinaryOperator::LessThan => {
                let value = self.extract_numeric_value(right);
                self.estimate_less_than_selectivity(value)
            }
            BinaryOperator::LessThanOrEqual => {
                let value = self.extract_numeric_value(right);
                self.estimate_less_than_selectivity(value)
                    .min(0.9)
                    .max(0.01)
            }
            BinaryOperator::GreaterThan => {
                let value = self.extract_numeric_value(right);
                self.estimate_greater_than_selectivity(value)
            }
            BinaryOperator::GreaterThanOrEqual => {
                let value = self.extract_numeric_value(right);
                self.estimate_greater_than_selectivity(value)
                    .min(0.9)
                    .max(0.01)
            }
            BinaryOperator::And => {
                let left_sel = self.estimate_from_expression(left, tag_name);
                let right_sel = self.estimate_from_expression(right, tag_name);
                // AND 的选择性通常比乘积略高（因为条件间可能有相关性）
                (left_sel * right_sel / defaults::AND_CORRELATION).min(1.0)
            }
            BinaryOperator::Or => {
                let left_sel = self.estimate_from_expression(left, tag_name);
                let right_sel = self.estimate_from_expression(right, tag_name);
                // OR 的选择性：P(A or B) = P(A) + P(B) - P(A and B)
                let combined = left_sel + right_sel - left_sel * right_sel * defaults::OR_CORRELATION;
                combined.min(0.99).max(0.01)
            }
            BinaryOperator::In => {
                // 估计 IN 列表的大小
                let list_size = self.estimate_list_size(right);
                self.estimate_in_selectivity(list_size)
            }
            _ => defaults::EQUALITY,
        }
    }

    /// 估计一元表达式选择性
    fn estimate_unary_expression(
        &self,
        op: &crate::core::types::UnaryOperator,
        expr: &Expression,
        tag_name: Option<&str>,
    ) -> f64 {
        use crate::core::types::UnaryOperator;

        match op {
            UnaryOperator::Not => {
                let inner = self.estimate_from_expression(expr, tag_name);
                self.estimate_not_selectivity(inner)
            }
            UnaryOperator::IsNull => {
                defaults::IS_NULL
            }
            UnaryOperator::IsNotNull => {
                defaults::IS_NOT_NULL
            }
            _ => defaults::EQUALITY,
        }
    }

    /// 估计函数表达式选择性
    fn estimate_function_expression(
        &self,
        name: &str,
        args: &[Expression],
    ) -> f64 {
        let name_lower = name.to_lowercase();

        match name_lower.as_str() {
            "like" | "ilike" if args.len() >= 2 => {
                // 尝试提取 LIKE 模式
                if let Expression::Literal(value) = &args[1] {
                    if let crate::core::value::Value::String(pattern) = value {
                        return self.estimate_like_selectivity(pattern);
                    }
                }
                defaults::EQUALITY
            }
            "exists" => defaults::EXISTS,
            "contains" | "has" => 0.2, // 包含查询通常选择性较高
            "starts_with" => 0.1, // 前缀匹配
            "ends_with" => 0.2, // 后缀匹配
            "in" => {
                let list_size = args.len().saturating_sub(1);
                self.estimate_in_selectivity(list_size)
            }
            _ => defaults::EQUALITY,
        }
    }

    /// 从表达式中提取属性名
    fn extract_property_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Property { property, .. } => Some(property.clone()),
            _ => None,
        }
    }

    /// 从表达式中提取数值
    fn extract_numeric_value(&self, expr: &Expression) -> Option<f64> {
        match expr {
            Expression::Literal(value) => {
                match value {
                    crate::core::value::Value::Int(i) => Some(*i as f64),
                    crate::core::value::Value::Float(f) => Some(*f),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 估计列表大小
    fn estimate_list_size(&self, expr: &Expression) -> usize {
        match expr {
            Expression::List(items) => items.len(),
            _ => 3, // 默认假设3个元素
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
