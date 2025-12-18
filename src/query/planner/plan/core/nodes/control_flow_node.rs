//! 控制流节点实现
//!
//! 包含Start、Argument、Select、Loop等控制流相关的计划节点

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
    PlanNodeProperties, PlanNodeVisitable,
};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// Argument节点 - 用于从另一个已执行的操作中获取命名别名
#[derive(Debug, Clone)]
pub struct ArgumentNode {
    id: i64,
    var: String,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ArgumentNode {
    pub fn new(id: i64, var: &str) -> Self {
        Self {
            id,
            var: var.to_string(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }
}

impl PlanNodeIdentifiable for ArgumentNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Argument
    }
}

impl PlanNodeProperties for ArgumentNode {
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

impl PlanNodeDependencies for ArgumentNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
         &[]
     }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
 }

impl PlanNodeMutable for ArgumentNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ArgumentNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ArgumentNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_argument(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ArgumentNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Select节点 - 在运行时选择if分支或else分支
#[derive(Debug, Clone)]
pub struct SelectNode {
    id: i64,
    condition: String,
    if_branch: Option<Arc<dyn PlanNode>>,
    else_branch: Option<Arc<dyn PlanNode>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl SelectNode {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            condition: condition.to_string(),
            if_branch: None,
            else_branch: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_if_branch(&mut self, branch: Arc<dyn PlanNode>) {
        self.if_branch = Some(branch);
    }

    pub fn set_else_branch(&mut self, branch: Arc<dyn PlanNode>) {
        self.else_branch = Some(branch);
    }

    pub fn if_branch(&self) -> &Option<Arc<dyn PlanNode>> {
        &self.if_branch
    }

    pub fn else_branch(&self) -> &Option<Arc<dyn PlanNode>> {
        &self.else_branch
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }
}

impl PlanNodeIdentifiable for SelectNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Select
    }
}

impl PlanNodeProperties for SelectNode {
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

impl PlanNodeDependencies for SelectNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
         // This is a simplified implementation
         // In a real implementation, we would need to store dependencies
         &[]
     }
 
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         // This is a simplified implementation
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
 
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
 }

impl PlanNodeMutable for SelectNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for SelectNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            condition: self.condition.clone(),
            if_branch: self.if_branch.as_ref().map(|node| node.clone_plan_node()),
            else_branch: self.else_branch.as_ref().map(|node| node.clone_plan_node()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for SelectNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_select(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for SelectNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Loop节点 - 在运行时多次执行分支
#[derive(Debug, Clone)]
pub struct LoopNode {
    id: i64,
    condition: String,
    body: Option<Arc<dyn PlanNode>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl LoopNode {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            condition: condition.to_string(),
            body: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_body(&mut self, body: Arc<dyn PlanNode>) {
        self.body = Some(body);
    }

    pub fn body(&self) -> &Option<Arc<dyn PlanNode>> {
        &self.body
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }
}

impl PlanNodeIdentifiable for LoopNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Loop
    }
}

impl PlanNodeProperties for LoopNode {
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

impl PlanNodeDependencies for LoopNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
         // This is a simplified implementation
         &[]
     }
 
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
 
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
 }

impl PlanNodeMutable for LoopNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for LoopNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            condition: self.condition.clone(),
            body: self.body.as_ref().map(|node| node.clone_plan_node()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for LoopNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_loop(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for LoopNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// PassThrough节点 - 用于透传情况的节点
#[derive(Debug, Clone)]
pub struct PassThroughNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl PassThroughNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl PlanNodeIdentifiable for PassThroughNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::PassThrough
    }
}

impl PlanNodeProperties for PassThroughNode {
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

impl PlanNodeDependencies for PassThroughNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
         &[]
     }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
 }

impl PlanNodeMutable for PassThroughNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for PassThroughNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for PassThroughNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_pass_through(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for PassThroughNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_node_creation() {
        let node = ArgumentNode::new(1, "var_name");
        assert_eq!(node.kind(), PlanNodeKind::Argument);
        assert_eq!(node.id(), 1);
        assert_eq!(node.var(), "var_name");
    }

    #[test]
    fn test_select_node_creation() {
        let node = SelectNode::new(1, "condition");
        assert_eq!(node.kind(), PlanNodeKind::Select);
        assert_eq!(node.id(), 1);
        assert_eq!(node.condition(), "condition");
        assert!(node.if_branch().is_none());
        assert!(node.else_branch().is_none());
    }

    #[test]
    fn test_loop_node_creation() {
        let node = LoopNode::new(1, "condition");
        assert_eq!(node.kind(), PlanNodeKind::Loop);
        assert_eq!(node.id(), 1);
        assert_eq!(node.condition(), "condition");
        assert!(node.body().is_none());
    }

    #[test]
    fn test_pass_through_node_creation() {
        let node = PassThroughNode::new(1);
        assert_eq!(node.kind(), PlanNodeKind::PassThrough);
        assert_eq!(node.id(), 1);
    }
}
