//! 连接节点实现
//! 
//! 包含各种连接节点类型，如内连接、左连接等

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable
};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use crate::query::parser::ast::expr::Expr;
use std::sync::Arc;


/// 内连接节点
/// 
/// 根据指定的连接键对两个输入进行内连接
#[derive(Debug, Clone)]
pub struct InnerJoinNode {
    id: i64,
    left: Arc<dyn PlanNode>,
    right: Arc<dyn PlanNode>,
    hash_keys: Vec<Expr>,
    probe_keys: Vec<Expr>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    // 内部存储的依赖向量，用于快速访问
    inner_deps: Vec<Arc<dyn PlanNode>>,
}

impl InnerJoinNode {
    /// 创建新的内连接节点
    pub fn new(
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        hash_keys: Vec<Expr>,
        probe_keys: Vec<Expr>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());
        
        let inner_deps = vec![left.clone(), right.clone()];
        
        Ok(Self {
            id: -1,
            left,
            right,
            hash_keys,
            probe_keys,
            output_var: None,
            col_names,
            cost: 0.0,
            inner_deps,
        })
    }
    
    /// 获取哈希键
    pub fn hash_keys(&self) -> &[Expr] {
        &self.hash_keys
    }
    
    /// 获取探测键
    pub fn probe_keys(&self) -> &[Expr] {
        &self.probe_keys
    }
}

impl PlanNodeIdentifiable for InnerJoinNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::HashInnerJoin }
}

impl PlanNodeProperties for InnerJoinNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for InnerJoinNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.inner_deps
    }

    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 内连接节点不支持添加依赖，它需要恰好两个输入
        // 在实际使用中，内连接节点在创建时就确定了依赖
        panic!("内连接节点不支持添加依赖，它需要恰好两个输入")
    }
}

impl PlanNodeMutable for InnerJoinNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for InnerJoinNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            left: self.left.clone_plan_node(),
            right: self.right.clone_plan_node(),
            hash_keys: self.hash_keys.clone(),
            probe_keys: self.probe_keys.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            inner_deps: self.inner_deps.clone(),
        })
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            left: self.left.clone_plan_node(),
            right: self.right.clone_plan_node(),
            hash_keys: self.hash_keys.clone(),
            probe_keys: self.probe_keys.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            inner_deps: self.inner_deps.clone(),
        })
    }
}

impl PlanNodeVisitable for InnerJoinNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_inner_join_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for InnerJoinNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_inner_join_node_creation() {
        let left_node = super::start_node::StartNode::new();
        let right_node = super::start_node::StartNode::new();
        let left_node = Arc::new(left_node);
        let right_node = Arc::new(right_node);
        
        let hash_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        let probe_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        
        let join_node = InnerJoinNode::new(
            left_node,
            right_node,
            hash_keys,
            probe_keys,
        ).unwrap();
        
        assert_eq!(join_node.kind(), PlanNodeKind::HashInnerJoin);
        assert_eq!(join_node.dependencies().len(), 2);
        assert_eq!(join_node.hash_keys().len(), 1);
        assert_eq!(join_node.probe_keys().len(), 1);
    }
    
    #[test]
    fn test_inner_join_node_dependencies() {
        let left_node = super::start_node::StartNode::new();
        let right_node = super::start_node::StartNode::new();
        let left_node = Arc::new(left_node);
        let right_node = Arc::new(right_node);
        
        let hash_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        let probe_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        
        let mut join_node = InnerJoinNode::new(
            left_node.clone(),
            right_node.clone(),
            hash_keys,
            probe_keys,
        ).unwrap();
        
        // 测试依赖管理
        assert_eq!(join_node.dependency_count(), 2);
        assert!(join_node.has_dependency(left_node.id()));
        assert!(join_node.has_dependency(right_node.id()));
        
        // 测试替换依赖
        let new_left_node = super::start_node::StartNode::new();
        let new_right_node = super::start_node::StartNode::new();
        let new_left_node = Arc::new(new_left_node);
        let new_right_node = Arc::new(new_right_node);
        
        join_node.replace_dependencies(vec![new_left_node.clone(), new_right_node.clone()]);
        
        assert_eq!(join_node.dependency_count(), 2);
        assert!(join_node.has_dependency(new_left_node.id()));
        assert!(join_node.has_dependency(new_right_node.id()));
    }
}