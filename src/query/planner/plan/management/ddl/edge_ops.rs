//! 边操作相关的计划节点
//! 包括创建/删除边等操作

use crate::query::planner::plan::core::{
    plan_node_traits::{PlanNode, PlanNodeIdentifiable, PlanNodeProperties, PlanNodeDependencies, PlanNodeMutable, PlanNodeVisitable, PlanNodeClonable},
    PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError,
};
use crate::query::context::validate::types::Variable;
use super::space_ops::Schema;
use std::sync::Arc;

/// 创建边计划节点
#[derive(Debug)]
pub struct CreateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub name: String,
    pub schema: Schema,
    pub if_not_exists: bool,
}

impl CreateEdge {
    pub fn new(id: i64, name: &str, schema: Schema, if_not_exists: bool) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            name: name.to_string(),
            schema,
            if_not_exists,
        }
    }
}

impl Clone for CreateEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            name: self.name.clone(),
            schema: self.schema.clone(),
            if_not_exists: self.if_not_exists,
        }
    }
}

impl PlanNodeIdentifiable for CreateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for CreateEdge {
    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for CreateEdge {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        &mut self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeMutable for CreateEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}

impl PlanNodeClonable for CreateEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for CreateEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for CreateEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}