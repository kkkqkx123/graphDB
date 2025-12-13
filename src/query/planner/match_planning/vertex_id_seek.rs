//! 顶点ID查找规划器
//! 根据顶点ID进行查找
//! 负责规划基于顶点ID的查找操作

use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use std::sync::Arc;

/// 顶点ID查找规划器
/// 负责规划基于顶点ID的查找操作
#[derive(Debug)]
pub struct VertexIdSeek {
    node_info: NodeInfo,
    vids: Vec<String>, // 顶点ID列表
}

impl VertexIdSeek {
    pub fn new(node_info: NodeInfo, vids: Vec<String>) -> Self {
        Self {
            node_info,
            vids,
        }
    }

    /// 构建顶点ID查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建获取顶点节点
        let get_vertices_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::GetVertices,
            create_start_node()?,
        ));

        // TODO: 设置顶点ID列表
        // 这里需要根据vids设置要查找的顶点ID

        Ok(SubPlan::new(Some(get_vertices_node.clone()), Some(get_vertices_node)))
    }

    /// 检查是否可以使用顶点ID查找
    pub fn match_node(&self) -> bool {
        // 如果节点有特定的ID列表，可以使用顶点ID查找
        !self.vids.is_empty()
    }
}

/// 创建起始节点
fn create_start_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    Ok(Arc::new(SingleInputNode::new(
        PlanNodeKind::Start,
        Arc::new(SingleInputNode::new(PlanNodeKind::Start, create_empty_node()?)),
    )))
}

/// 创建空节点
fn create_empty_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;
    
    // 创建一个空的计划节点作为占位符
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}