//! 图遍历方向优化器模块
//!
//! 基于边统计信息选择最优的遍历方向（正向或反向）
//!
//! ## 优化策略
//!
//! - 选择度数较小的方向以减少中间结果
//! - 考虑超级节点的影响
//! - 支持基于代价的方向选择
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::TraversalDirectionOptimizer;
//! use graphdb::query::optimizer::cost::CostCalculator;
//! use std::sync::Arc;
//!
//! let optimizer = TraversalDirectionOptimizer::new(cost_calculator);
//! let decision = optimizer.optimize_direction("KNOWS", None);
//! ```

use std::sync::Arc;

use crate::core::types::EdgeDirection;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::stats::EdgeTypeStatistics;

/// 遍历方向决策
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraversalDirection {
    /// 正向遍历（出边方向）
    /// 从源顶点遍历到目标顶点
    Forward,
    /// 反向遍历（入边方向）
    /// 从目标顶点遍历到源顶点
    Backward,
    /// 双向遍历
    /// 同时考虑两个方向
    Bidirectional,
}

impl TraversalDirection {
    /// 获取方向名称
    pub fn name(&self) -> &'static str {
        match self {
            TraversalDirection::Forward => "Forward",
            TraversalDirection::Backward => "Backward",
            TraversalDirection::Bidirectional => "Bidirectional",
        }
    }

    /// 转换为 EdgeDirection
    pub fn to_edge_direction(&self) -> EdgeDirection {
        match self {
            TraversalDirection::Forward => EdgeDirection::Out,
            TraversalDirection::Backward => EdgeDirection::In,
            TraversalDirection::Bidirectional => EdgeDirection::Both,
        }
    }

    /// 从 EdgeDirection 转换
    pub fn from_edge_direction(direction: &EdgeDirection) -> Self {
        match direction {
            EdgeDirection::Out => TraversalDirection::Forward,
            EdgeDirection::In => TraversalDirection::Backward,
            EdgeDirection::Both => TraversalDirection::Bidirectional,
        }
    }
}

/// 方向选择原因
#[derive(Debug, Clone)]
pub enum DirectionSelectionReason {
    /// 出度小于入度
    OutDegreeLower {
        out_degree: f64,
        in_degree: f64,
    },
    /// 入度小于出度
    InDegreeLower {
        in_degree: f64,
        out_degree: f64,
    },
    /// 度数相等或接近
    DegreesEqual {
        out_degree: f64,
        in_degree: f64,
    },
    /// 基于代价比较的选择
    CostBased {
        forward_cost: f64,
        backward_cost: f64,
    },
    /// 避免超级节点
    AvoidSuperNode {
        super_node_direction: TraversalDirection,
        threshold: f64,
    },
    /// 统计信息不可用，使用默认方向
    StatsUnavailable,
    /// 显式指定方向
    ExplicitDirection,
}

/// 遍历方向决策结果
#[derive(Debug, Clone)]
pub struct TraversalDirectionDecision {
    /// 选择的方向
    pub direction: TraversalDirection,
    /// 估计的输出行数
    pub estimated_output_rows: u64,
    /// 估计的代价
    pub estimated_cost: f64,
    /// 选择原因
    pub reason: DirectionSelectionReason,
    /// 平均度数（选择的方向）
    pub avg_degree: f64,
    /// 是否涉及超级节点
    pub involves_super_node: bool,
}

/// 遍历方向优化器
///
/// 基于边的统计信息选择最优遍历方向
#[derive(Debug)]
pub struct TraversalDirectionOptimizer {
    cost_calculator: Arc<CostCalculator>,
    /// 超级节点阈值（度数超过此值视为超级节点）
    super_node_threshold: f64,
    /// 度数差异阈值（差异小于此值视为相等）
    degree_equality_threshold: f64,
}

/// 方向优化上下文
#[derive(Debug, Clone)]
pub struct DirectionContext {
    /// 边类型
    pub edge_type: String,
    /// 起始节点数量
    pub start_nodes: u64,
    /// 显式指定的方向（如果有）
    pub explicit_direction: Option<TraversalDirection>,
    /// 是否允许双向遍历
    pub allow_bidirectional: bool,
    /// 遍历步数
    pub steps: u32,
}

