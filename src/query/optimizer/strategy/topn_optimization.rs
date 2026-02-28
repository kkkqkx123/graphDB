//! 排序消除优化器模块
//!
//! **基于代价的排序优化策略**，专注于 Sort + Limit 到 TopN 的转换决策。
//!
//! ## 注意
//!
//! 此模块**仅包含基于代价的优化逻辑**。
//!
//! 启发式排序消除（如索引有序性消除）已在 rewrite 阶段通过 `EliminateSortRule` 处理。
//! 这种分层设计确保：
//! 1. 简单确定的优化（索引匹配）在 rewrite 阶段尽早执行
//! 2. 复杂的代价决策（TopN 转换）在物理优化阶段执行
//!
//! ## 优化策略
//!
//! - 将 Sort + Limit 转换为 TopN（基于代价比较）
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::SortEliminationOptimizer;
//! use graphdb::query::optimizer::cost::CostCalculator;
//! use std::sync::Arc;
//!
//! let optimizer = SortEliminationOptimizer::new(cost_calculator);
//! let decision = optimizer.optimize_sort(&sort_context);
//! ```

use std::sync::Arc;

use crate::query::optimizer::cost::CostCalculator;
use crate::query::planner::plan::core::nodes::{SortItem, SortNode};

/// 排序消除决策
#[derive(Debug, Clone, PartialEq)]
pub enum SortEliminationDecision {
    /// 保留排序（无法消除或转换）
    KeepSort {
        /// 保留原因
        reason: SortKeepReason,
        /// 估计代价
        estimated_cost: f64,
    },
    /// 转换为 TopN
    ConvertToTopN {
        /// 转换原因
        reason: TopNConversionReason,
        /// TopN 估计代价
        topn_cost: f64,
        /// 原排序代价
        original_cost: f64,
    },
}

/// 保留排序的原因
#[derive(Debug, Clone, PartialEq)]
pub enum SortKeepReason {
    /// 无 Limit 子节点，无法转换为 TopN
    NoLimitForTopN,
    /// Limit 值太小，不值得转换
    LimitTooSmall,
    /// 基于代价分析保留排序更优
    CostBasedDecision,
}

/// 转换为 TopN 的原因
#[derive(Debug, Clone, PartialEq)]
pub enum TopNConversionReason {
    /// Sort + Limit 组合，TopN 代价更低
    SortWithLimit,
    /// 小数据量使用 TopN 更优
    SmallLimit,
    /// 基于代价分析
    CostBased,
}

/// 排序优化上下文
#[derive(Debug, Clone)]
pub struct SortContext {
    /// 排序节点
    pub sort_node: SortNode,
    /// 输入行数估计
    pub input_rows: u64,
    /// 是否有 Limit 子节点
    pub has_limit_child: bool,
    /// Limit 值（如果有）
    pub limit_value: Option<i64>,
}

impl SortContext {
    /// 创建新的排序上下文
    pub fn new(sort_node: SortNode, input_rows: u64) -> Self {
        Self {
            sort_node,
            input_rows,
            has_limit_child: false,
            limit_value: None,
        }
    }

    /// 设置 Limit 信息
    pub fn with_limit(mut self, limit: i64) -> Self {
        self.has_limit_child = true;
        self.limit_value = Some(limit);
        self
    }
}

/// 排序消除优化器
///
/// **基于代价模型**的排序优化器，专注于 Sort + Limit 到 TopN 的转换决策。
///
/// 注意：索引有序性消除等启发式优化已在 rewrite 阶段完成。
#[derive(Debug)]
pub struct SortEliminationOptimizer {
    cost_calculator: Arc<CostCalculator>,
    /// TopN 转换阈值（当 limit < threshold * input_rows 时考虑转换）
    topn_threshold: f64,
    /// 最小 Limit 值才考虑 TopN
    min_limit_for_topn: i64,
}

impl SortEliminationOptimizer {
    /// 创建新的排序消除优化器
    pub fn new(cost_calculator: Arc<CostCalculator>) -> Self {
        Self {
            cost_calculator,
            topn_threshold: 0.1, // 默认 10%
            min_limit_for_topn: 1,
        }
    }

