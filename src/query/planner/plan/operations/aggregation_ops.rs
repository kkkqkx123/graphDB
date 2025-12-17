//! 聚合操作计划节点定义
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

// 聚合节点
#[derive(Debug)]
pub struct Aggregate {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub group_keys: Vec<String>,
    pub agg_exprs: Vec<String>,
}

impl Aggregate {
    pub fn new(id: i64, group_keys: Vec<String>, agg_exprs: Vec<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Aggregate,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            group_keys,
            agg_exprs,
        }
    }
}

impl Clone for Aggregate {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            group_keys: self.group_keys.clone(),
            agg_exprs: self.agg_exprs.clone(),
        }
    }
}

impl PlanNodeIdentifiable for Aggregate {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Aggregate {
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

impl PlanNodeDependencies for Aggregate {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for Aggregate {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Aggregate {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Aggregate {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_aggregate(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Aggregate {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
