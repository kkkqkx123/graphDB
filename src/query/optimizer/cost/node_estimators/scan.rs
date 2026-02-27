//! 扫描操作估算器
//!
//! 为扫描节点提供代价估算：
//! - ScanVertices
//! - ScanEdges
//! - IndexScan
//! - EdgeIndexScan

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::algorithms::{IndexScan, ScanType};
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 扫描操作估算器
pub struct ScanEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> ScanEstimator<'a> {
    /// 创建新的扫描估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }

    /// 估算索引扫描的选择性
    pub fn estimate_index_scan_selectivity(&self, node: &IndexScan) -> f64 {
        if node.scan_limits.is_empty() {
            return 0.1;
        }

        let mut total_selectivity: f64 = 1.0;
        for limit in &node.scan_limits {
            let sel = match limit.scan_type {
                ScanType::Unique => 0.01,
                ScanType::Prefix => 0.05,
                ScanType::Range => 0.1,
                ScanType::Full => 1.0,
            };
            total_selectivity *= sel;
        }
        total_selectivity.min(1.0)
    }

    /// 估算边索引扫描的选择性
    pub fn estimate_edge_index_scan_selectivity(
        &self,
        node: &crate::query::planner::plan::core::nodes::graph_scan_node::EdgeIndexScanNode,
    ) -> f64 {
        if node.scan_limits().is_empty() {
            return 0.1;
        }

        let mut total_selectivity: f64 = 1.0;
        for limit in node.scan_limits() {
            let sel = match limit.scan_type {
                ScanType::Unique => 0.01,
                ScanType::Prefix => 0.05,
                ScanType::Range => 0.1,
                ScanType::Full => 1.0,
            };
            total_selectivity *= sel;
        }
        total_selectivity.min(1.0)
    }

    /// 从 IndexScan 节点获取标签名称
    fn get_tag_name_from_index_scan(&self, node: &IndexScan) -> String {
        // 尝试通过 tag_id 获取标签名称
        if let Some(tag_name) = self.cost_calculator.statistics_manager().get_tag_name_by_id(node.tag_id) {
            return tag_name;
        }

        // 回退：尝试从 scan_limits 中的列名推断标签名称
        if let Some(limit) = node.scan_limits.first() {
            let column = &limit.column;
            if let Some(dot_pos) = column.find('.') {
                return column[..dot_pos].to_string();
            }
        }

        "default".to_string()
    }

    /// 从 IndexScan 节点获取属性名称
    fn get_property_name_from_index_scan(&self, node: &IndexScan) -> String {
        if let Some(limit) = node.scan_limits.first() {
            limit.column.clone()
        } else {
            "default".to_string()
        }
    }
}

impl<'a> NodeEstimator for ScanEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        _child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                let tag_name = n.tag().map(|s| s.as_str()).unwrap_or("default");
                let row_count = self.cost_calculator.statistics_manager().get_vertex_count(tag_name);
                let cost = self.cost_calculator.calculate_scan_vertices_cost(tag_name);
                Ok((cost, row_count.max(1)))
            }
            PlanNodeEnum::ScanEdges(n) => {
                let edge_type = n.edge_type().unwrap_or_else(|| "default".to_string());
                let row_count = self.cost_calculator.statistics_manager().get_edge_count(&edge_type);
                let cost = self.cost_calculator.calculate_scan_edges_cost(&edge_type);
                Ok((cost, row_count.max(1)))
            }
            PlanNodeEnum::IndexScan(n) => {
                let selectivity = self.estimate_index_scan_selectivity(n);
                let tag_name = self.get_tag_name_from_index_scan(n);
                let property_name = self.get_property_name_from_index_scan(n);
                let table_rows = self.cost_calculator.statistics_manager().get_vertex_count(&tag_name);
                let output_rows = (selectivity * table_rows as f64).max(1.0) as u64;
                let cost = self.cost_calculator
                    .calculate_index_scan_cost(&tag_name, &property_name, selectivity);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::EdgeIndexScan(n) => {
                let edge_type = n.edge_type();
                let selectivity = self.estimate_edge_index_scan_selectivity(n);
                let edge_count = self.cost_calculator.statistics_manager().get_edge_count(edge_type);
                let output_rows = (selectivity * edge_count as f64).max(1.0) as u64;
                let cost = self.cost_calculator
                    .calculate_edge_index_scan_cost(edge_type, selectivity);
                Ok((cost, output_rows))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("扫描估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
