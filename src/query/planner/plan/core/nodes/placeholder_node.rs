//! 占位符节点实现
//! 
//! PlaceholderNode 用于表示计划中的占位符，通常用于参数化查询

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt, PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 占位符节点
/// 
/// 表示计划中的占位符，通常用于参数化查询
#[derive(Debug, Clone)]
pub struct PlaceholderNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies_vec: Vec<Arc<dyn PlanNode>>, // 添加依赖向量
}

impl PlaceholderNode {
    /// 创建新的占位符节点
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

impl PlanNodeIdentifiable for PlaceholderNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Argument }
}

impl PlanNodeProperties for PlaceholderNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for PlaceholderNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.dependencies_vec.clone()
    }

    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 占位符节点不支持依赖
        panic!("占位符节点不支持依赖")
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        // 占位符节点没有依赖，所以无法移除依赖
        false
    }
}

impl PlanNodeDependenciesExt for PlaceholderNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.dependencies_vec)
    }
}

impl PlanNodeMutable for PlaceholderNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for PlaceholderNode {
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

impl PlanNodeVisitable for PlaceholderNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_placeholder(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for PlaceholderNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_placeholder_node_creation() {
        let placeholder_node = PlaceholderNode::new();
        
        assert_eq!(placeholder_node.kind(), PlanNodeKind::Argument);
        assert_eq!(placeholder_node.dependencies().len(), 0);
        assert_eq!(placeholder_node.col_names().len(), 0);
    }
    
    #[test]
    fn test_placeholder_node_dependencies() {
        let placeholder_node = PlaceholderNode::new();
        
        assert_eq!(placeholder_node.dependency_count(), 0);
        assert!(!placeholder_node.has_dependency(1));
        assert!(!placeholder_node.has_dependency(2));
    }
    
    #[test]
    fn test_placeholder_node_mutable() {
        let mut placeholder_node = PlaceholderNode::new();
        
        // 测试设置属性
        placeholder_node.set_col_names(vec!["param".to_string()]);
        assert_eq!(placeholder_node.col_names().len(), 1);
        assert_eq!(placeholder_node.col_names()[0], "param");
    }
}