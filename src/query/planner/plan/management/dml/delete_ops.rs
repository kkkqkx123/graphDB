//! 数据删除操作相关的计划节点
//! 包括删除顶点和边的操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
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
    pub vid_ref: String, // 顶点ID引用，可能是表达式
}

impl DeleteVertices {
    pub fn new(id: i64, space_id: i32, vid_ref: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vid_ref: vid_ref.to_string(),
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn vid_ref(&self) -> &str {
        &self.vid_ref
    }
}

impl Clone for DeleteVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vid_ref: self.vid_ref.clone(),
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

impl PlanNodeDependencies for DeleteVertices {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
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

impl PlanNodeDependenciesExt for DeleteVertices {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DeleteVertices {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DeleteVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
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
    pub vid_ref: String,    // 顶点ID引用，可能是表达式
    pub tag_ids: Vec<i32>, // 要删除的标签ID列表
}

impl DeleteTags {
    pub fn new(id: i64, space_id: i32, vid_ref: &str, tag_ids: Vec<i32>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteTags,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vid_ref: vid_ref.to_string(),
            tag_ids,
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn vid_ref(&self) -> &str {
        &self.vid_ref
    }

    pub fn tag_ids(&self) -> &[i32] {
        &self.tag_ids
    }
}

impl Clone for DeleteTags {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vid_ref: self.vid_ref.clone(),
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

impl PlanNodeDependencies for DeleteTags {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
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

impl PlanNodeDependenciesExt for DeleteTags {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DeleteTags {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DeleteTags {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
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
    pub edge_key_ref: String, // 边键引用，可能是表达式
}

impl DeleteEdges {
    pub fn new(id: i64, space_id: i32, edge_key_ref: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_key_ref: edge_key_ref.to_string(),
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn edge_key_ref(&self) -> &str {
        &self.edge_key_ref
    }
}

impl Clone for DeleteEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_key_ref: self.edge_key_ref.clone(),
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

impl PlanNodeDependencies for DeleteEdges {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
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

impl PlanNodeDependenciesExt for DeleteEdges {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DeleteEdges {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DeleteEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
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