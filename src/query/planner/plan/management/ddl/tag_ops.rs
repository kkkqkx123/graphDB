//! 标签操作相关的计划节点
//! 包括创建/删除标签等操作

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

/// 创建标签计划节点
#[derive(Debug)]
pub struct CreateTag {
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

impl CreateTag {
    pub fn new(id: i64, name: &str, schema: Schema, if_not_exists: bool) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateTag,
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

impl Clone for CreateTag {
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

impl PlanNodeIdentifiable for CreateTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for CreateTag {
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

impl PlanNodeDependencies for CreateTag {
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

impl PlanNodeDependenciesExt for CreateTag {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for CreateTag {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateTag {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for CreateTag {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for CreateTag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 描述标签计划节点
#[derive(Debug)]
pub struct DescTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_name: String,
}

impl DescTag {
    pub fn new(id: i64, tag_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DescTag,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Field".to_string(),
                "Type".to_string(),
                "Null".to_string(),
                "Default".to_string(),
                "Comment".to_string(),
            ],
            cost: 0.0,
            tag_name: tag_name.to_string(),
        }
    }
}

impl Clone for DescTag {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_name: self.tag_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DescTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DescTag {
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

impl PlanNodeDependencies for DescTag {
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

impl PlanNodeDependenciesExt for DescTag {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DescTag {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DescTag {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DescTag {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DescTag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除标签计划节点
#[derive(Debug)]
pub struct DropTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub tag_name: String,
}

impl DropTag {
    pub fn new(id: i64, if_exists: bool, tag_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropTag,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exists,
            tag_name: tag_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl Clone for DropTag {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exists: self.if_exists,
            tag_name: self.tag_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DropTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DropTag {
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

impl PlanNodeDependencies for DropTag {
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

impl PlanNodeDependenciesExt for DropTag {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DropTag {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropTag {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DropTag {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DropTag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示标签列表计划节点
#[derive(Debug)]
pub struct ShowTags {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ShowTags {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowTags,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Name".to_string()],
            cost: 0.0,
        }
    }
}

impl Clone for ShowTags {
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

impl PlanNodeIdentifiable for ShowTags {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowTags {
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

impl PlanNodeDependencies for ShowTags {
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

impl PlanNodeDependenciesExt for ShowTags {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowTags {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowTags {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowTags {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowTags {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示创建标签计划节点
#[derive(Debug)]
pub struct ShowCreateTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_name: String,
}

impl ShowCreateTag {
    pub fn new(id: i64, tag_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowCreateTag,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Tag".to_string(), "Create Tag".to_string()],
            cost: 0.0,
            tag_name: tag_name.to_string(),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl Clone for ShowCreateTag {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_name: self.tag_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ShowCreateTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowCreateTag {
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

impl PlanNodeDependencies for ShowCreateTag {
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

impl PlanNodeDependenciesExt for ShowCreateTag {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowCreateTag {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowCreateTag {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowCreateTag {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowCreateTag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
