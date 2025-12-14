//! 属性索引查找规划器
//! 根据属性索引进行查找
//! 负责规划基于属性索引的查找操作

use crate::query::planner::plan::core::{PlanNodeMutable, PlanNodeClonable};
use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use crate::graph::expression::expr_type::Expression;
use std::sync::Arc;

/// 属性索引查找规划器
/// 负责规划基于属性索引的查找操作
#[derive(Debug)]
pub struct PropIndexSeek {
    node_info: NodeInfo,
    prop_exprs: Vec<Expression>, // 属性表达式列表
}

impl PropIndexSeek {
    pub fn new(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            prop_exprs,
        }
    }

    /// 构建属性索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 检查是否有属性表达式可以用来做索引查找
        if self.prop_exprs.is_empty() {
            return Err(PlannerError::UnsupportedOperation(
                "No property expressions for index seek".to_string(),
            ));
        }

        // 创建起始节点
        let start_node = self.create_start_node()?;

        // 创建索引扫描节点
        let index_scan_node = SingleInputNode::new(
            PlanNodeKind::IndexScan,
            start_node,
        );

        // 设置节点列名，包含标签名和其他相关信息
        let mut col_names = vec![self.node_info.alias.clone()];
        for label in &self.node_info.labels {
            col_names.push(format!("label_{}", label));
        }

        // 创建新的索引扫描节点并设置属性
        let mut new_index_scan_node = index_scan_node.clone();
        new_index_scan_node.set_col_names(col_names);
        let index_scan_node = Arc::new(new_index_scan_node);

        let cloned_node = index_scan_node.clone_plan_node();
        Ok(SubPlan::new(Some(index_scan_node), Some(cloned_node)))
    }

    /// 检查是否可以使用属性索引查找
    pub fn match_node(&self) -> bool {
        // 如果节点有属性表达式，可以使用属性索引查找
        !self.prop_exprs.is_empty()
    }

    /// 创建起始节点
    fn create_start_node(&self) -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
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
}