impl TraversalDirectionOptimizer {
    /// 创建新的遍历方向优化器
    pub fn new(cost_calculator: Arc<CostCalculator>) -> Self {
        Self {
            cost_calculator,
            super_node_threshold: 1000.0, // 默认超级节点阈值
            degree_equality_threshold: 0.1, // 10% 差异视为相等
        }
    }

    /// 设置超级节点阈值
    pub fn with_super_node_threshold(mut self, threshold: f64) -> Self {
        self.super_node_threshold = threshold;
        self
    }

    /// 设置度数相等阈值
    pub fn with_equality_threshold(mut self, threshold: f64) -> Self {
        self.degree_equality_threshold = threshold;
        self
    }

    /// 优化遍历方向
    ///
    /// # 参数
    /// - `context`: 方向优化上下文
    ///
    /// # 返回
    /// 方向决策结果
    pub fn optimize_direction(&self, context: &DirectionContext) -> TraversalDirectionDecision {
        // 如果有显式指定的方向，优先使用
        if let Some(explicit) = context.explicit_direction {
            return self.create_explicit_decision(context, explicit);
        }

        // 获取边统计信息
        let stats = self
            .cost_calculator
            .statistics_manager()
            .get_edge_stats(&context.edge_type);

        match stats {
            Some(edge_stats) => self.optimize_with_stats(context, &edge_stats),
            None => self.create_default_decision(context),
        }
    }

    /// 基于统计信息优化方向
    fn optimize_with_stats(
        &self,
        context: &DirectionContext,
        stats: &EdgeTypeStatistics,
    ) -> TraversalDirectionDecision {
        let out_degree = stats.avg_out_degree;
        let in_degree = stats.avg_in_degree;

        // 检查是否涉及超级节点
        let forward_is_super = out_degree > self.super_node_threshold;
        let backward_is_super = in_degree > self.super_node_threshold;

        // 如果两个方向都是超级节点，选择度数较小的
        if forward_is_super && backward_is_super {
            let direction = if out_degree <= in_degree {
                TraversalDirection::Forward
            } else {
                TraversalDirection::Backward
            };

            let avg_degree = if out_degree <= in_degree {
                out_degree
            } else {
                in_degree
            };

            return TraversalDirectionDecision {
                direction,
                estimated_output_rows: (context.start_nodes as f64 * avg_degree) as u64,
                estimated_cost: self.calculate_cost(context, true),
                reason: DirectionSelectionReason::AvoidSuperNode {
                    super_node_direction: if out_degree > in_degree {
                        TraversalDirection::Forward
                    } else {
                        TraversalDirection::Backward
                    },
                    threshold: self.super_node_threshold,
                },
                avg_degree,
                involves_super_node: true,
            };
        }

        // 如果只有一个方向是超级节点，避免该方向
        if forward_is_super {
            return TraversalDirectionDecision {
                direction: TraversalDirection::Backward,
                estimated_output_rows: (context.start_nodes as f64 * in_degree) as u64,
                estimated_cost: self.calculate_cost(context, false),
                reason: DirectionSelectionReason::AvoidSuperNode {
                    super_node_direction: TraversalDirection::Forward,
                    threshold: self.super_node_threshold,
                },
                avg_degree: in_degree,
                involves_super_node: true,
            };
        }

        if backward_is_super {
            return TraversalDirectionDecision {
                direction: TraversalDirection::Forward,
                estimated_output_rows: (context.start_nodes as f64 * out_degree) as u64,
                estimated_cost: self.calculate_cost(context, false),
                reason: DirectionSelectionReason::AvoidSuperNode {
                    super_node_direction: TraversalDirection::Backward,
                    threshold: self.super_node_threshold,
                },
                avg_degree: out_degree,
                involves_super_node: true,
            };
        }

        // 基于度数比较选择方向
        let degree_diff = (out_degree - in_degree).abs();
        let degree_ratio = degree_diff / ((out_degree + in_degree) / 2.0);

        if degree_ratio < self.degree_equality_threshold {
            // 度数接近，基于代价计算选择
            self.select_by_cost(context, out_degree, in_degree)
        } else if out_degree < in_degree {
            // 出度较小，选择正向
            TraversalDirectionDecision {
                direction: TraversalDirection::Forward,
                estimated_output_rows: (context.start_nodes as f64 * out_degree) as u64,
                estimated_cost: self.calculate_cost(context, false),
                reason: DirectionSelectionReason::OutDegreeLower {
                    out_degree,
                    in_degree,
                },
                avg_degree: out_degree,
                involves_super_node: false,
            }
        } else {
            TraversalDirectionDecision {
                direction: TraversalDirection::Backward,
                estimated_output_rows: (context.start_nodes as f64 * in_degree) as u64,
                estimated_cost: self.calculate_cost(context, false),
                reason: DirectionSelectionReason::InDegreeLower {
                    in_degree,
                    out_degree,
                },
                avg_degree: in_degree,
                involves_super_node: false,
            }
        }
    }