    /// 设置 TopN 转换阈值
    pub fn with_topn_threshold(mut self, threshold: f64) -> Self {
        self.topn_threshold = threshold.clamp(0.001, 1.0);
        self
    }

    /// 设置最小 Limit 值
    pub fn with_min_limit(mut self, min_limit: i64) -> Self {
        self.min_limit_for_topn = min_limit.max(1);
        self
    }

    /// 优化排序操作
    ///
    /// 基于代价分析决定是否将 Sort + Limit 转换为 TopN。
    ///
    /// # 参数
    /// - `context`: 排序优化上下文
    ///
    /// # 返回
    /// 排序优化决策（保留排序或转换为 TopN）
    pub fn optimize_sort(&self, context: &SortContext) -> SortEliminationDecision {
        let sort_items = context.sort_node.sort_items();

        // 检查是否可以转换为 TopN
        if let Some(decision) = self.check_topn_conversion(context, sort_items) {
            return decision;
        }

        // 无法转换，保留排序
        let sort_cost = self.calculate_sort_cost(context.input_rows, sort_items.len());
        SortEliminationDecision::KeepSort {
            reason: SortKeepReason::NoLimitForTopN,
            estimated_cost: sort_cost,
        }
    }

    /// 检查是否可以转换为 TopN
    ///
    /// 基于代价比较决定是否将 Sort + Limit 转换为 TopN。
    /// 这是**基于代价的决策**，不是无条件转换。
    fn check_topn_conversion(
        &self,
        context: &SortContext,
        sort_items: &[SortItem],
    ) -> Option<SortEliminationDecision> {
        let limit = context.limit_value?;

        if limit < self.min_limit_for_topn {
            return None;
        }

        // 检查是否满足 TopN 转换条件
        let limit_ratio = limit as f64 / context.input_rows as f64;

        if limit_ratio < self.topn_threshold || context.input_rows > 10000 {
            let original_cost = self.calculate_sort_cost(context.input_rows, sort_items.len());
            let topn_cost = self
                .cost_calculator
                .calculate_topn_cost(context.input_rows, limit);

            if topn_cost < original_cost {
                return Some(SortEliminationDecision::ConvertToTopN {
                    reason: if context.has_limit_child {
                        TopNConversionReason::SortWithLimit
                    } else {
                        TopNConversionReason::CostBased
                    },
                    topn_cost,
                    original_cost,
                });
            }
        }

        None
    }

    /// 计算排序代价
    fn calculate_sort_cost(&self, input_rows: u64, sort_columns: usize) -> f64 {
        self.cost_calculator
            .calculate_sort_cost(input_rows, sort_columns, None)
    }

    /// 检查是否可以转换为 TopN 节点
    ///
    /// # 参数
    /// - `sort_items`: 排序项
    /// - `limit`: Limit 值
    /// - `input_rows`: 输入行数
    ///
    /// # 返回
    /// 如果可以转换，返回 (TopN 代价, 原排序代价)
    pub fn check_topn_conversion_cost(
        &self,
        sort_items: &[SortItem],
        limit: i64,
        input_rows: u64,
    ) -> Option<(f64, f64)> {
        if limit < self.min_limit_for_topn {
            return None;
        }

        let limit_ratio = limit as f64 / input_rows as f64;

        if limit_ratio < self.topn_threshold || input_rows > 10000 {
            let original_cost = self.calculate_sort_cost(input_rows, sort_items.len());
            let topn_cost = self
                .cost_calculator
                .calculate_topn_cost(input_rows, limit);

            if topn_cost < original_cost {
                return Some((topn_cost, original_cost));
            }
        }

        None
    }

