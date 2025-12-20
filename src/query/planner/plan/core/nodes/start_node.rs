//! 起始节点实现
//!
//! StartNode 用于表示执行计划的起始点

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
    PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 起始节点
///
/// 表示执行计划的起始点，没有输入依赖
#[derive(Debug, Clone)]
pub struct StartNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies_vec: Vec<Arc<dyn PlanNode>>, // 添加依赖向量
}

impl StartNode {
    /// 创建新的起始节点
    pub fn new() -> Self {
        Self {
            id: -1,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
            dependencies_vec: vec![],
        }
    }
}

impl PlanNodeIdentifiable for StartNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Start
    }
}

impl PlanNodeProperties for StartNode {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }
    fn col_names(&self) -> &[String] {
        &self.col_names
    }
    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for StartNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.dependencies_vec.clone()
    }

    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 起始节点不支持依赖
        panic!("起始节点不支持依赖")
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        // 起始节点没有依赖，所以无法移除依赖
        false
    }
}

impl PlanNodeDependenciesExt for StartNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.dependencies_vec)
    }
}

impl PlanNodeMutable for StartNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for StartNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: vec![],
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: vec![],
        })
    }
}

impl PlanNodeVisitable for StartNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_start_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for StartNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_node_creation() {
        let start_node = StartNode::new();

        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }

    #[test]
    fn test_start_node_dependencies() {
        let start_node = StartNode::new();

        assert_eq!(start_node.dependency_count(), 0);
        assert!(!start_node.has_dependency(1));
        assert!(!start_node.has_dependency(2));
    }

    #[test]
    fn test_start_node_mutable() {
        let mut start_node = StartNode::new();

        // 测试设置属性
        start_node.set_col_names(vec!["test".to_string()]);
        assert_eq!(start_node.col_names().len(), 1);
        assert_eq!(start_node.col_names()[0], "test");
    }
}
