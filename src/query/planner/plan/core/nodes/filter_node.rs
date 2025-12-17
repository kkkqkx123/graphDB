//! 过滤节点实现
//! 
//! FilterNode 用于根据指定的条件过滤输入数据流

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable, SingleInputPlanNode
};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use crate::query::parser::ast::expr::Expr;
use std::sync::Arc;


/// 过滤节点
/// 
/// 根据指定的条件表达式过滤输入数据流
#[derive(Debug, Clone)]
pub struct FilterNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    condition: Expr,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        condition: Expr,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        
        Ok(Self {
            id: -1,  // 将在后续分配
            input,
            condition,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }
    
    /// 获取过滤条件
    pub fn condition(&self) -> &Expr {
        &self.condition
    }
}

impl PlanNodeIdentifiable for FilterNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Filter }
}

impl PlanNodeProperties for FilterNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for FilterNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { 
        std::slice::from_ref(&self.input) 
    }
    
    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        // 过滤节点只支持单个输入，取第一个
        if let Some(dep) = deps.into_iter().next() {
            self.input = dep;
        }
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        // 过滤节点只支持单个输入，替换现有输入
        self.input = dep;
    }
    
    fn remove_dependency(&mut self, id: i64) -> bool {
        if self.input.id() == id {
            // 不能移除唯一的输入，返回 false
            false
        } else {
            false
        }
    }
    
    fn clear_dependencies(&mut self) {
        // 过滤节点必须有输入，不能清空
        // 这里可以选择创建一个默认的起始节点
        // 但为了安全，我们选择不做任何操作
    }
}

impl PlanNodeMutable for FilterNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
}

impl PlanNodeClonable for FilterNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            condition: self.condition.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for FilterNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_filter_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for FilterNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SingleInputPlanNode for FilterNode {
    fn input(&self) -> &Arc<dyn PlanNode> {
        &self.input
    }
    
    fn set_input(&mut self, input: Arc<dyn PlanNode>) {
        self.input = input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_filter_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);
        
        let condition = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let filter_node = FilterNode::new(start_node, condition).unwrap();
        
        assert_eq!(filter_node.kind(), PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }
    
    #[test]
    fn test_filter_node_dependencies() {
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);
        
        let condition = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let mut filter_node = FilterNode::new(start_node, condition).unwrap();
        
        // 测试依赖管理
        assert_eq!(filter_node.dependency_count(), 1);
        assert!(filter_node.has_dependency(start_node.id()));
        
        // 测试替换依赖
        let new_start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let new_start_node = Arc::new(new_start_node);
        filter_node.replace_dependencies(vec![new_start_node.clone()]);
        
        assert_eq!(filter_node.dependency_count(), 1);
        assert!(filter_node.has_dependency(new_start_node.id()));
    }
}