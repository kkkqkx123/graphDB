//! 选择性估计模块
//!
//! 提供查询条件的选择性估算功能
//! 参考 PostgreSQL 的选择性估计算法

use crate::core::Value;
use super::statistics::{ColumnStatistics, GraphStatistics, TableStatistics};

/// 范围条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOp {
    LessThan,      // <
    LessEqual,     // <=
    GreaterThan,   // >
    GreaterEqual,  // >=
    Between,       // BETWEEN
}

/// 布尔操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    And,
    Or,
    Not,
}

/// 连接类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// 选择性估计器
pub struct SelectivityEstimator;

impl SelectivityEstimator {
    /// 创建新的选择性估计器
    pub fn new() -> Self {
        Self
    }

    /// 估计等值条件的选择性
    /// 
    /// 算法：
    /// 1. 如果值在 MCV 列表中，返回对应的频率
    /// 2. 否则使用非 MCV 均匀分布假设
    /// 
    /// # 参数
    /// - `column_stats`: 列统计信息
    /// - `value`: 查询值
    /// 
    /// # 返回
    /// 选择性值 (0.0 - 1.0)
    pub fn estimate_equal(&self, column_stats: &ColumnStatistics, value: &Value) -> f64 {
        // 1. 检查是否在 MCV 中
        if let Some((_, freq)) = column_stats.most_common_values.iter().find(|(v, _)| v == value) {
            return *freq;
        }

        // 2. 使用非 MCV 均匀分布假设
        let mcv_total_freq = column_stats.mcv_total_frequency();
        let non_mcv_distinct = column_stats.non_mcv_distinct_count();

        if non_mcv_distinct > 0 {
            let selectivity = (1.0 - mcv_total_freq) / non_mcv_distinct as f64;
            // 限制最大选择性（避免对高基数的列过度估计）
            selectivity.min(0.5)
        } else {
            // 没有统计信息时的默认估计
            0.1
        }
    }

    /// 估计不等条件的选择性 (!=)
    pub fn estimate_not_equal(&self, column_stats: &ColumnStatistics, value: &Value) -> f64 {
        let equal_sel = self.estimate_equal(column_stats, value);
        (1.0 - equal_sel).max(0.0)
    }

    /// 估计范围条件的选择性
    /// 
    /// 使用直方图进行估计，如果没有直方图则使用启发式估计
    pub fn estimate_range(&self, column_stats: &ColumnStatistics, op: RangeOp, value: &Value) -> f64 {
        if column_stats.histogram_bounds.is_empty() {
            // 无直方图，使用启发式估计
            return self.heuristic_range_selectivity(op);
        }

        // 使用直方图计算
        match op {
            RangeOp::LessThan | RangeOp::LessEqual => {
                self.histogram_less_than(column_stats, value)
            }
            RangeOp::GreaterThan | RangeOp::GreaterEqual => {
                let less_sel = self.histogram_less_than(column_stats, value);
                (1.0 - less_sel).max(0.0)
            }
            RangeOp::Between => {
                // BETWEEN 需要两个值，这里简化处理
                0.1
            }
        }
    }

    /// 估计 IN 条件的选择性
    pub fn estimate_in(&self, column_stats: &ColumnStatistics, values: &[Value]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        // 计算每个值的选择性并累加
        let total_selectivity: f64 = values.iter()
            .map(|v| self.estimate_equal(column_stats, v))
            .sum();

        // 限制最大选择性不超过 1.0
        total_selectivity.min(1.0)
    }

    /// 估计 IS NULL 条件的选择性
    pub fn estimate_is_null(&self, column_stats: &ColumnStatistics) -> f64 {
        column_stats.null_fraction
    }

    /// 估计 IS NOT NULL 条件的选择性
    pub fn estimate_is_not_null(&self, column_stats: &ColumnStatistics) -> f64 {
        1.0 - column_stats.null_fraction
    }

    /// 估计布尔复合条件的选择性
    /// 
    /// 假设条件之间是独立的（简化处理）
    pub fn estimate_boolean(&self, op: BooleanOp, left_sel: f64, right_sel: Option<f64>) -> f64 {
        let left = left_sel.clamp(0.0, 1.0);
        
        match op {
            BooleanOp::And => {
                let right = right_sel.unwrap_or(1.0).clamp(0.0, 1.0);
                // 假设独立性: P(A∧B) = P(A) × P(B)
                left * right
            }
            BooleanOp::Or => {
                let right = right_sel.unwrap_or(0.0).clamp(0.0, 1.0);
                // P(A∨B) = P(A) + P(B) - P(A∧B)
                (left + right - left * right).min(1.0)
            }
            BooleanOp::Not => {
                (1.0 - left).max(0.0)
            }
        }
    }

