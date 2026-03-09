//! CTE 物化优化器模块
//!
//! **基于分析的 CTE 物化优化策略**，决定是否将 CTE（Common Table Expression）物化到内存中。
//!
//! ## 优化策略
//!
//! - 将被多次引用的 CTE 物化，避免重复计算
//! - 仅物化确定性的 CTE（不含 rand(), now() 等）
//! - 基于引用计数和结果集大小做决策
//!
//! ## 适用条件
//!
//! 1. CTE 被引用次数 > 1
//! 2. CTE 不包含非确定性函数（如 rand(), now()）
//! 3. CTE 估算行数 < 10000
//! 4. CTE 复杂度 < 80
//!
//! ## 使用示例
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

use crate::core::Expression;
use crate::query::optimizer::analysis::{ExpressionAnalyzer, ReferenceCountAnalyzer};
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::core::nodes::{MaterializeNode, PlanNodeEnum};
use crate::query::validator::context::ExpressionAnalysisContext;

/// CTE 物化决策
#[derive(Debug, Clone, PartialEq)]
pub enum MaterializationDecision {
    /// 物化 CTE
    Materialize {
        /// 物化原因
        reason: MaterializeReason,
        /// 引用次数
        reference_count: usize,
        /// 估算结果集大小
        estimated_rows: u64,
        /// 估算物化代价
        materialize_cost: f64,
        /// 估算重复计算代价
        recompute_cost: f64,
    },
    /// 不物化
    DoNotMaterialize {
        /// 不物化原因
        reason: NoMaterializeReason,
    },
}

/// 物化原因
#[derive(Debug, Clone, PartialEq)]
pub enum MaterializeReason {
    /// 被多次引用
    MultipleReferences,
    /// 基于代价分析物化更优
    CostBased,
}

/// 不物化原因
#[derive(Debug, Clone, PartialEq)]
pub enum NoMaterializeReason {
    /// 只被引用一次
    SingleReference,
    /// 包含非确定性函数
    NonDeterministic,
    /// 结果集太大
    TooLarge,
    /// 表达式太复杂
    TooComplex,
}

/// CTE 物化优化器
///
/// 基于引用计数分析、表达式分析和统计信息，决定是否物化 CTE。
#[derive(Debug, Clone)]
pub struct MaterializationOptimizer {
    /// 引用计数分析器
    reference_count_analyzer: ReferenceCountAnalyzer,
    /// 表达式分析器
    expression_analyzer: ExpressionAnalyzer,
    /// 统计信息管理器
    stats_manager: StatisticsManager,
    /// 最小引用次数阈值
    min_reference_count: usize,
    /// 最大结果集大小阈值
    max_result_rows: u64,
    /// 最大表达式复杂度阈值
    max_complexity: u32,
}

impl MaterializationOptimizer {
    /// 创建新的优化器
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

    /// 设置最小引用次数阈值
    pub fn with_min_reference_count(mut self, count: usize) -> Self {
        self.min_reference_count = count;
        self
    }

    /// 设置最大结果集大小阈值
    pub fn with_max_result_rows(mut self, max_rows: u64) -> Self {
        self.max_result_rows = max_rows;
        self
    }

    /// 设置最大复杂度阈值
    pub fn with_max_complexity(mut self, max_complexity: u32) -> Self {
        self.max_complexity = max_complexity;
        self
    }

    /// 判断是否应该物化 CTE
    ///
    /// # 参数
    /// - `cte_node`: CTE 子计划根节点
    /// - `plan_root`: 整个计划树的根节点（用于引用计数分析）
    ///
    /// # 返回
    /// 物化决策
    pub fn should_materialize(
        &self,
        cte_node: &PlanNodeEnum,
        plan_root: &PlanNodeEnum,
    ) -> MaterializationDecision {
        // 1. 执行引用计数分析
        let ref_analysis = self.reference_count_analyzer.analyze(plan_root);

        // 2. 检查 CTE 是否被多次引用
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

        // 3. 检查 CTE 是否是确定性的
        if !self.is_deterministic(cte_node) {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::NonDeterministic,
            };
        }

        // 4. 检查表达式复杂度
        let complexity = self.get_max_complexity(cte_node);
        if complexity > self.max_complexity {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::TooComplex,
            };
        }

        // 5. 估算结果集大小
        let estimated_rows = self.estimate_result_rows(cte_node);
        if estimated_rows > self.max_result_rows {
            return MaterializationDecision::DoNotMaterialize {
                reason: NoMaterializeReason::TooLarge,
            };
        }

        // 6. 代价比较
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

    /// 检查节点是否确定性
    fn is_deterministic(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                let analysis = self.expression_analyzer.analyze(&condition);
                if !analysis.is_deterministic {
                    return false;
                }
                self.is_deterministic(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n))
            }
            PlanNodeEnum::Project(n) => self.is_deterministic(
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Aggregate(n) => {
                // 聚合函数通常是确定性的，除非它们的输入是非确定性的
                // 我们通过递归检查输入节点来确保确定性
                self.is_deterministic(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n))
            }
            PlanNodeEnum::Sort(n) => self.is_deterministic(
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Limit(n) => self.is_deterministic(
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::TopN(n) => self.is_deterministic(
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            PlanNodeEnum::Union(n) => self.is_deterministic(
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
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

    /// 获取节点中最大的表达式复杂度
    fn get_max_complexity(&self, node: &PlanNodeEnum) -> u32 {
        let mut max_complexity = 0u32;

        match node {
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                let analysis = self.expression_analyzer.analyze(&condition);
                max_complexity = max_complexity.max(analysis.complexity_score);
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            PlanNodeEnum::Project(n) => {
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            PlanNodeEnum::Aggregate(n) => {
                // 聚合函数的复杂度由输入决定
                max_complexity = max_complexity.max(self.get_max_complexity(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)));
            }
            _ => {}
        }

        max_complexity
    }

    /// 估算结果集行数
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
                crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
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
                let input_rows = self.estimate_result_rows(crate::query::planner::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n));
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
        // 每次引用都要重新计算
        (reference_count as f64) * (rows as f64) * 0.1
    }

    /// 估算物化代价
    fn estimate_materialize_cost(&self, rows: u64, complexity: u32) -> f64 {
        // 物化代价 = 计算代价 + 存储代价
        let compute_cost = (rows as f64) * 0.1;
        let storage_cost = (rows as f64) * 0.05; // 存储成本
        let complexity_overhead = (complexity as f64) * 0.01;

        compute_cost + storage_cost + complexity_overhead
    }

    /// 执行物化转换
    ///
    /// # 参数
    /// - `cte_node`: CTE 子计划根节点
    ///
    /// # 返回
    /// 包装了 MaterializeNode 的节点
    pub fn materialize(
        &self,
        cte_node: PlanNodeEnum,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        let materialize_node = MaterializeNode::new(cte_node)?;
        Ok(PlanNodeEnum::Materialize(materialize_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // 简单表达式是确定性的
        let simple_expr = Expression::Literal(crate::core::Value::Int(42));
        let ctx = std::sync::Arc::new(ExpressionAnalysisContext::new());
        let meta = crate::core::types::expression::ExpressionMeta::new(simple_expr);
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
        let meta2 = crate::core::types::expression::ExpressionMeta::new(nondet_expr);
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

        // 测试代价估算
        let recompute_cost = _optimizer.estimate_recompute_cost(3, 1000);
        let materialize_cost = _optimizer.estimate_materialize_cost(1000, 50);

        assert!(recompute_cost > 0.0);
        assert!(materialize_cost > 0.0);
    }
}
