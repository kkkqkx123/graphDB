//! 聚合操作计划节点定义
use super::plan_node::{PlanNode as BasePlanNode, PlanNodeKind};
use super::plan_node_visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;

// 聚合节点
#[derive(Debug)]
pub struct Aggregate {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
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

impl BasePlanNode for Aggregate {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_aggregate(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

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