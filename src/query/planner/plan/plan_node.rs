//! PlanNode特征和基础实现
//! 定义执行计划节点的通用接口和各种基础节点类型

use crate::impl_plan_node_for;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::plan_node_traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
    PlanNodeProperties, PlanNodeVisitable,
};
use crate::query::planner::plan::core::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::planner::plan::core::PlanNodeKind;
use std::sync::Arc;

/// 单一依赖节点 - 具有一个依赖的计划节点
#[derive(Debug)]
pub struct SingleDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

// Ensure SingleDependencyNode implements Send + Sync for PlanNode trait requirement
unsafe impl Send for SingleDependencyNode {}
unsafe impl Sync for SingleDependencyNode {}

impl Clone for SingleDependencyNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
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

impl_plan_node_for!(SingleDependencyNode);

impl PlanNodeVisitable for SingleDependencyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 单一输入节点 - 处理单一输入的计划节点
#[derive(Debug)]
pub struct SingleInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

// Ensure SingleInputNode implements Send + Sync for PlanNode trait requirement
unsafe impl Send for SingleInputNode {}
unsafe impl Sync for SingleInputNode {}

impl Clone for SingleInputNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
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

impl_plan_node_for!(SingleInputNode);

impl PlanNodeVisitable for SingleInputNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 二元输入节点 - 具有两个依赖的计划节点
#[derive(Debug)]
pub struct BinaryInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

// Ensure BinaryInputNode implements Send + Sync for PlanNode trait requirement
unsafe impl Send for BinaryInputNode {}
unsafe impl Sync for BinaryInputNode {}

impl Clone for BinaryInputNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl BinaryInputNode {
    pub fn new(kind: PlanNodeKind, left: Arc<dyn PlanNode>, right: Arc<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
            kind,
            dependencies: vec![left, right],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl_plan_node_for!(BinaryInputNode);

impl PlanNodeVisitable for BinaryInputNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

/// 可变依赖节点 - 具有可变数量依赖的计划节点
#[derive(Debug)]
pub struct VariableDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

// Ensure VariableDependencyNode implements Send + Sync for PlanNode trait requirement
unsafe impl Send for VariableDependencyNode {}
unsafe impl Sync for VariableDependencyNode {}

impl Clone for VariableDependencyNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl VariableDependencyNode {
    pub fn new(kind: PlanNodeKind) -> Self {
        Self {
            id: -1, // 将在后续分配
            kind,
            dependencies: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.dependencies.push(dep);
    }
}

impl_plan_node_for!(VariableDependencyNode);

impl PlanNodeVisitable for VariableDependencyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}