    /// 基于代价选择方向
    fn select_by_cost(
        &self,
        context: &DirectionContext,
        out_degree: f64,
        in_degree: f64,
    ) -> TraversalDirectionDecision {
        let forward_cost = self.calculate_cost(context, false);
        let backward_cost = self.calculate_cost(context, false);

        let (direction, avg_degree) = if forward_cost <= backward_cost {
            (TraversalDirection::Forward, out_degree)
        } else {
            (TraversalDirection::Backward, in_degree)
        };

        TraversalDirectionDecision {
            direction,
            estimated_output_rows: (context.start_nodes as f64 * avg_degree) as u64,
            estimated_cost: forward_cost.min(backward_cost),
            reason: DirectionSelectionReason::CostBased {
                forward_cost,
                backward_cost,
            },
            avg_degree,
            involves_super_node: false,
        }
    }

    /// 计算遍历代价
    fn calculate_cost(&self, context: &DirectionContext, is_super: bool) -> f64 {
        let base_cost = self
            .cost_calculator
            .calculate_expand_cost(context.start_nodes, Some(&context.edge_type));

        if is_super {
            base_cost * self.cost_calculator.config().super_node_penalty
        } else {
            base_cost
        }
    }

    /// 创建显式方向的决策
    fn create_explicit_decision(
        &self,
        context: &DirectionContext,
        direction: TraversalDirection,
    ) -> TraversalDirectionDecision {
        // 尝试获取统计信息
        let avg_degree = self
            .cost_calculator
            .statistics_manager()
            .get_edge_stats(&context.edge_type)
            .map(|s| match direction {
                TraversalDirection::Forward => s.avg_out_degree,
                TraversalDirection::Backward => s.avg_in_degree,
                TraversalDirection::Bidirectional => (s.avg_out_degree + s.avg_in_degree) / 2.0,
            })
            .unwrap_or(2.0);

        let is_super = avg_degree > self.super_node_threshold;

        TraversalDirectionDecision {
            direction,
            estimated_output_rows: (context.start_nodes as f64 * avg_degree) as u64,
            estimated_cost: self.calculate_cost(context, is_super),
            reason: DirectionSelectionReason::ExplicitDirection,
            avg_degree,
            involves_super_node: is_super,
        }
    }

    /// 创建默认决策（统计信息不可用）
    fn create_default_decision(&self, context: &DirectionContext) -> TraversalDirectionDecision {
        let default_degree = 2.0;

        TraversalDirectionDecision {
            direction: TraversalDirection::Forward, // 默认正向
            estimated_output_rows: (context.start_nodes as f64 * default_degree) as u64,
            estimated_cost: self.calculate_cost(context, false),
            reason: DirectionSelectionReason::StatsUnavailable,
            avg_degree: default_degree,
            involves_super_node: false,
        }
    }

    /// 快速方向选择（简化版本，用于决策缓存）
    pub fn select_direction_quick(&self, edge_type: &str) -> TraversalDirection {
        let stats = self.cost_calculator.statistics_manager().get_edge_stats(edge_type);

        match stats {
            Some(s) => {
                if s.avg_out_degree > self.super_node_threshold
                    && s.avg_in_degree <= self.super_node_threshold
                {
                    TraversalDirection::Backward
                } else if s.avg_in_degree > self.super_node_threshold
                    && s.avg_out_degree <= self.super_node_threshold
                {
                    TraversalDirection::Forward
                } else if s.avg_out_degree <= s.avg_in_degree {
                    TraversalDirection::Forward
                } else {
                    TraversalDirection::Backward
                }
            }
            None => TraversalDirection::Forward,
        }
    }

