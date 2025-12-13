//! PlanNode基础实现
//! 定义执行计划节点的基础结构体

use super::plan_node_kind::PlanNodeKind;
use super::plan_node_traits::{PlanNode, PlanNodeIdentifiable, PlanNodeProperties, 
                             PlanNodeDependencies, PlanNodeMutable, PlanNodeVisitable, PlanNodeClonable};
use crate::query::planner::plan::core::{PlanNodeVisitor, PlanNodeVisitError};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 单一依赖节点 - 具有一个依赖的计划节点
/// 使用Arc优化所有权设计
#[derive(Debug)]
pub struct SingleDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for SingleDependencyNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self.dependencies.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl SingleDependencyNode {
    pub fn new(kind: PlanNodeKind, dep: Arc<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
            kind,
            dependencies: vec![dep],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

// 为SingleDependencyNode实现所有trait
crate::impl_plan_node_for!(SingleDependencyNode);

impl PlanNodeVisitable for SingleDependencyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 单一输入节点 - 处理单一输入的计划节点
/// 使用Arc优化所有权设计
#[derive(Debug)]
pub struct SingleInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for SingleInputNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self.dependencies.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl SingleInputNode {
    pub fn new(kind: PlanNodeKind, dep: Arc<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
            kind,
            dependencies: vec![dep],
            output_var: None,
            col_names: vec!["default".to_string()],
            cost: 0.0,
        }
    }
}

// 为SingleInputNode实现所有trait
crate::impl_plan_node_for!(SingleInputNode);

impl PlanNodeVisitable for SingleInputNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 二元输入节点 - 处理两个输入的计划节点
/// 使用Arc优化所有权设计
#[derive(Debug)]
pub struct BinaryInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for BinaryInputNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self.dependencies.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl BinaryInputNode {
    pub fn new(kind: PlanNodeKind, left: Arc<dyn PlanNode>, right: Arc<dyn PlanNode>) -> Self {
        Self {
            id: -1,
            kind,
            dependencies: vec![left, right],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

// 为BinaryInputNode实现所有trait
crate::impl_plan_node_for!(BinaryInputNode);

impl PlanNodeVisitable for BinaryInputNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 变量依赖节点 - 具有变量依赖的计划节点
/// 使用Arc优化所有权设计
#[derive(Debug)]
pub struct VariableDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for VariableDependencyNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self.dependencies.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl VariableDependencyNode {
    pub fn new(kind: PlanNodeKind) -> Self {
        Self {
            id: -1,
            kind,
            dependencies: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

// 为VariableDependencyNode实现所有trait
crate::impl_plan_node_for!(VariableDependencyNode);

impl PlanNodeVisitable for VariableDependencyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}