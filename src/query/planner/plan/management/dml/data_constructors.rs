//! 数据构造操作相关的计划节点
//! 包括创建新顶点、标签和属性的操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 创建新顶点计划节点
#[derive(Debug)]
pub struct NewVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewVertex {
    pub fn new(id: i64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewVertex,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            tag_id,
            props,
        }
    }
}

impl Clone for NewVertex {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_id: self.tag_id,
            props: self.props.clone(),
        }
    }
}

impl PlanNodeIdentifiable for NewVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for NewVertex {
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

impl PlanNodeDependencies for NewVertex {
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

impl PlanNodeDependenciesExt for NewVertex {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for NewVertex {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for NewVertex {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for NewVertex {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_new_vertex(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for NewVertex {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 创建新标签计划节点
#[derive(Debug)]
pub struct NewTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewTag {
    pub fn new(id: i64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewTag,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            tag_id,
            props,
        }
    }
}

impl Clone for NewTag {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_id: self.tag_id,
            props: self.props.clone(),
        }
    }
}

impl PlanNodeIdentifiable for NewTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for NewTag {
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

impl PlanNodeDependencies for NewTag {
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

impl PlanNodeDependenciesExt for NewTag {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for NewTag {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for NewTag {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for NewTag {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_new_tag(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for NewTag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 创建新属性计划节点
#[derive(Debug)]
pub struct NewProp {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub prop_name: String,
    pub prop_value: String,
}

impl NewProp {
    pub fn new(id: i64, prop_name: &str, prop_value: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewProp,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            prop_name: prop_name.to_string(),
            prop_value: prop_value.to_string(),
        }
    }
}

impl Clone for NewProp {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            prop_name: self.prop_name.clone(),
            prop_value: self.prop_value.clone(),
        }
    }
}

impl PlanNodeIdentifiable for NewProp {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for NewProp {
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

impl PlanNodeDependencies for NewProp {
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

impl PlanNodeDependenciesExt for NewProp {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for NewProp {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for NewProp {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for NewProp {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_new_prop(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for NewProp {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 创建新边计划节点
#[derive(Debug)]
pub struct NewEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewEdge {
    pub fn new(id: i64, edge_type_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            edge_type_id,
            props,
        }
    }
}

impl Clone for NewEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            edge_type_id: self.edge_type_id,
            props: self.props.clone(),
        }
    }
}

impl PlanNodeIdentifiable for NewEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for NewEdge {
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

impl PlanNodeDependencies for NewEdge {
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

impl PlanNodeDependenciesExt for NewEdge {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for NewEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for NewEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for NewEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_new_edge(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for NewEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
