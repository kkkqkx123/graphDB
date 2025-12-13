//! 可变属性索引查找规划器
//! 根据可变的属性索引进行查找
//! 负责规划基于可变属性索引的查找操作

use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use crate::graph::expression::expr_type::Expression;
use std::sync::Arc;

/// 可变属性索引查找规划器
/// 负责规划基于可变属性索引的查找操作
#[derive(Debug)]
pub struct VariablePropIndexSeek {
    node_info: NodeInfo,
    prop_exprs: Vec<Expression>, // 属性表达式列表
}

impl VariablePropIndexSeek {
    pub fn new(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            prop_exprs,
        }
    }

    /// 构建可变属性索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建索引扫描节点
        let index_scan_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::IndexScan,
            create_start_node()?,
        ));

        // TODO: 设置可变属性索引表达式
        // 这里需要根据prop_exprs设置要扫描的属性索引表达式

        Ok(SubPlan::new(Some(index_scan_node.clone()), Some(index_scan_node)))
    }

    /// 检查是否可以使用可变属性索引查找
    pub fn match_node(&self) -> bool {
        // 如果节点有属性表达式且包含变量，可以使用可变属性索引查找
        !self.prop_exprs.is_empty() && self.prop_exprs.iter().any(|expr| {
            matches!(expr, Expression::Label(_) | Expression::Variable(_))
        })
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