//! 可变顶点ID查找规划器
//! 根据可变的顶点ID进行查找
//! 负责规划基于可变顶点ID的查找操作

use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use crate::graph::expression::expr_type::Expression;
use std::sync::Arc;

/// 可变顶点ID查找规划器
/// 负责规划基于可变顶点ID的查找操作
#[derive(Debug)]
pub struct VariableVertexIdSeek {
    node_info: NodeInfo,
    vid_expr: Expression, // 顶点ID表达式
}

impl VariableVertexIdSeek {
    pub fn new(node_info: NodeInfo, vid_expr: Expression) -> Self {
        Self {
            node_info,
            vid_expr,
        }
    }

    /// 构建可变顶点ID查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建获取顶点节点
        let get_vertices_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::GetVertices,
            create_start_node()?,
        ));

        // TODO: 设置可变顶点ID表达式
        // 这里需要根据vid_expr设置要查找的顶点ID表达式

        Ok(SubPlan::new(Some(get_vertices_node.clone()), Some(get_vertices_node)))
    }

    /// 检查是否可以使用可变顶点ID查找
    pub fn match_node(&self) -> bool {
        // 如果节点有顶点ID表达式，可以使用可变顶点ID查找
        matches!(self.vid_expr, Expression::Label(_) | Expression::Variable(_))
    }
}

/// 创建起始节点
fn create_start_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;
    
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}