//! 标签索引查找规划器
//! 根据标签索引进行查找
//! 负责规划基于标签索引的查找操作

use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;

/// 标签索引查找规划器
/// 负责规划基于标签索引的查找操作
#[derive(Debug)]
pub struct LabelIndexSeek {
    node_info: NodeInfo,
}

impl LabelIndexSeek {
    pub fn new(node_info: NodeInfo) -> Self {
        Self {
            node_info,
        }
    }

    /// 构建标签索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建索引扫描节点
        let index_scan_node = Box::new(SingleInputNode::new(
            PlanNodeKind::IndexScan,
            create_start_node()?,
        ));

        // TODO: 设置标签索引信息
        // 这里需要根据node_info.labels设置要扫描的标签索引

        Ok(SubPlan::new(Some(index_scan_node.clone()), Some(index_scan_node)))
    }

    /// 检查是否可以使用标签索引查找
    pub fn match_node(&self) -> bool {
        // 如果节点有标签，可以使用标签索引查找
        !self.node_info.labels.is_empty()
    }
}

/// 创建起始节点
fn create_start_node() -> Result<Box<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;
    
    Ok(Box::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}