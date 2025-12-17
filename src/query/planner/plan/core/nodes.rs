//! 具体的计划节点实现
//! 
//! 这个模块实现了具体的计划节点类型，替代通用的 SingleInputNode、BinaryInputNode 等
//! 提供类型安全的节点操作和更清晰的语义

use super::plan_node_kind::PlanNodeKind;
use super::plan_node_traits::{PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable};
use super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use crate::query::parser::ast::expr::Expr;
use crate::query::validator::structs::common_structs::YieldColumn;
use std::sync::Arc;

/// 单输入节点特征
pub trait SingleInputPlanNode: PlanNode {
    fn input(&self) -> &Arc<dyn PlanNode>;
    fn set_input(&mut self, input: Arc<dyn PlanNode>);
}

/// 双输入节点特征
pub trait BinaryInputPlanNode: PlanNode {
    fn left(&self) -> &Arc<dyn PlanNode>;
    fn right(&self) -> &Arc<dyn PlanNode>;
    fn set_left(&mut self, left: Arc<dyn PlanNode>);
    fn set_right(&mut self, right: Arc<dyn PlanNode>);
}

/// 过滤节点
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
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { std::slice::from_ref(&self.input) }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        // 使用 unsafe 代码来返回可变引用，因为我们需要修改单个依赖
        unsafe { std::mem::transmute(&mut [self.input.clone()] as &mut [Arc<dyn PlanNode>]) }
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

/// 投影节点
#[derive(Debug, Clone)]
pub struct ProjectNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    columns: Vec<YieldColumn>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ProjectNode {
    pub fn new(
        input: Arc<dyn PlanNode>,
        columns: Vec<YieldColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter()
            .map(|col| col.alias.clone())
            .collect();
        
        Ok(Self {
            id: -1,
            input,
            columns,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }
    
    pub fn columns(&self) -> &[YieldColumn] {
        &self.columns
    }
}

impl PlanNodeIdentifiable for ProjectNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Project }
}

impl PlanNodeProperties for ProjectNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for ProjectNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { std::slice::from_ref(&self.input) }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        unsafe { std::mem::transmute(&mut [self.input.clone()] as &mut [Arc<dyn PlanNode>]) }
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep;
    }
    
    fn remove_dependency(&mut self, id: i64) -> bool {
        if self.input.id() == id {
            false
        } else {
            false
        }
    }
}

impl PlanNodeMutable for ProjectNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
}

impl PlanNodeClonable for ProjectNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            columns: self.columns.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for ProjectNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_project_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ProjectNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SingleInputPlanNode for ProjectNode {
    fn input(&self) -> &Arc<dyn PlanNode> {
        &self.input
    }
    
    fn set_input(&mut self, input: Arc<dyn PlanNode>) {
        self.input = input;
    }
}

/// 内连接节点
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
}

impl InnerJoinNode {
    pub fn new(
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        hash_keys: Vec<Expr>,
        probe_keys: Vec<Expr>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());
        
        Ok(Self {
            id: -1,
            left,
            right,
            hash_keys,
            probe_keys,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }
    
    pub fn hash_keys(&self) -> &[Expr] {
        &self.hash_keys
    }
    
    pub fn probe_keys(&self) -> &[Expr] {
        &self.probe_keys
    }
}

impl PlanNodeIdentifiable for InnerJoinNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::HashInnerJoin }
}

impl PlanNodeProperties for InnerJoinNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for InnerJoinNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        // 使用 unsafe 代码来返回两个依赖的引用
        unsafe { std::mem::transmute([&self.left, &self.right] as [&Arc<dyn PlanNode>; 2]) }
    }
    
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        unsafe { std::mem::transmute(&mut [self.left.clone(), self.right.clone()] as &mut [Arc<dyn PlanNode>]) }
    }
    
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 内连接节点不支持添加依赖
    }
    
    fn remove_dependency(&mut self, id: i64) -> bool {
        if self.left.id() == id || self.right.id() == id {
            false
        } else {
            false
        }
    }
}

impl PlanNodeMutable for InnerJoinNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
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

