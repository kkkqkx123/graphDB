//! 代价赋值器模块
//!
//! 为执行计划中的所有节点计算代价（仅用于优化决策，不存储到节点中）
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::cost::CostAssigner;
//! use graphdb::query::optimizer::stats::StatisticsManager;
//! use graphdb::query::planner::plan::ExecutionPlan;
//! use std::sync::Arc;
//!
//! let stats_manager = Arc::new(StatisticsManager::new());
//! let assigner = CostAssigner::new(stats_manager);
//!
//! // 为执行计划计算代价（仅用于优化决策）
//! // let total_cost = assigner.assign_costs(&mut plan)?;
//! ```
//!
//! ## 架构说明
//!
//! 代价计算完全隔离在优化器层，不再存储到 PlanNode 中。
//! 代价仅用于优化决策（如索引选择、连接算法选择等），
//! 执行阶段不需要代价信息。

use std::sync::Arc;

use crate::core::error::optimize::{CostError, CostResult};
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};

use super::{
    CostCalculator, CostModelConfig, SelectivityEstimator,
    estimate::NodeCostEstimate,
    child_accessor::ChildAccessor,
    node_estimators::{
        NodeEstimator,
        ScanEstimator, GraphTraversalEstimator, JoinEstimator,
        SortLimitEstimator, SetOperationEstimator, ControlFlowEstimator,
        GraphAlgorithmEstimator, DataProcessingEstimator,
    },
};

/// 代价赋值器
///
/// 为执行计划中的所有节点计算并设置代价
#[derive(Debug, Clone)]
pub struct CostAssigner {
    cost_calculator: CostCalculator,
    selectivity_estimator: SelectivityEstimator,
    config: CostModelConfig,
}

