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

use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};
use crate::query::planner::plan::core::nodes::plan_node_traits::MultipleInputNode;

use super::{CostCalculator, CostModelConfig};

/// 代价赋值错误
#[derive(Debug, Clone)]
pub enum CostError {
    /// 不支持的节点类型
    UnsupportedNodeType(String),
    /// 缺少统计信息
    MissingStatistics(String),
    /// 计算错误
    CalculationError(String),
}

impl std::fmt::Display for CostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CostError::UnsupportedNodeType(node) => {
                write!(f, "不支持的节点类型: {}", node)
            }
            CostError::MissingStatistics(msg) => write!(f, "缺少统计信息: {}", msg),
            CostError::CalculationError(msg) => write!(f, "计算错误: {}", msg),
        }
    }
}

impl std::error::Error for CostError {}

/// 代价赋值器
///
/// 为执行计划中的所有节点计算并设置代价
#[derive(Debug, Clone)]
pub struct CostAssigner {
    cost_calculator: CostCalculator,
}

impl CostAssigner {
    /// 创建新的代价赋值器（使用默认配置）
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self {
            cost_calculator: CostCalculator::new(stats_manager),
        }
    }

    /// 创建新的代价赋值器（使用指定配置）
    pub fn with_config(stats_manager: Arc<StatisticsManager>, config: CostModelConfig) -> Self {
        Self {
            cost_calculator: CostCalculator::with_config(stats_manager, config),
        }
    }

    /// 获取代价计算器
    pub fn cost_calculator(&self) -> &CostCalculator {
        &self.cost_calculator
    }

    /// 为整个执行计划赋值代价
    ///
    /// 这会递归遍历计划树，为每个节点计算并设置代价
    pub fn assign_costs(&self, plan: &mut ExecutionPlan) -> Result<f64, CostError> {
        match plan.root_mut() {
            Some(root) => {
                let total_cost = self.assign_node_costs_recursive(root)?;
                Ok(total_cost)
            }
            None => Ok(0.0),
        }
    }

    /// 递归为节点及其子节点赋值代价
    ///
    /// 使用后序遍历：先计算子节点代价，再计算当前节点
    /// 注意：代价不再存储在节点中，仅用于优化决策
    fn assign_node_costs_recursive(&self, node: &mut PlanNodeEnum) -> Result<f64, CostError> {
        // 1. 先递归计算子节点的代价（后序遍历）
        let child_costs = self.calculate_child_costs(node)?;

        // 2. 根据节点类型计算自身代价
        let node_cost = self.calculate_node_cost(node, &child_costs)?;

        // 3. 返回累计代价（节点自身代价 + 子节点代价）
        // 注意：代价不再设置到节点中，仅用于优化决策
        let total_cost = node_cost + child_costs.iter().sum::<f64>();
        Ok(total_cost)
    }

    /// 计算子节点代价
    fn calculate_child_costs(&self, node: &mut PlanNodeEnum) -> Result<Vec<f64>, CostError> {
        let mut costs = Vec::new();

        // 获取子节点并递归计算
        // 注意：这里我们需要获取可变引用来递归计算
        // 由于 children() 返回不可变引用，我们需要使用 dependencies() 和手动遍历
        let child_count = self.get_child_count(node);

        for i in 0..child_count {
            if let Some(child) = self.get_child_mut(node, i) {
                let cost = self.assign_node_costs_recursive(child)?;
                costs.push(cost);
            }
        }

        Ok(costs)
    }

    /// 获取子节点数量
    fn get_child_count(&self, node: &PlanNodeEnum) -> usize {
        node.children().len()
    }

    /// 获取可变子节点引用
    fn get_child_mut<'a>(&self, node: &'a mut PlanNodeEnum, index: usize) -> Option<&'a mut PlanNodeEnum> {
        match node {
            // 双输入节点
            PlanNodeEnum::InnerJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::LeftJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::CrossJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::HashInnerJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::HashLeftJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::FullOuterJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },

            // 多输入节点 - 需要特殊处理
            PlanNodeEnum::Expand(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::ExpandAll(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::AppendVertices(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),

            // 无输入节点和单输入节点 - 不支持可变访问
            _ => None,
        }
    }

    /// 计算节点的代价
    fn calculate_node_cost(
        &self,
        node: &PlanNodeEnum,
        child_costs: &[f64],
    ) -> Result<f64, CostError> {
        let cost = match node {
            // ==================== 扫描操作 ====================
            PlanNodeEnum::ScanVertices(_) => {
                // 从节点中提取标签信息（如果有）
                // 简化处理：使用默认标签或从统计信息中推断
                self.cost_calculator.calculate_scan_vertices_cost("default")
            }
            PlanNodeEnum::ScanEdges(_) => {
                self.cost_calculator.calculate_scan_edges_cost("default")
            }
            PlanNodeEnum::IndexScan(_n) => {
                // 从索引扫描节点提取信息
                let tag_name = "default";
                let property_name = "default";
                let selectivity = 0.1; // 默认选择性
                self.cost_calculator
                    .calculate_index_scan_cost(tag_name, property_name, selectivity)
            }
            PlanNodeEnum::EdgeIndexScan(_n) => {
                let edge_type = "default";
                let selectivity = 0.1;
                self.cost_calculator
                    .calculate_edge_index_scan_cost(edge_type, selectivity)
            }

            // ==================== 图遍历操作 ====================
            PlanNodeEnum::Expand(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                self.cost_calculator.calculate_expand_cost(input_rows, edge_type)
            }
            PlanNodeEnum::ExpandAll(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                self.cost_calculator
                    .calculate_expand_all_cost(input_rows, edge_type)
            }
            PlanNodeEnum::Traverse(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                let steps = n.max_steps();
                self.cost_calculator
                    .calculate_traverse_cost(input_rows, edge_type, steps)
            }
            PlanNodeEnum::AppendVertices(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_append_vertices_cost(input_rows)
            }
            PlanNodeEnum::GetNeighbors(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator
                    .calculate_get_neighbors_cost(input_rows, None)
            }
            PlanNodeEnum::GetVertices(_) => {
                // 假设获取少量顶点
                self.cost_calculator.calculate_get_vertices_cost(10)
            }
            PlanNodeEnum::GetEdges(_) => {
                self.cost_calculator.calculate_get_edges_cost(10)
            }

            // ==================== 过滤和投影 ====================
            PlanNodeEnum::Filter(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                // 简化：假设平均 2 个条件
                self.cost_calculator.calculate_filter_cost(input_rows, 2)
            }
            PlanNodeEnum::Project(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let columns = n.columns().len();
                self.cost_calculator.calculate_project_cost(input_rows, columns)
            }

            // ==================== 连接操作 ====================
            PlanNodeEnum::HashInnerJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_hash_join_cost(left_rows, right_rows)
            }
            PlanNodeEnum::HashLeftJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator
                    .calculate_hash_left_join_cost(left_rows, right_rows)
            }
            PlanNodeEnum::InnerJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_inner_join_cost(left_rows, right_rows)
            }
            PlanNodeEnum::LeftJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_left_join_cost(left_rows, right_rows)
            }
            PlanNodeEnum::CrossJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_cross_join_cost(left_rows, right_rows)
            }
            PlanNodeEnum::FullOuterJoin(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator
                    .calculate_full_outer_join_cost(left_rows, right_rows)
            }

            // ==================== 排序和聚合 ====================
            PlanNodeEnum::Sort(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let sort_columns = n.sort_items().len();
                self.cost_calculator.calculate_sort_cost(input_rows, sort_columns)
            }
            PlanNodeEnum::Limit(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let limit = n.count();
                self.cost_calculator.calculate_limit_cost(input_rows, limit)
            }
            PlanNodeEnum::TopN(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let limit = n.limit();
                self.cost_calculator.calculate_topn_cost(input_rows, limit)
            }
            PlanNodeEnum::Aggregate(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let agg_funcs = n.aggregation_functions().len();
                self.cost_calculator.calculate_aggregate_cost(input_rows, agg_funcs)
            }
            PlanNodeEnum::Dedup(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_dedup_cost(input_rows)
            }

            // ==================== 数据处理和集合操作 ====================
            PlanNodeEnum::Union(n) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_union_cost(left_rows, right_rows, n.distinct())
            }
            PlanNodeEnum::Minus(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_minus_cost(left_rows, right_rows)
            }
            PlanNodeEnum::Intersect(_) => {
                let left_rows = self.estimate_input_rows(child_costs, 0);
                let right_rows = self.estimate_input_rows(child_costs, 1);
                self.cost_calculator.calculate_intersect_cost(left_rows, right_rows)
            }
            PlanNodeEnum::Unwind(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                // 假设平均列表大小为 3
                self.cost_calculator.calculate_unwind_cost(input_rows, 3.0)
            }
            PlanNodeEnum::DataCollect(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_data_collect_cost(input_rows)
            }
            PlanNodeEnum::Sample(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_sample_cost(input_rows)
            }

            // ==================== 控制流节点 ====================
            PlanNodeEnum::Loop(_) => {
                let body_cost = child_costs.first().copied().unwrap_or(0.0);
                // 假设平均 3 次迭代
                self.cost_calculator.calculate_loop_cost(body_cost, 3)
            }
            PlanNodeEnum::Select(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                // 假设 2 个分支
                self.cost_calculator.calculate_select_cost(input_rows, 2)
            }
            PlanNodeEnum::PassThrough(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_pass_through_cost(input_rows)
            }
            PlanNodeEnum::Argument(_) => 0.0,

            // ==================== 图算法 ====================
            PlanNodeEnum::ShortestPath(_) => {
                // 假设从少量节点开始，最大深度为 5
                self.cost_calculator.calculate_shortest_path_cost(1, 5)
            }
            PlanNodeEnum::AllPaths(_) => {
                self.cost_calculator.calculate_all_paths_cost(1, 5)
            }
            PlanNodeEnum::MultiShortestPath(_) => {
                self.cost_calculator.calculate_multi_shortest_path_cost(2, 5)
            }
            PlanNodeEnum::BFSShortest(_) => {
                // BFS 最短路径与标准最短路径类似
                self.cost_calculator.calculate_shortest_path_cost(1, 5)
            }

            // ==================== 起始节点 ====================
            PlanNodeEnum::Start(_) => 0.0,

            // ==================== 管理节点 ====================
            // 管理节点（DDL/DML）通常代价较低或不在查询优化范围内
            _ => 1.0,
        };

        Ok(cost)
    }

    /// 估算输入行数
    fn estimate_input_rows(&self, child_costs: &[f64], index: usize) -> u64 {
        // 简化估算：从子节点代价反推行数
        // 实际实现中应该使用更准确的行数估算
        child_costs
            .get(index)
            .copied()
            .map(|c| c.max(1.0) as u64)
            .unwrap_or(1)
    }
}

impl Default for CostAssigner {
    fn default() -> Self {
        Self {
            cost_calculator: CostCalculator::default(),
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
