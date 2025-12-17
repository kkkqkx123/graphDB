//! 排序和限制操作节点
//! 包含Sort、Limit、TopN等排序和限制相关的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

// 排序节点
#[derive(Debug)]
pub struct Sort {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub sort_items: Vec<String>, // 排序字段
    pub limit: Option<i64>,      // 限制数量
}

impl Sort {
    pub fn new(id: i64, sort_items: Vec<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Sort,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            sort_items,
            limit: None,
        }
    }
}

impl Clone for Sort {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            sort_items: self.sort_items.clone(),
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for Sort {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Sort {
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

impl PlanNodeDependencies for Sort {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for Sort {
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

impl PlanNodeClonable for Sort {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for Sort {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_sort(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Sort {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 限制节点
#[derive(Debug)]
pub struct Limit {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub offset: i64, // 偏移量
    pub count: i64,  // 计数
}

impl Limit {
    pub fn new(id: i64, offset: i64, count: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Limit,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            offset,
            count,
        }
    }

    pub fn count(&self) -> i64 {
        self.count
    }
}

impl Clone for Limit {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            offset: self.offset,
            count: self.count,
        }
    }
}

impl PlanNodeIdentifiable for Limit {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Limit {
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

impl PlanNodeDependencies for Limit {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for Limit {
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

impl PlanNodeClonable for Limit {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for Limit {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_limit(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Limit {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TopN节点
#[derive(Debug)]
pub struct TopN {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub sort_items: Vec<String>, // 排序字段
    pub limit: i64,              // 限制数量
}

impl TopN {
    pub fn new(id: i64, sort_items: Vec<String>, limit: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::TopN,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            sort_items,
            limit,
        }
    }
}

impl Clone for TopN {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            sort_items: self.sort_items.clone(),
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for TopN {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for TopN {
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

impl PlanNodeDependencies for TopN {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for TopN {
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

impl PlanNodeClonable for TopN {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for TopN {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_top_n(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for TopN {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 采样节点
#[derive(Debug)]
pub struct Sample {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub count: i64, // 采样数量
}

impl Sample {
    pub fn new(id: i64, count: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Sample,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            count,
        }
    }
}

impl Clone for Sample {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            count: self.count,
        }
    }
}

impl PlanNodeIdentifiable for Sample {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Sample {
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

impl PlanNodeDependencies for Sample {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for Sample {
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

impl PlanNodeClonable for Sample {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for Sample {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_sample(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Sample {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