impl CostAssigner {
    /// 创建新的代价赋值器（使用默认配置）
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self {
            cost_calculator: CostCalculator::new(stats_manager.clone()),
            selectivity_estimator: SelectivityEstimator::new(stats_manager),
            config: CostModelConfig::default(),
        }
    }

    /// 创建新的代价赋值器（使用指定配置）
    pub fn with_config(stats_manager: Arc<StatisticsManager>, config: CostModelConfig) -> Self {
        Self {
            cost_calculator: CostCalculator::with_config(stats_manager.clone(), config),
            selectivity_estimator: SelectivityEstimator::new(stats_manager),
            config,
        }
    }

    /// 获取代价计算器
    pub fn cost_calculator(&self) -> &CostCalculator {
        &self.cost_calculator
    }

    /// 获取选择性估计器
    pub fn selectivity_estimator(&self) -> &SelectivityEstimator {
        &self.selectivity_estimator
    }

    /// 获取配置
    pub fn config(&self) -> &CostModelConfig {
        &self.config
    }

    /// 为整个执行计划赋值代价
    ///
    /// 这会递归遍历计划树，为每个节点计算并设置代价
    pub fn assign_costs(&self, plan: &mut ExecutionPlan) -> CostResult<f64> {
        match plan.root_mut() {
            Some(root) => {
                let estimate = self.assign_node_costs_recursive(root)?;
                Ok(estimate.total_cost)
            }
            None => Ok(0.0),
        }
    }

    /// 为整个执行计划赋值代价并返回详细估算结果
    ///
    /// 返回根节点的代价和行数估算结果
    pub fn assign_costs_with_estimate(&self, plan: &mut ExecutionPlan) -> CostResult<NodeCostEstimate> {
        match plan.root_mut() {
            Some(root) => self.assign_node_costs_recursive(root),
            None => Ok(NodeCostEstimate::new(0.0, 0.0, 0)),
        }
    }

    /// 递归为节点及其子节点赋值代价
    ///
    /// 使用后序遍历：先计算子节点代价，再计算当前节点
    /// 返回包含代价和行数的估算结果
    fn assign_node_costs_recursive(&self, node: &mut PlanNodeEnum) -> CostResult<NodeCostEstimate> {
        // 1. 先递归计算子节点的代价和行数（后序遍历）
        let child_estimates = self.calculate_child_estimates(node)?;

        // 2. 根据节点类型计算自身代价和输出行数
        let estimate = self.calculate_node_estimate(node, &child_estimates)?;

        Ok(estimate)
    }

    /// 计算子节点的代价和行数估算
    fn calculate_child_estimates(&self, node: &mut PlanNodeEnum) -> CostResult<Vec<NodeCostEstimate>> {
        let mut estimates = Vec::new();
        let child_count = node.child_count();

        for i in 0..child_count {
            if let Some(child) = node.get_child_mut(i) {
                let estimate = self.assign_node_costs_recursive(child)?;
                estimates.push(estimate);
            }
        }

        Ok(estimates)
    }

    /// 计算节点的代价和输出行数估算
    fn calculate_node_estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> CostResult<NodeCostEstimate> {
        // 计算子节点的累计代价
        let child_total_cost: f64 = child_estimates.iter().map(|e| e.total_cost).sum();

        // 根据节点类型选择合适的估算器
        let (node_cost, output_rows) = self.estimate_by_node_type(node, child_estimates)?;

        let total_cost = node_cost + child_total_cost;
        Ok(NodeCostEstimate::new(node_cost, total_cost, output_rows))
    }

    /// 根据节点类型选择估算器进行估算
    fn estimate_by_node_type(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> CostResult<(f64, u64)> {
        match node {
            // 扫描操作
            PlanNodeEnum::ScanVertices(_) |
            PlanNodeEnum::ScanEdges(_) |
            PlanNodeEnum::IndexScan(_) |
            PlanNodeEnum::EdgeIndexScan(_) => {
                let estimator = ScanEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 图遍历操作
            PlanNodeEnum::Expand(_) |
            PlanNodeEnum::ExpandAll(_) |
            PlanNodeEnum::Traverse(_) |
            PlanNodeEnum::AppendVertices(_) |
            PlanNodeEnum::GetNeighbors(_) |
            PlanNodeEnum::GetVertices(_) |
            PlanNodeEnum::GetEdges(_) => {
                let estimator = GraphTraversalEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 连接操作
            PlanNodeEnum::HashInnerJoin(_) |
            PlanNodeEnum::HashLeftJoin(_) |
            PlanNodeEnum::InnerJoin(_) |
            PlanNodeEnum::LeftJoin(_) |
            PlanNodeEnum::CrossJoin(_) |
            PlanNodeEnum::FullOuterJoin(_) => {
                let estimator = JoinEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 排序和限制操作
            PlanNodeEnum::Sort(_) |
            PlanNodeEnum::Limit(_) |
            PlanNodeEnum::TopN(_) |
            PlanNodeEnum::Aggregate(_) |
            PlanNodeEnum::Dedup(_) |
            PlanNodeEnum::Sample(_) => {
                let estimator = SortLimitEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 集合操作
            PlanNodeEnum::Union(_) |
            PlanNodeEnum::Minus(_) |
            PlanNodeEnum::Intersect(_) => {
                let estimator = SetOperationEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 控制流节点
            PlanNodeEnum::Loop(_) |
            PlanNodeEnum::Select(_) |
            PlanNodeEnum::PassThrough(_) |
            PlanNodeEnum::Argument(_) => {
                let estimator = ControlFlowEstimator::new(&self.cost_calculator, self.config);
                estimator.estimate(node, child_estimates)
            }

            // 图算法
            PlanNodeEnum::ShortestPath(_) |
            PlanNodeEnum::AllPaths(_) |
            PlanNodeEnum::MultiShortestPath(_) |
            PlanNodeEnum::BFSShortest(_) => {
                let estimator = GraphAlgorithmEstimator::new(&self.cost_calculator);
                estimator.estimate(node, child_estimates)
            }

            // 数据处理
            PlanNodeEnum::Filter(_) |
            PlanNodeEnum::Project(_) |
            PlanNodeEnum::Unwind(_) |
            PlanNodeEnum::DataCollect(_) |
            PlanNodeEnum::Start(_) => {
                let estimator = DataProcessingEstimator::new(
                    &self.cost_calculator,
                    &self.selectivity_estimator,
                    self.config,
                );
                estimator.estimate(node, child_estimates)
            }

            // 其他节点类型
            _ => {
                // 对于未明确处理的节点类型，返回保守估计
                Ok((1.0, 1))
            }
        }
    }
}

impl Default for CostAssigner {
    fn default() -> Self {
        let stats_manager = Arc::new(StatisticsManager::new());
        let config = CostModelConfig::default();
        Self {
            cost_calculator: CostCalculator::with_config(stats_manager.clone(), config),
            selectivity_estimator: SelectivityEstimator::new(stats_manager),
            config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_assigner_creation() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let assigner = CostAssigner::new(stats_manager);
        assert_eq!(assigner.cost_calculator().config().seq_page_cost, 1.0);
    }

    #[test]
    fn test_cost_assigner_with_config() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let config = CostModelConfig::for_ssd();
        let assigner = CostAssigner::with_config(stats_manager, config);
        assert_eq!(assigner.cost_calculator().config().random_page_cost, 1.1);
    }
}
