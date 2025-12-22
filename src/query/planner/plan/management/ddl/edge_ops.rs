//! 边操作相关的计划节点
//! 包括创建/删除边等操作

use super::space_ops::Schema;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
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

impl PlanNodeDependencies for CreateEdge {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.deps.len();
        self.deps.retain(|dep| dep.id() != id);
        let final_len = self.deps.len();

        initial_len != final_len
    }
}

impl PlanNodeDependenciesExt for CreateEdge {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for CreateEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
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

/// 删除边计划节点
#[derive(Debug)]
pub struct DropEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub edge_name: String,
}

impl DropEdge {
    pub fn new(id: i64, if_exists: bool, edge_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exists,
            edge_name: edge_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl Clone for DropEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exists: self.if_exists,
            edge_name: self.edge_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DropEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DropEdge {
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

impl PlanNodeDependencies for DropEdge {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.deps.len();
        self.deps.retain(|dep| dep.id() != id);
        let final_len = self.deps.len();

        initial_len != final_len
    }
}

impl PlanNodeDependenciesExt for DropEdge {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DropEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DropEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DropEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示边列表计划节点
#[derive(Debug)]
pub struct ShowEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ShowEdges {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Name".to_string()],
            cost: 0.0,
        }
    }
}

impl Clone for ShowEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl PlanNodeIdentifiable for ShowEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowEdges {
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

impl PlanNodeDependencies for ShowEdges {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.deps.len();
        self.deps.retain(|dep| dep.id() != id);
        let final_len = self.deps.len();

        initial_len != final_len
    }
}

impl PlanNodeDependenciesExt for ShowEdges {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowEdges {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示创建边计划节点
#[derive(Debug)]
pub struct ShowCreateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub edge_name: String,
}

impl ShowCreateEdge {
    pub fn new(id: i64, edge_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowCreateEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Edge".to_string(), "Create Edge".to_string()],
            cost: 0.0,
            edge_name: edge_name.to_string(),
        }
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl Clone for ShowCreateEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            edge_name: self.edge_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ShowCreateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowCreateEdge {
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

impl PlanNodeDependencies for ShowCreateEdge {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.deps.len();
        self.deps.retain(|dep| dep.id() != id);
        let final_len = self.deps.len();

        initial_len != final_len
    }
}

impl PlanNodeDependenciesExt for ShowCreateEdge {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowCreateEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowCreateEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowCreateEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowCreateEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
