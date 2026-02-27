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

use crate::core::Expression;
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};
use crate::query::planner::plan::core::nodes::plan_node_traits::{MultipleInputNode, SingleInputNode};

use super::{CostCalculator, CostModelConfig, SelectivityEstimator};

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

/// 节点代价和行数估算结果
#[derive(Debug, Clone, Copy)]
pub struct NodeCostEstimate {
    /// 节点自身代价（不包含子节点）
    pub node_cost: f64,
    /// 累计代价（包含所有子节点）
    pub total_cost: f64,
    /// 估算的输出行数
    pub output_rows: u64,
}

impl NodeCostEstimate {
    /// 创建新的估算结果
    pub fn new(node_cost: f64, total_cost: f64, output_rows: u64) -> Self {
        Self {
            node_cost,
            total_cost,
            output_rows,
        }
    }

    /// 创建叶子节点的估算结果（无子节点）
    pub fn leaf(node_cost: f64, output_rows: u64) -> Self {
        Self {
            node_cost,
            total_cost: node_cost,
            output_rows,
        }
    }
}

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
            // ==================== 双输入节点 ====================
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

            // ==================== 单输入节点 ====================
            PlanNodeEnum::Project(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Filter(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Sort(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Limit(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::TopN(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Sample(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Dedup(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::DataCollect(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Aggregate(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Unwind(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Assign(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::PatternApply(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::RollUpApply(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Traverse(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Union(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }
            PlanNodeEnum::Minus(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }
            PlanNodeEnum::Intersect(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }

            // ==================== 多输入节点 ====================
            PlanNodeEnum::Expand(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::ExpandAll(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::AppendVertices(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::GetVertices(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::GetNeighbors(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),

            // ==================== 控制流节点 ====================
            PlanNodeEnum::Loop(n) => {
                if index == 0 { n.body_mut().as_mut().map(|b| b.as_mut()) } else { None }
            }
            PlanNodeEnum::Select(n) => match index {
                0 => n.if_branch_mut().as_mut().map(|b| b.as_mut()),
                1 => n.else_branch_mut().as_mut().map(|b| b.as_mut()),
                _ => None,
            },

            // ==================== 无输入节点 ====================
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
            PlanNodeEnum::ScanVertices(n) => {
                let tag_name = n.tag().map(|s| s.as_str()).unwrap_or("default");
                self.cost_calculator.calculate_scan_vertices_cost(tag_name)
            }
            PlanNodeEnum::ScanEdges(n) => {
                let edge_type = n.edge_type().unwrap_or_else(|| "default".to_string());
                self.cost_calculator.calculate_scan_edges_cost(&edge_type)
            }
            PlanNodeEnum::IndexScan(n) => {
                let selectivity = self.estimate_index_scan_selectivity(n);
                let tag_name = self.get_tag_name_from_index_scan(n);
                let property_name = self.get_property_name_from_index_scan(n);
                self.cost_calculator
                    .calculate_index_scan_cost(&tag_name, &property_name, selectivity)
            }
            PlanNodeEnum::EdgeIndexScan(n) => {
                let edge_type = n.edge_type();
                let selectivity = self.estimate_edge_index_scan_selectivity(n);
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
            PlanNodeEnum::GetNeighbors(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                self.cost_calculator
                    .calculate_get_neighbors_cost(input_rows, edge_type)
            }
            PlanNodeEnum::GetVertices(n) => {
                let vid_count = n.limit().unwrap_or(100) as u64;
                self.cost_calculator.calculate_get_vertices_cost(vid_count)
            }
            PlanNodeEnum::GetEdges(n) => {
                let edge_count = n.limit().unwrap_or(100) as u64;
                self.cost_calculator.calculate_get_edges_cost(edge_count)
            }

            // ==================== 过滤和投影 ====================
            PlanNodeEnum::Filter(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let condition_count = self.count_filter_conditions(n.condition());
                self.cost_calculator.calculate_filter_cost(input_rows, condition_count)
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
            PlanNodeEnum::Unwind(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let list_size = self.estimate_unwind_list_size(n);
                self.cost_calculator.calculate_unwind_cost(input_rows, list_size)
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
            PlanNodeEnum::Loop(n) => {
                let body_cost = child_costs.first().copied().unwrap_or(0.0);
                let iterations = self.estimate_loop_iterations(n);
                self.cost_calculator.calculate_loop_cost(body_cost, iterations)
            }
            PlanNodeEnum::Select(n) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                let branch_count = self.estimate_select_branch_count(n);
                self.cost_calculator.calculate_select_cost(input_rows, branch_count)
            }
            PlanNodeEnum::PassThrough(_) => {
                let input_rows = self.estimate_input_rows(child_costs, 0);
                self.cost_calculator.calculate_pass_through_cost(input_rows)
            }
            PlanNodeEnum::Argument(_) => 0.0,

            // ==================== 图算法 ====================
            PlanNodeEnum::ShortestPath(n) => {
                let max_depth = n.max_step() as u32;
                self.cost_calculator.calculate_shortest_path_cost(1, max_depth)
            }
            PlanNodeEnum::AllPaths(n) => {
                let max_depth = n.max_hop() as u32;
                self.cost_calculator.calculate_all_paths_cost(1, max_depth)
            }
            PlanNodeEnum::MultiShortestPath(n) => {
                let max_depth = n.steps() as u32;
                self.cost_calculator.calculate_multi_shortest_path_cost(2, max_depth)
            }
            PlanNodeEnum::BFSShortest(n) => {
                let max_depth = n.steps() as u32;
                self.cost_calculator.calculate_shortest_path_cost(1, max_depth)
            }

            // ==================== 起始节点 ====================
            PlanNodeEnum::Start(_) => 0.0,

            // ==================== 管理节点 ====================
            _ => 1.0,
        };

        Ok(cost)
    }

    /// 估算输入行数
    fn estimate_input_rows(&self, child_costs: &[f64], index: usize) -> u64 {
        child_costs
            .get(index)
            .copied()
            .map(|c| c.max(1.0) as u64)
            .unwrap_or(1)
    }

    /// 估算索引扫描的选择性
    fn estimate_index_scan_selectivity(&self, node: &crate::query::planner::plan::algorithms::IndexScan) -> f64 {
        if node.scan_limits.is_empty() {
            return 0.1;
        }

        let mut total_selectivity: f64 = 1.0;
        for limit in &node.scan_limits {
            let sel = match limit.scan_type {
                crate::query::planner::plan::algorithms::ScanType::Unique => 0.01,
                crate::query::planner::plan::algorithms::ScanType::Prefix => 0.05,
                crate::query::planner::plan::algorithms::ScanType::Range => 0.1,
                crate::query::planner::plan::algorithms::ScanType::Full => 1.0,
            };
            total_selectivity *= sel;
        }
        total_selectivity.min(1.0)
    }

    /// 估算边索引扫描的选择性
    fn estimate_edge_index_scan_selectivity(&self, node: &crate::query::planner::plan::core::nodes::graph_scan_node::EdgeIndexScanNode) -> f64 {
        if node.scan_limits().is_empty() {
            return 0.1;
        }

        let mut total_selectivity: f64 = 1.0;
        for limit in node.scan_limits() {
            let sel = match limit.scan_type {
                crate::query::planner::plan::algorithms::ScanType::Unique => 0.01,
                crate::query::planner::plan::algorithms::ScanType::Prefix => 0.05,
                crate::query::planner::plan::algorithms::ScanType::Range => 0.1,
                crate::query::planner::plan::algorithms::ScanType::Full => 1.0,
            };
            total_selectivity *= sel;
        }
        total_selectivity.min(1.0)
    }

    /// 从 IndexScan 节点获取标签名称
    ///
    /// 首先尝试通过 tag_id 从统计信息管理器中查找标签名称，
    /// 如果找不到则返回 "default" 作为回退值
    fn get_tag_name_from_index_scan(&self, node: &crate::query::planner::plan::algorithms::IndexScan) -> String {
        // 尝试通过 tag_id 获取标签名称
        if let Some(tag_name) = self.cost_calculator.statistics_manager().get_tag_name_by_id(node.tag_id) {
            return tag_name;
        }

        // 回退：尝试从 scan_limits 中的列名推断标签名称
        // 例如：如果列名是 "Person.name"，则标签可能是 "Person"
        if let Some(limit) = node.scan_limits.first() {
            let column = &limit.column;
            if let Some(dot_pos) = column.find('.') {
                return column[..dot_pos].to_string();
            }
        }

        "default".to_string()
    }

    /// 从 IndexScan 节点获取属性名称
    fn get_property_name_from_index_scan(&self, node: &crate::query::planner::plan::algorithms::IndexScan) -> String {
        if let Some(limit) = node.scan_limits.first() {
            limit.column.clone()
        } else {
            "default".to_string()
        }
    }

    /// 计算过滤条件数量
    fn count_filter_conditions(&self, condition: &Expression) -> usize {
        match condition {
            Expression::Binary { op, left, right } => {
                use crate::core::types::BinaryOperator;
                match op {
                    BinaryOperator::And => {
                        self.count_filter_conditions(left) + self.count_filter_conditions(right)
                    }
                    BinaryOperator::Or => {
                        (self.count_filter_conditions(left) + self.count_filter_conditions(right)).max(1)
                    }
                    _ => 1,
                }
            }
            Expression::Unary { .. } => 1,
            Expression::Function { args, .. } => {
                args.iter().map(|_| 1).sum::<usize>().max(1)
            }
            _ => 1,
        }
    }

    /// 估算 Unwind 节点的列表大小
    ///
    /// 尝试从列表表达式中推断大小，如果无法推断则使用配置默认值
    fn estimate_unwind_list_size(&self, node: &crate::query::planner::plan::core::nodes::data_processing_node::UnwindNode) -> f64 {
        let list_expr = node.list_expression();
        
        // 尝试解析表达式推断列表大小
        // 例如：range(1, 10) -> 9, [1,2,3] -> 3
        if let Some(size) = self.try_parse_list_size(list_expr) {
            return size;
        }
        
        // 使用配置默认值
        self.config.default_unwind_list_size
    }

    /// 尝试从表达式字符串解析列表大小
    fn try_parse_list_size(&self, expr: &str) -> Option<f64> {
        let expr = expr.trim();
        
        // 尝试解析 range(start, end) 或 range(start, end, step)
        if expr.starts_with("range(") && expr.ends_with(')') {
            let args_str = &expr[6..expr.len()-1];
            let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
            
            if args.len() >= 2 {
                let start: i64 = args[0].parse().ok()?;
                let end: i64 = args[1].parse().ok()?;
                let step: i64 = if args.len() >= 3 {
                    args[2].parse().ok()?
                } else {
                    1
                };
                
                if step != 0 {
                    let count = ((end - start) / step).abs() as f64;
                    return Some(count.max(0.0));
                }
            }
        }
        
        // 尝试解析数组字面量 [a, b, c]
        if expr.starts_with('[') && expr.ends_with(']') {
            let inner = &expr[1..expr.len()-1];
            if inner.is_empty() {
                return Some(0.0);
            }
            let count = inner.split(',').count() as f64;
            return Some(count);
        }
        
        None
    }

    /// 估算 Loop 节点的迭代次数
    ///
    /// 尝试从条件中推断迭代次数，如果无法推断则使用配置默认值
    fn estimate_loop_iterations(&self, _node: &crate::query::planner::plan::core::nodes::control_flow_node::LoopNode) -> u32 {
        // 当前无法从条件字符串可靠推断迭代次数
        // 未来可以尝试解析条件表达式，例如 "i < 10" -> 10
        self.config.default_loop_iterations
    }

    /// 估算 Select 节点的分支数
    ///
    /// 根据实际分支情况计算分支数
    fn estimate_select_branch_count(&self, node: &crate::query::planner::plan::core::nodes::control_flow_node::SelectNode) -> usize {
        let mut count = 0;
        if node.if_branch().is_some() {
            count += 1;
        }
        if node.else_branch().is_some() {
            count += 1;
        }
        
        if count == 0 {
            self.config.default_select_branches
        } else {
            count
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