    /// 获取排序优化建议
    ///
    /// 分析排序操作并返回优化建议
    pub fn get_optimization_advice(&self, context: &SortContext) -> Vec<String> {
        let mut advice = Vec::new();

        match self.optimize_sort(context) {
            SortEliminationDecision::ConvertToTopN {
                reason,
                topn_cost,
                original_cost,
            } => {
                let savings = original_cost - topn_cost;
                advice.push(format!(
                    "建议将 Sort + Limit 转换为 TopN，原因: {:?}，预计节省代价: {:.2}",
                    reason, savings
                ));
            }
            SortEliminationDecision::KeepSort { reason, .. } => {
                advice.push(format!("保留排序操作，原因: {:?}", reason));

                if matches!(reason, SortKeepReason::NoLimitForTopN) {
                    advice.push("如果查询包含 LIMIT，考虑将 Sort + Limit 转换为 TopN".to_string());
                }
            }
        }

        advice
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::stats::StatisticsManager;

    fn create_test_optimizer() -> SortEliminationOptimizer {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        SortEliminationOptimizer::new(cost_calculator)
    }

    #[test]
    fn test_sort_elimination_optimizer_creation() {
        let optimizer = create_test_optimizer();
        assert_eq!(optimizer.topn_threshold, 0.1);
        assert_eq!(optimizer.min_limit_for_topn, 1);
    }

    #[test]
    fn test_with_topn_threshold() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        let optimizer = SortEliminationOptimizer::new(cost_calculator)
            .with_topn_threshold(0.2);

        assert_eq!(optimizer.topn_threshold, 0.2);
    }

    #[test]
    fn test_with_min_limit() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        let optimizer = SortEliminationOptimizer::new(cost_calculator)
            .with_min_limit(10);

        assert_eq!(optimizer.min_limit_for_topn, 10);
    }

    #[test]
    fn test_threshold_clamping() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));

        let optimizer1 = SortEliminationOptimizer::new(cost_calculator.clone())
            .with_topn_threshold(2.0); // 超过 1.0
        assert_eq!(optimizer1.topn_threshold, 1.0);

        let optimizer2 = SortEliminationOptimizer::new(cost_calculator)
            .with_topn_threshold(0.0001); // 小于 0.001
        assert_eq!(optimizer2.topn_threshold, 0.001);
    }

    #[test]
    fn test_context_creation() {
        let start_node = crate::query::planner::plan::core::nodes::StartNode::new();
        let sort_node = SortNode::new(
            crate::query::planner::plan::PlanNodeEnum::Start(start_node),
            vec![SortItem::asc("name".to_string())],
        ).expect("Failed to create SortNode");

        let context = SortContext::new(sort_node, 1000);
        assert_eq!(context.input_rows, 1000);
        assert!(!context.has_limit_child);
        assert_eq!(context.limit_value, None);
    }

    #[test]
    fn test_context_with_limit() {
        let start_node = crate::query::planner::plan::core::nodes::StartNode::new();
        let sort_node = SortNode::new(
            crate::query::planner::plan::PlanNodeEnum::Start(start_node),
            vec![SortItem::asc("name".to_string())],
        ).expect("Failed to create SortNode");

        let context = SortContext::new(sort_node, 1000)
            .with_limit(10);

        assert!(context.has_limit_child);
        assert_eq!(context.limit_value, Some(10));
    }

    #[test]
    fn test_check_topn_conversion_cost_limit_too_small() {
        let optimizer = create_test_optimizer();
        let sort_items = vec![SortItem::asc("name".to_string())];

        // Limit 为 0，应该返回 None
        let result = optimizer.check_topn_conversion_cost(&sort_items, 0, 1000);
        assert_eq!(result, None);
    }

    #[test]
    fn test_sort_keep_reason_display() {
        let reason = SortKeepReason::NoLimitForTopN;
        assert!(format!("{:?}", reason).contains("NoLimitForTopN"));

        let reason = SortKeepReason::LimitTooSmall;
        assert!(format!("{:?}", reason).contains("LimitTooSmall"));

        let reason = SortKeepReason::CostBasedDecision;
        assert!(format!("{:?}", reason).contains("CostBasedDecision"));
    }

    #[test]
    fn test_topn_conversion_reason_display() {
        let reason = TopNConversionReason::SortWithLimit;
        assert!(format!("{:?}", reason).contains("SortWithLimit"));

        let reason = TopNConversionReason::SmallLimit;
        assert!(format!("{:?}", reason).contains("SmallLimit"));

        let reason = TopNConversionReason::CostBased;
        assert!(format!("{:?}", reason).contains("CostBased"));
    }
}
