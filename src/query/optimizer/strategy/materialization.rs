//! CTE (Common Table Expression) Materialization Optimizer Module
//!
//! “Analysis-based optimization strategy for CTE materialization” – Determines whether to materialize a CTE (Common Table Expression) in memory or not.
//!
//! ## Optimization Strategies
//!
//! Materialize the CTE (Common Table Expression) that will be referenced multiple times to avoid duplicate calculations.
//! - 仅物化确定性的 CTE（不含 rand(), now() 等）
//! Decisions are made based on the reference count and the size of the result set.
//!
//! ## Applicable Conditions
//!
//! The number of citations for CTE is greater than 1.
//! 2. CTE 不包含非确定性函数（如 rand(), now()）
//! 3. The estimated number of rows for CTE is less than 10,000.
//! 4. The complexity of CTE (Common Table Expression) is less than 80.
//!
//! ## Usage Examples
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::MaterializationOptimizer;
//! use graphdb::query::optimizer::OptimizerEngine;
//!
//! let optimizer = MaterializationOptimizer::new(
//!     engine.reference_count_analyzer(),
//!     engine.expression_analyzer(),
//!     engine.stats_manager(),
//! );
//! let decision = optimizer.should_materialize(&cte_node);
//! ```

use crate::query::optimizer::analysis::{ExpressionAnalyzer, ReferenceCountAnalyzer};
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planning::plan::core::nodes::{MaterializeNode, PlanNodeEnum};

/// CTE (Common Table Expression) materialization decision
#[derive(Debug, Clone, PartialEq)]
pub enum MaterializationDecision {
    /// Materialized CTE (Common Table Expression)
    Materialize {
        /// Physical causes
        reason: MaterializeReason,
        /// Number of citations
        reference_count: usize,
        /// Estimating the size of the result set
        estimated_rows: u64,
        /// Estimating the materialization costs
        materialize_cost: f64,
        /// Estimate the cost of redundant calculations
        recompute_cost: f64,
    },
    /// Immaterialization
    DoNotMaterialize {
        /// Reasons for non-materialization
        reason: NoMaterializeReason,
    },
}

/// 物化原因
#[derive(Debug, Clone, PartialEq)]
pub enum MaterializeReason {
    /// Cited multiple times
    MultipleReferences,
    /// The materialization based on cost analysis is more optimal.
    CostBased,
}

/// 不物化原因
#[derive(Debug, Clone, PartialEq)]
pub enum NoMaterializeReason {
    /// Cited only once
    SingleReference,
    /// Contains non-deterministic functions
    NonDeterministic,
    /// The result set is too large.
    TooLarge,
    /// The expression is too complex.
    TooComplex,
}

/// CTE (Common Table Expression) Materialization Optimizer
///
/// Decide whether to materialize a CTE (Common Table Expression) based on reference count analysis, expression analysis, and statistical information.
#[derive(Debug, Clone)]
pub struct MaterializationOptimizer {
    /// Reference Count Analyzer
    reference_count_analyzer: ReferenceCountAnalyzer,
    /// Expression Analyzer
    expression_analyzer: ExpressionAnalyzer,
    /// Statistical Information Manager
    stats_manager: StatisticsManager,
    /// Threshold for the minimum number of citations
    min_reference_count: usize,
    /// Maximum result set size threshold
    max_result_rows: u64,
    /// Maximum expression complexity threshold
    max_complexity: u32,
}

impl MaterializationOptimizer {
    /// Create a new optimizer.
    pub fn new(
        reference_count_analyzer: &ReferenceCountAnalyzer,
        expression_analyzer: &ExpressionAnalyzer,
        stats_manager: &StatisticsManager,
    ) -> Self {
        Self {
            reference_count_analyzer: reference_count_analyzer.clone(),
            expression_analyzer: expression_analyzer.clone(),
            stats_manager: stats_manager.clone(),
            min_reference_count: 2,
            max_result_rows: 10000,
            max_complexity: 80,
        }
    }

    /// Set a threshold for the minimum number of citations required
    pub fn with_min_reference_count(mut self, count: usize) -> Self {
        self.min_reference_count = count;
        self
    }

    /// Set a threshold for the maximum size of the result set
    pub fn with_max_result_rows(mut self, max_rows: u64) -> Self {
        self.max_result_rows = max_rows;
        self
    }

    /// Set a threshold for the maximum complexity.
    pub fn with_max_complexity(mut self, max_complexity: u32) -> Self {
        self.max_complexity = max_complexity;
        self
    }