    /// 估计连接选择性
    /// 
    /// 对于等值连接，使用公式：1 / max(左不同值, 右不同值)
    pub fn estimate_join(
        &self,
        left_stats: &ColumnStatistics,
        right_stats: &ColumnStatistics,
        join_type: JoinType,
    ) -> f64 {
        match join_type {
            JoinType::Inner => {
                // 等值连接基础选择性
                let distinct_left = left_stats.distinct_count.max(1);
                let distinct_right = right_stats.distinct_count.max(1);
                let base_selectivity = 1.0 / distinct_left.max(distinct_right) as f64;

                // 考虑空值调整
                let null_adjustment = (1.0 - left_stats.null_fraction)
                    * (1.0 - right_stats.null_fraction);

                base_selectivity * null_adjustment
            }
            JoinType::Left => {
                // 左连接保留所有左表行
                1.0
            }
            JoinType::Right => {
                // 右连接保留所有右表行
                1.0
            }
            JoinType::Full => {
                // 全连接，使用启发式估计
                1.0
            }
        }
    }

    /// 估计图遍历选择性
    /// 
    /// 根据度分布和边类型估计遍历结果集大小
    pub fn estimate_traversal(
        &self,
        graph_stats: &GraphStatistics,
        from_tag: &str,
        edge_type: Option<&str>,
        steps: u32,
    ) -> f64 {
        let from_count = graph_stats.get_tag_count(from_tag) as f64;
        if from_count == 0.0 {
            return 0.0;
        }

        // 估计每步遍历的平均扩展数
        let avg_expansion = match edge_type {
            Some(et) => {
                let edge_count = graph_stats.get_edge_type_count(et) as f64;
                if from_count > 0.0 {
                    edge_count / from_count
                } else {
                    graph_stats.avg_out_degree
                }
            }
            None => graph_stats.avg_out_degree,
        };

        // 计算多步遍历后的估计行数
        let mut total_rows = from_count;
        for _ in 0..steps {
            total_rows *= avg_expansion;
        }

        // 选择性 = 扩展后的行数 / 起始行数
        // 限制最大值为 1.0
        (total_rows / from_count).min(1.0)
    }

    /// 估计模式匹配条件的选择性 (LIKE, REGEXP 等)
    /// 
    /// 使用启发式方法，基于模式串的特征
    pub fn estimate_pattern_match(&self, pattern: &str, is_prefix: bool) -> f64 {
        if is_prefix {
            // 前缀匹配（如 'abc%'）
            // 估计选择性为 10%
            0.1
        } else if pattern.contains('%') || pattern.contains('_') {
            // 包含通配符的 LIKE
            // 估计选择性为 5%
            0.05
        } else {
            // 精确匹配
            0.01
        }
    }

    // ============ 私有辅助方法 ============

    /// 启发式范围选择性估计
    /// 
    /// 当没有直方图时使用
    fn heuristic_range_selectivity(&self, op: RangeOp) -> f64 {
        match op {
            RangeOp::LessThan | RangeOp::LessEqual => 0.3,
            RangeOp::GreaterThan | RangeOp::GreaterEqual => 0.3,
            RangeOp::Between => 0.2,
        }
    }

    /// 使用直方图估计小于某值的选择性
    fn histogram_less_than(&self, column_stats: &ColumnStatistics, value: &Value) -> f64 {
        let bounds = &column_stats.histogram_bounds;
        if bounds.is_empty() {
            return 0.3;
        }

        let num_buckets = bounds.len() - 1;
        if num_buckets == 0 {
            return 0.5;
        }

        // 找到值所在的桶
        let mut bucket_idx = 0;
        for (i, bound) in bounds.iter().enumerate() {
            if value < bound {
                bucket_idx = i;
                break;
            }
            bucket_idx = i + 1;
        }

        if bucket_idx == 0 {
            // 值小于第一个边界
            return 0.0;
        }

        if bucket_idx >= num_buckets {
            // 值大于等于最后一个边界
            return 1.0;
        }

        // 计算桶内的比例（线性插值）
        let prev_bound = &bounds[bucket_idx - 1];
        let next_bound = &bounds[bucket_idx];
        
        let fraction = self.value_fraction_between(value, prev_bound, next_bound);
        
        // 选择性 = (完整桶数 + 当前桶比例) / 总桶数
        let selectivity = (bucket_idx - 1) as f64 + fraction;
        selectivity / num_buckets as f64
    }

