//! 控制流节点实现
//!
//! 包含Start、Argument、Select、Loop等控制流相关的计划节点

use crate::query::context::validate::types::Variable;

/// Argument节点 - 用于从另一个已执行的操作中获取命名别名
#[derive(Debug)]
pub struct ArgumentNode {
    id: i64,
    var: String,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 ArgumentNode 实现 Clone
impl Clone for ArgumentNode {
    fn clone(&self) -> Self {
        ArgumentNode {
            id: self.id,
            var: self.var.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(), // 依赖关系不复制，因为它们在新的上下文中无效
        }
    }
}

impl ArgumentNode {
    pub fn new(id: i64, var: &str) -> Self {
        Self {
            id,
            var: var.to_string(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }
}

impl ArgumentNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Argument"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Argument(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Argument(cloned)
    }
}

/// Select节点 - 在运行时选择if分支或else分支
#[derive(Debug)]
pub struct SelectNode {
    id: i64,
    condition: String,
    if_branch: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    else_branch: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 SelectNode 实现 Clone
impl Clone for SelectNode {
    fn clone(&self) -> Self {
        SelectNode {
            id: self.id,
            condition: self.condition.clone(),
            if_branch: self.if_branch.as_ref().map(|node| node.clone()),
            else_branch: self.else_branch.as_ref().map(|node| node.clone()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(), // 依赖关系不复制，因为它们在新的上下文中无效
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
            cost: 0.0,
            dependencies: Vec::new(),
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
}

impl SelectNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Select"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Select(Self {
            id: self.id,
            condition: self.condition.clone(),
            if_branch: self.if_branch.clone(),
            else_branch: self.else_branch.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Select(cloned)
    }
}

/// Loop节点 - 在运行时多次执行分支
#[derive(Debug)]
pub struct LoopNode {
    id: i64,
    condition: String,
    body: Option<Box<super::plan_node_enum::PlanNodeEnum>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 LoopNode 实现 Clone
impl Clone for LoopNode {
    fn clone(&self) -> Self {
        LoopNode {
            id: self.id,
            condition: self.condition.clone(),
            body: self.body.as_ref().map(|node| node.clone()),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(), // 依赖关系不复制，因为它们在新的上下文中无效
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
            cost: 0.0,
            dependencies: Vec::new(),
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
}

impl LoopNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Loop"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Loop(Self {
            id: self.id,
            condition: self.condition.clone(),
            body: self.body.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Loop(cloned)
    }
}

/// PassThrough节点 - 用于透传情况的节点
#[derive(Debug)]
pub struct PassThroughNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 PassThroughNode 实现 Clone
impl Clone for PassThroughNode {
    fn clone(&self) -> Self {
        PassThroughNode {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(), // 依赖关系不复制，因为它们在新的上下文中无效
        }
    }
}

impl PassThroughNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }
}

impl PassThroughNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "PassThrough"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::PassThrough(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::PassThrough(cloned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_node_creation() {
        let node = ArgumentNode::new(1, "var_name");
        assert_eq!(node.type_name(), "Argument");
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
        assert_eq!(node.type_name(), "PassThrough");
        assert_eq!(node.id(), 1);
    }
}
