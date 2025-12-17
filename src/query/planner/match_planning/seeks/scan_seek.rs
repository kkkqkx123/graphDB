use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::PlanNodeKind;
//! 扫描查找规划器
//! 进行全表扫描操作的规划
//! 负责规划全表扫描操作

use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use std::sync::Arc;

/// 扫描查找规划器
/// 负责规划全表扫描操作
#[derive(Debug)]
pub struct ScanSeek {
    node_info: NodeInfo,
}

impl ScanSeek {
    pub fn new(node_info: NodeInfo) -> Self {
        Self { node_info }
    }

    /// 构建扫描查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建扫描顶点节点
        let scan_vertices_node = Arc::new(PlanNodeFactory::create_placeholder_node()??,
        ));

        // TODO: 设置扫描条件
        // 这里需要根据node_info设置扫描条件，如标签过滤等

        Ok(SubPlan::new(
            Some(scan_vertices_node.clone()),
            Some(scan_vertices_node),
        ))
    }

    /// 检查是否可以使用扫描查找
    pub fn match_node(&self) -> bool {
        // 扫描查找总是可用的，作为最后的备选方案
        true
    }

    /// 获取扫描成本估计
    pub fn estimate_cost(&self) -> f64 {
        // 全表扫描的成本较高
        1000.0
    }
}

/// 创建起始节点
fn create_start_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::core::nodes::PlanNodeFactory;

    Ok(PlanNodeFactory::create_start_node()?)
}