    /// 计算值在两个边界之间的比例
    fn value_fraction_between(&self, value: &Value, low: &Value, high: &Value) -> f64 {
        // 根据 Value 类型进行比较
        match (value, low, high) {
            (Value::Int(v), Value::Int(l), Value::Int(h)) => {
                if h > l {
                    ((v - l) as f64 / (h - l) as f64).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            }
            (Value::Float(v), Value::Float(l), Value::Float(h)) => {
                if h > l {
                    ((v - l) / (h - l)).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            }
            _ => 0.5, // 默认中间值
        }
    }
}

impl Default for SelectivityEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// 表级选择性估计
/// 
/// 用于估计对整个表应用过滤条件后的行数
pub struct TableSelectivityEstimator;

impl TableSelectivityEstimator {
    /// 估计过滤后的行数
    pub fn estimate_filtered_rows(
        table_stats: &TableStatistics,
        selectivity: f64,
    ) -> u64 {
        (table_stats.row_count as f64 * selectivity.clamp(0.0, 1.0)) as u64
    }

    /// 估计两个表连接后的行数
    pub fn estimate_join_rows(
        left_stats: &TableStatistics,
        right_stats: &TableStatistics,
        join_selectivity: f64,
    ) -> u64 {
        let left_rows = left_stats.row_count as f64;
        let right_rows = right_stats.row_count as f64;
        
        // 笛卡尔积 × 连接选择性
        (left_rows * right_rows * join_selectivity.clamp(0.0, 1.0)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_equal_with_mcv() {
        let mut col_stats = ColumnStatistics::new("test_col");
        col_stats.most_common_values = vec![
            (Value::Int(1), 0.3),
            (Value::Int(2), 0.2),
        ];
        col_stats.distinct_count = 10;

        let estimator = SelectivityEstimator::new();
        
        // 值在 MCV 中
        assert!((estimator.estimate_equal(&col_stats, &Value::Int(1)) - 0.3).abs() < 0.001);
        assert!((estimator.estimate_equal(&col_stats, &Value::Int(2)) - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_estimate_equal_without_mcv() {
        let mut col_stats = ColumnStatistics::new("test_col");
        col_stats.most_common_values = vec![
            (Value::Int(1), 0.3),
            (Value::Int(2), 0.2),
        ];
        col_stats.distinct_count = 10;

        let estimator = SelectivityEstimator::new();
        
        // 值不在 MCV 中，使用均匀分布假设
        // (1 - 0.5) / (10 - 2) = 0.0625
        let sel = estimator.estimate_equal(&col_stats, &Value::Int(99));
        assert!(sel > 0.0 && sel < 0.5);
    }

    #[test]
    fn test_estimate_boolean() {
        let estimator = SelectivityEstimator::new();

        // AND
        let and_sel = estimator.estimate_boolean(BooleanOp::And, 0.5, Some(0.3));
        assert!((and_sel - 0.15).abs() < 0.001);

        // OR
        let or_sel = estimator.estimate_boolean(BooleanOp::Or, 0.5, Some(0.3));
        assert!((or_sel - 0.65).abs() < 0.001);

        // NOT
        let not_sel = estimator.estimate_boolean(BooleanOp::Not, 0.3, None);
        assert!((not_sel - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_estimate_join() {
        let mut left_stats = ColumnStatistics::new("left_col");
        left_stats.distinct_count = 100;
        left_stats.null_fraction = 0.0;

        let mut right_stats = ColumnStatistics::new("right_col");
        right_stats.distinct_count = 50;
        right_stats.null_fraction = 0.0;

        let estimator = SelectivityEstimator::new();
        let join_sel = estimator.estimate_join(&left_stats, &right_stats, JoinType::Inner);
        
        // 1 / max(100, 50) = 0.01
        assert!((join_sel - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_estimate_traversal() {
        let mut graph_stats = GraphStatistics::default();
        graph_stats.tag_counts.insert("Person".to_string(), 1000);
        graph_stats.edge_type_counts.insert("KNOWS".to_string(), 5000);
        graph_stats.avg_out_degree = 5.0;

        let estimator = SelectivityEstimator::new();
        let traversal_sel = estimator.estimate_traversal(&graph_stats, "Person", Some("KNOWS"), 1);
        
        // 5000 / 1000 = 5.0, min(5.0, 1.0) = 1.0
        assert_eq!(traversal_sel, 1.0);
    }
}