    /// 检查是否需要双向遍历
    pub fn should_use_bidirectional(&self, edge_type: &str) -> bool {
        let stats = self.cost_calculator.statistics_manager().get_edge_stats(edge_type);

        match stats {
            Some(s) => {
                // 如果两个方向的度数都很低，可以使用双向遍历
                s.avg_out_degree < 10.0 && s.avg_in_degree < 10.0
            }
            None => false,
        }
    }

    /// 获取边的度数信息
    pub fn get_degree_info(&self, edge_type: &str) -> Option<DegreeInfo> {
        self.cost_calculator
            .statistics_manager()
            .get_edge_stats(edge_type)
            .map(|s| DegreeInfo {
                avg_out_degree: s.avg_out_degree,
                avg_in_degree: s.avg_in_degree,
                max_out_degree: s.max_out_degree,
                max_in_degree: s.max_in_degree,
                is_out_super: s.avg_out_degree > self.super_node_threshold,
                is_in_super: s.avg_in_degree > self.super_node_threshold,
            })
    }
}

/// 度数信息
#[derive(Debug, Clone)]
pub struct DegreeInfo {
    /// 平均出度
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 最大出度
    pub max_out_degree: u64,
    /// 最大入度
    pub max_in_degree: u64,
    /// 出度方向是否为超级节点
    pub is_out_super: bool,
    /// 入度方向是否为超级节点
    pub is_in_super: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::stats::StatisticsManager;

    fn create_test_optimizer() -> TraversalDirectionOptimizer {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        TraversalDirectionOptimizer::new(cost_calculator).with_super_node_threshold(100.0)
    }

    #[test]
    fn test_explicit_direction() {
        let optimizer = create_test_optimizer();
        let context = DirectionContext {
            edge_type: "KNOWS".to_string(),
            start_nodes: 100,
            explicit_direction: Some(TraversalDirection::Backward),
            allow_bidirectional: false,
            steps: 1,
        };

        let decision = optimizer.optimize_direction(&context);
        assert_eq!(decision.direction, TraversalDirection::Backward);
        matches!(decision.reason, DirectionSelectionReason::ExplicitDirection);
    }

    #[test]
    fn test_default_direction_when_no_stats() {
        let optimizer = create_test_optimizer();
        let context = DirectionContext {
            edge_type: "UNKNOWN".to_string(),
            start_nodes: 100,
            explicit_direction: None,
            allow_bidirectional: false,
            steps: 1,
        };

        let decision = optimizer.optimize_direction(&context);
        assert_eq!(decision.direction, TraversalDirection::Forward);
        matches!(decision.reason, DirectionSelectionReason::StatsUnavailable);
    }

    #[test]
    fn test_traversal_direction_name() {
        assert_eq!(TraversalDirection::Forward.name(), "Forward");
        assert_eq!(TraversalDirection::Backward.name(), "Backward");
        assert_eq!(TraversalDirection::Bidirectional.name(), "Bidirectional");
    }

    #[test]
    fn test_traversal_direction_conversion() {
        assert_eq!(
            TraversalDirection::from_edge_direction(&EdgeDirection::Out),
            TraversalDirection::Forward
        );
        assert_eq!(
            TraversalDirection::from_edge_direction(&EdgeDirection::In),
            TraversalDirection::Backward
        );
        assert_eq!(
            TraversalDirection::from_edge_direction(&EdgeDirection::Both),
            TraversalDirection::Bidirectional
        );

        assert_eq!(
            TraversalDirection::Forward.to_edge_direction(),
            EdgeDirection::Out
        );
        assert_eq!(
            TraversalDirection::Backward.to_edge_direction(),
            EdgeDirection::In
        );
        assert_eq!(
            TraversalDirection::Bidirectional.to_edge_direction(),
            EdgeDirection::Both
        );
    }

    #[test]
    fn test_quick_selection_no_stats() {
        let optimizer = create_test_optimizer();
        let direction = optimizer.select_direction_quick("UNKNOWN");
        assert_eq!(direction, TraversalDirection::Forward);
    }

    #[test]
    fn test_degree_info() {
        let optimizer = create_test_optimizer();
        // 未知边类型应该返回 None
        let info = optimizer.get_degree_info("UNKNOWN");
        assert!(info.is_none());
    }
}