impl BinaryInputPlanNode for InnerJoinNode {
    fn left(&self) -> &Arc<dyn PlanNode> {
        &self.left
    }
    
    fn right(&self) -> &Arc<dyn PlanNode> {
        &self.right
    }
    
    fn set_left(&mut self, left: Arc<dyn PlanNode>) {
        self.left = left;
    }
    
    fn set_right(&mut self, right: Arc<dyn PlanNode>) {
        self.right = right;
    }
}

/// 起始节点
#[derive(Debug, Clone)]
pub struct StartNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl StartNode {
    pub fn new() -> Self {
        Self {
            id: -1,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        }
    }
}

impl PlanNodeIdentifiable for StartNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Start }
}

impl PlanNodeProperties for StartNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for StartNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        static mut EMPTY_DEPS: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY_DEPS }
    }
    
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 起始节点不支持依赖
    }
    
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for StartNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
}

impl PlanNodeClonable for StartNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
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
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 占位符节点
#[derive(Debug, Clone)]
pub struct PlaceholderNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl PlaceholderNode {
    pub fn new() -> Self {
        Self {
            id: -1,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        }
    }
}

impl PlanNodeIdentifiable for PlaceholderNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Argument }
}

impl PlanNodeProperties for PlaceholderNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for PlaceholderNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        static mut EMPTY_DEPS: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY_DEPS }
    }
    
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {
        // 占位符节点不支持依赖
    }
    
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for PlaceholderNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
}

impl PlanNodeClonable for PlaceholderNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for PlaceholderNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_placeholder_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for PlaceholderNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 节点工厂
pub struct PlanNodeFactory;

impl PlanNodeFactory {
    /// 创建过滤节点
    pub fn create_filter(
        input: Arc<dyn PlanNode>,
        condition: Expr,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(FilterNode::new(input, condition)?))
    }
    
    /// 创建投影节点
    pub fn create_project(
        input: Arc<dyn PlanNode>,
        columns: Vec<YieldColumn>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ProjectNode::new(input, columns)?))
    }
    
    /// 创建内连接节点
    pub fn create_inner_join(
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        hash_keys: Vec<Expr>,
        probe_keys: Vec<Expr>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(InnerJoinNode::new(left, right, hash_keys, probe_keys)?))
    }
    
    /// 创建起始节点
    pub fn create_start_node() -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(StartNode::new()))
    }
    
    /// 创建占位符节点
    pub fn create_placeholder_node() -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(PlaceholderNode::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_filter_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let condition = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let filter_node = PlanNodeFactory::create_filter(start_node, condition).unwrap();
        
        assert_eq!(filter_node.kind(), PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }
    
    #[test]
    fn test_project_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let columns = vec![YieldColumn {
            expr: Expr::Variable(VariableExpr::new("test".to_string(), Span::default())),
            alias: "test".to_string(),
        }];
        let project_node = PlanNodeFactory::create_project(start_node, columns).unwrap();
        
        assert_eq!(project_node.kind(), PlanNodeKind::Project);
        assert_eq!(project_node.dependencies().len(), 1);
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }
    
    #[test]
    fn test_inner_join_node() {
        let left_node = PlanNodeFactory::create_start_node().unwrap();
        let right_node = PlanNodeFactory::create_start_node().unwrap();
        let hash_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        let probe_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        
        let join_node = PlanNodeFactory::create_inner_join(
            left_node,
            right_node,
            hash_keys,
            probe_keys,
        ).unwrap();
        
        assert_eq!(join_node.kind(), PlanNodeKind::HashInnerJoin);
        assert_eq!(join_node.dependencies().len(), 2);
    }
    
    #[test]
    fn test_start_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        
        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }
    
    #[test]
    fn test_placeholder_node() {
        let placeholder_node = PlanNodeFactory::create_placeholder_node().unwrap();
        
        assert_eq!(placeholder_node.kind(), PlanNodeKind::Argument);
        assert_eq!(placeholder_node.dependencies().len(), 0);
        assert_eq!(placeholder_node.col_names().len(), 0);
    }
}