    /// Determine whether it is appropriate to materialize the CTE (Common Table Expression).
    ///
    /// # Parameters
    /// `cte_node`: The root node of the CTE (Common Table Expression) sub-plan.
    /// `plan_root`: The root node of the entire plan tree (used for reference count analysis)
    ///
    /// # Return
    /// Materialized Decision Making
    pub fn should_materialize(
        &self,
        cte_node: &PlanNodeEnum,
        plan_root: &PlanNodeEnum,
    ) -> MaterializationDecision {
        // 1. Perform a reference counting analysis.
        let ref_analysis = self.reference_count_analyzer.analyze(plan_root);

        // 2. Check whether CTE is referenced multiple times.
        let ref_info = match ref_analysis.node_reference_map.get(&cte_node.id()) {
            Some(info) => info,
            None => {
                return MaterializationDecision::DoNotMaterialize {
                    reason: NoMaterializeReason::SingleReference,
                }
            }
        };

        if ref_info.reference_count < self.min_reference_count {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::SingleReference,
            };
        }

        // 3. Check whether CTE is deterministic.
        if !self.is_deterministic(cte_node) {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::NonDeterministic,
            };
        }

        // 4. Check the complexity of the expression.
        let complexity = self.get_max_complexity(cte_node);
        if complexity > self.max_complexity {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::TooComplex,
            };
        }

        // 5. Estimating the size of the result set
        let estimated_rows = self.estimate_result_rows(cte_node);
        if estimated_rows > self.max_result_rows {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::TooLarge,
            };
        }

        // 6. Comparison of costs
        let recompute_cost = self.estimate_recompute_cost(ref_info.reference_count, estimated_rows);
        let materialize_cost = self.estimate_materialize_cost(estimated_rows, complexity);

        if materialize_cost < recompute_cost {
            MaterializationDecision::Materialize {
                reason: MaterializeReason::CostBased,
                reference_count: ref_info.reference_count,
                estimated_rows,
                materialize_cost,
                recompute_cost,
            }
        } else {
            MaterializationDecision::Materialize {
                reason: MaterializeReason::MultipleReferences,
                reference_count: ref_info.reference_count,
                estimated_rows,
                materialize_cost,
                recompute_cost,
            }
        }
    }

    /// Check whether the node is deterministic.
    fn is_deterministic(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                let analysis = self.expression_analyzer.analyze(condition);
                if !analysis.is_deterministic {
                    return false;
                }
                self.is_deterministic(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n))
            }
            PlanNodeEnum::Project(n) => self.is_deterministic(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Aggregate(n) => {
                // Aggregate functions are usually deterministic, unless their inputs are non-deterministic.
                // We ensure certainty by recursively checking the input nodes.
                self.is_deterministic(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n))
            }
            PlanNodeEnum::Sort(n) => self.is_deterministic(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Limit(n) => self.is_deterministic(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::TopN(n) => self.is_deterministic(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Union(n) => self.is_deterministic(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::InnerJoin(join_node) => {
                for key in join_node.hash_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                for key in join_node.probe_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                true
            }
            PlanNodeEnum::LeftJoin(join_node) => {
                for key in join_node.hash_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                for key in join_node.probe_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                true
            }
            PlanNodeEnum::CrossJoin(_) => true,
            PlanNodeEnum::HashInnerJoin(join_node) => {
                for key in join_node.hash_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                for key in join_node.probe_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                true
            }
            PlanNodeEnum::HashLeftJoin(join_node) => {
                for key in join_node.hash_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                for key in join_node.probe_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                true
            }
            PlanNodeEnum::FullOuterJoin(join_node) => {
                for key in join_node.hash_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                for key in join_node.probe_keys() {
                    let analysis = self.expression_analyzer.analyze(key);
                    if !analysis.is_deterministic {
                        return false;
                    }
                }
                true
            }
            PlanNodeEnum::ScanVertices(_) => true,
            PlanNodeEnum::ScanEdges(_) => true,
            PlanNodeEnum::GetVertices(_) => true,
            PlanNodeEnum::GetEdges(_) => true,
            PlanNodeEnum::IndexScan(_) => true,
            _ => true,
        }
    }

    /// Obtain the maximum expression complexity of the node.
    fn get_max_complexity(&self, node: &PlanNodeEnum) -> u32 {
        let mut max_complexity = 0u32;

        match node {
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                let analysis = self.expression_analyzer.analyze(condition);
                max_complexity = max_complexity.max(analysis.complexity_score);
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            PlanNodeEnum::Project(n) => {
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            PlanNodeEnum::Aggregate(n) => {
                // The complexity of aggregate functions is determined by the input.
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            _ => {}
        }

        max_complexity
    }

    /// Estimated number of rows in the result set
    fn estimate_result_rows(&self, node: &PlanNodeEnum) -> u64 {
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                if let Some(tag_name) = n.tag() {
                    if let Some(stats) = self.stats_manager.get_tag_stats(tag_name) {
                        stats.vertex_count
                    } else {
                        1000
                    }
                } else {
                    1000
                }
            }
            PlanNodeEnum::Filter(n) => (self.estimate_result_rows(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ) as f64
                * 0.3) as u64,
            PlanNodeEnum::InnerJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::LeftJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::CrossJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::HashInnerJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::HashLeftJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::FullOuterJoin(join_node) => {
                let left_rows = self.estimate_result_rows(join_node.left_input());
                let right_rows = self.estimate_result_rows(join_node.right_input());
                (left_rows as f64 * right_rows as f64 * 0.3) as u64
            }
            PlanNodeEnum::Aggregate(n) => {
                let input_rows = self.estimate_result_rows(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n));
                let group_keys = n.group_keys().len();
                if group_keys == 0 {
                    1
                } else {
                    (input_rows as f64 / (group_keys as f64 * 10.0)) as u64
                }
            }
            _ => 1000,
        }
    }

    /// 估算重复计算代价
    fn estimate_recompute_cost(&self, reference_count: usize, rows: u64) -> f64 {
        // The calculation must be performed again for each citation.
        (reference_count as f64) * (rows as f64) * 0.1
    }

    /// 估算物化代价
    fn estimate_materialize_cost(&self, rows: u64, complexity: u32) -> f64 {
        // Materialization cost = Computational cost + Storage cost
        let compute_cost = (rows as f64) * 0.1;
        let storage_cost = (rows as f64) * 0.05; // Storage costs
        let complexity_overhead = (complexity as f64) * 0.01;

        compute_cost + storage_cost + complexity_overhead
    }

    /// Perform the materialization transformation.
    ///
    /// # 参数
    /// - `cte_node`: CTE 子计划根节点
    ///
    /// # 返回
    /// Nodes that have the MaterializeNode package installed
    pub fn materialize(
        &self,
        cte_node: PlanNodeEnum,
    ) -> Result<PlanNodeEnum, crate::query::planning::planner::PlannerError> {
        let materialize_node = MaterializeNode::new(cte_node)?;
        Ok(PlanNodeEnum::Materialize(materialize_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::validator::context::expression_context::ExpressionAnalysisContext;

    #[test]
    fn test_optimizer_creation() {
        let reference_count_analyzer = ReferenceCountAnalyzer::new();
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let optimizer = MaterializationOptimizer::new(
            &reference_count_analyzer,
            &expression_analyzer,
            &stats_manager,
        );
        assert_eq!(optimizer.min_reference_count, 2);
    }

    #[test]
    fn test_optimizer_with_config() {
        let reference_count_analyzer = ReferenceCountAnalyzer::new();
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let optimizer = MaterializationOptimizer::new(
            &reference_count_analyzer,
            &expression_analyzer,
            &stats_manager,
        )
        .with_min_reference_count(3)
        .with_max_result_rows(5000)
        .with_max_complexity(60);
        assert_eq!(optimizer.min_reference_count, 3);
        assert_eq!(optimizer.max_result_rows, 5000);
        assert_eq!(optimizer.max_complexity, 60);
    }

    #[test]
    fn test_deterministic_check() {
        let reference_count_analyzer = ReferenceCountAnalyzer::new();
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let _optimizer = MaterializationOptimizer::new(
            &reference_count_analyzer,
            &expression_analyzer,
            &stats_manager,
        );

        // Simple expressions are deterministic.
        let simple_expr = Expression::Literal(crate::core::Value::Int(42));
        let ctx = std::sync::Arc::new(ExpressionAnalysisContext::new());
        let meta = crate::core::types::expr::ExpressionMeta::new(simple_expr);
        let id = ctx.register_expression(meta);
        let simple_ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);
        let analysis = expression_analyzer.analyze(&simple_ctx_expr);
        assert!(analysis.is_deterministic);

        // rand() 函数是非确定性的
        let nondet_expr = Expression::Function {
            name: "rand".to_string(),
            args: vec![],
        };
        let ctx2 = std::sync::Arc::new(ExpressionAnalysisContext::new());
        let meta2 = crate::core::types::expr::ExpressionMeta::new(nondet_expr);
        let id2 = ctx2.register_expression(meta2);
        let nondet_ctx_expr = crate::core::types::ContextualExpression::new(id2, ctx2);
        let analysis = expression_analyzer.analyze(&nondet_ctx_expr);
        assert!(!analysis.is_deterministic);
    }

    #[test]
    fn test_cost_estimation() {
        let reference_count_analyzer = ReferenceCountAnalyzer::new();
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let _optimizer = MaterializationOptimizer::new(
            &reference_count_analyzer,
            &expression_analyzer,
            &stats_manager,
        );

        // Test Cost Estimation
        let recompute_cost = _optimizer.estimate_recompute_cost(3, 1000);
        let materialize_cost = _optimizer.estimate_materialize_cost(1000, 50);

        assert!(recompute_cost > 0.0);
        assert!(materialize_cost > 0.0);
    }
}
