//! 控制流节点实现
//!
//! 包含Start、Argument、Select、Loop等控制流相关的计划节点

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{PlanNode, PlanNodeClonable};
use crate::define_plan_node;

define_plan_node! {
    pub struct ArgumentNode {
        var: String,
    }
    enum: Argument
    input: ZeroInputNode
}

impl ArgumentNode {
    pub fn new(id: i64, var: &str) -> Self {
        Self {
            id,
            var: var.to_string(),
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }
}

define_plan_node! {
    pub struct PassThroughNode {
    }
    enum: PassThrough
    input: ZeroInputNode
}

impl PassThroughNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
        }
    }
}

/// Select节点 - 在运行时选择if分支或else分支
#[derive(Debug)]
pub struct SelectNode {
    id: i64,
    condition: String,
    if_branch: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    else_branch: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    output_var: Option<String>,
    col_names: Vec<String>,
}

impl Clone for SelectNode {
    fn clone(&self) -> Self {
        SelectNode {
            id: self.id,
            condition: self.condition.clone(),
            if_branch: self.if_branch.as_ref().map(|node| node.clone()),
            else_branch: self.else_branch.as_ref().map(|node| node.clone()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
        }
    }
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
        }
    }

    pub fn set_if_branch(&mut self, branch: super::plan_node_enum::PlanNodeEnum) {
        self.if_branch = Some(Box::new(branch));
    }

    pub fn set_else_branch(&mut self, branch: super::plan_node_enum::PlanNodeEnum) {
        self.else_branch = Some(Box::new(branch));
    }

    pub fn if_branch(&self) -> &Option<Box<super::plan_node_enum::PlanNodeEnum>> {
        &self.if_branch
    }

    pub fn else_branch(&self) -> &Option<Box<super::plan_node_enum::PlanNodeEnum>> {
        &self.else_branch
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }

    pub fn type_name(&self) -> &'static str {
        "Select"
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn output_var(&self) -> Option<&str> {
        self.output_var.as_deref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn set_output_var(&mut self, var: String) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Select(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Select(cloned)
    }
}

impl PlanNode for SelectNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        "Select"
    }

    fn output_var(&self) -> Option<&str> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn set_output_var(&mut self, var: String) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::Select(self)
    }
}

impl PlanNodeClonable for SelectNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// Loop节点 - 在运行时多次执行分支
#[derive(Debug)]
pub struct LoopNode {
    id: i64,
    condition: String,
    body: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    output_var: Option<String>,
    col_names: Vec<String>,
}

impl Clone for LoopNode {
    fn clone(&self) -> Self {
        LoopNode {
            id: self.id,
            condition: self.condition.clone(),
            body: self.body.as_ref().map(|node| node.clone()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
        }
    }
}

impl LoopNode {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            condition: condition.to_string(),
            body: None,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn set_body(&mut self, body: super::plan_node_enum::PlanNodeEnum) {
        self.body = Some(Box::new(body));
    }

    pub fn body(&self) -> &Option<Box<super::plan_node_enum::PlanNodeEnum>> {
        &self.body
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }

    pub fn type_name(&self) -> &'static str {
        "Loop"
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn output_var(&self) -> Option<&str> {
        self.output_var.as_deref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn set_output_var(&mut self, var: String) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Loop(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Loop(cloned)
    }
}

impl PlanNode for LoopNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        "Loop"
    }

    fn output_var(&self) -> Option<&str> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn set_output_var(&mut self, var: String) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::Loop(self)
    }
}

impl PlanNodeClonable for LoopNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_node_creation() {
        let node = ArgumentNode::new(1, "var_name");
        assert_eq!(node.type_name(), "ArgumentNode");
        assert_eq!(node.id(), 1);
        assert_eq!(node.var(), "var_name");
    }

    #[test]
    fn test_select_node_creation() {
        let node = SelectNode::new(1, "condition");
        assert_eq!(node.type_name(), "Select");
        assert_eq!(node.id(), 1);
        assert_eq!(node.condition(), "condition");
        assert!(node.if_branch().is_none());
        assert!(node.else_branch().is_none());
    }

    #[test]
    fn test_loop_node_creation() {
        let node = LoopNode::new(1, "condition");
        assert_eq!(node.type_name(), "Loop");
        assert_eq!(node.id(), 1);
        assert_eq!(node.condition(), "condition");
        assert!(node.body().is_none());
    }

    #[test]
    fn test_pass_through_node_creation() {
        let node = PassThroughNode::new(1);
        assert_eq!(node.type_name(), "PassThroughNode");
        assert_eq!(node.id(), 1);
    }
}
