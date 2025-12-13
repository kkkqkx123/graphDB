//! 数据删除操作相关的计划节点
//! 包括删除顶点、边和标签的操作

use crate::query::planner::plan::core::{
    plan_node_traits::{PlanNode, PlanNodeIdentifiable, PlanNodeProperties, PlanNodeDependencies, PlanNodeMutable, PlanNodeVisitable, PlanNodeClonable},
    PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError,
};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 删除顶点计划节点
#[derive(Debug)]
pub struct DeleteVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub filter: Option<String>, // 过滤条件
    pub delete_all: bool,       // 是否删除所有顶点
}

impl DeleteVertices {
    pub fn new(
        id: i64,
        space_id: i32,
        filter: Option<String>,
        delete_all: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            filter,
            delete_all,
        }
    }
}

impl Clone for DeleteVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            filter: self.filter.clone(),
            delete_all: self.delete_all,
        }
    }
}

impl PlanNodeIdentifiable for DeleteVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DeleteVertices {
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

impl PlanNodeDependencies for DeleteVertices {
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

impl PlanNodeMutable for DeleteVertices {
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

impl PlanNodeClonable for DeleteVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for DeleteVertices {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_delete_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DeleteVertices {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除边计划节点
#[derive(Debug)]
pub struct DeleteEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: Option<i32>, // 可选的边类型ID
    pub filter: Option<String>,    // 过滤条件
    pub delete_all: bool,          // 是否删除所有边
}

impl DeleteEdges {
    pub fn new(
        id: i64,
        space_id: i32,
        edge_type_id: Option<i32>,
        filter: Option<String>,
        delete_all: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_type_id,
            filter,
            delete_all,
        }
    }
}

impl Clone for DeleteEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_type_id: self.edge_type_id,
            filter: self.filter.clone(),
            delete_all: self.delete_all,
        }
    }
}

impl PlanNodeIdentifiable for DeleteEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DeleteEdges {
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

impl PlanNodeDependencies for DeleteEdges {
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

impl PlanNodeMutable for DeleteEdges {
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

impl PlanNodeClonable for DeleteEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for DeleteEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_delete_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DeleteEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除标签计划节点
#[derive(Debug)]
pub struct DeleteTags {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_ids: Vec<i32>, // 要删除的标签ID列表
}

impl DeleteTags {
    pub fn new(
        id: i64,
        space_id: i32,
        tag_ids: Vec<i32>,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteTags,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_ids,
        }
    }
}

impl Clone for DeleteTags {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_ids: self.tag_ids.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DeleteTags {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DeleteTags {
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

impl PlanNodeDependencies for DeleteTags {
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

impl PlanNodeMutable for DeleteTags {
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

impl PlanNodeClonable for DeleteTags {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for DeleteTags {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_delete_tags(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DeleteTags {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}