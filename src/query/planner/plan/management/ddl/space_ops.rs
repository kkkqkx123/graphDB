//! 空间操作相关的计划节点
//! 包括创建/删除空间等操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

// 基础创建节点结构
#[derive(Debug)]
pub struct CreateNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_not_exist: bool, // 是否使用IF NOT EXISTS
}

impl Clone for CreateNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_not_exist: self.if_not_exist,
        }
    }
}

impl CreateNode {
    pub fn new(id: i64, kind: PlanNodeKind, if_not_exist: bool) -> Self {
        Self {
            id,
            kind,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_not_exist,
        }
    }

    pub fn if_not_exist(&self) -> bool {
        self.if_not_exist
    }
}

impl PlanNodeIdentifiable for CreateNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for CreateNode {
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

impl PlanNodeDependencies for CreateNode {
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

impl PlanNodeDependenciesExt for CreateNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for CreateNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for CreateNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for CreateNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 基础删除节点结构
#[derive(Debug)]
pub struct DropNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exist: bool, // 是否使用IF EXISTS
}

impl Clone for DropNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exist: self.if_exist,
        }
    }
}

impl DropNode {
    pub fn new(id: i64, kind: PlanNodeKind, if_exist: bool) -> Self {
        Self {
            id,
            kind,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exist,
        }
    }

    pub fn if_exist(&self) -> bool {
        self.if_exist
    }
}

impl PlanNodeIdentifiable for DropNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DropNode {
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

impl PlanNodeDependencies for DropNode {
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

impl PlanNodeDependenciesExt for DropNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DropNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DropNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DropNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 元数据定义相关结构
#[derive(Debug, Clone)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String, // 简化为字符串，实际可能是复杂类型
    pub nullable: bool,
    pub default_value: Option<String>,
}

/// 创建空间计划节点
#[derive(Debug)]
pub struct CreateSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_not_exist: bool,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
}

impl CreateSpace {
    pub fn new(
        id: i64,
        if_not_exist: bool,
        space_name: &str,
        partition_num: i32,
        replica_factor: i32,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_not_exist,
            space_name: space_name.to_string(),
            partition_num,
            replica_factor,
        }
    }
}

impl Clone for CreateSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_not_exist: self.if_not_exist,
            space_name: self.space_name.clone(),
            partition_num: self.partition_num,
            replica_factor: self.replica_factor,
        }
    }
}

impl PlanNodeIdentifiable for CreateSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for CreateSpace {
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

impl PlanNodeDependencies for CreateSpace {
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

impl PlanNodeDependenciesExt for CreateSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for CreateSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for CreateSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for CreateSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 描述空间计划节点
#[derive(Debug)]
pub struct DescSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_name: String,
}

impl DescSpace {
    pub fn new(id: i64, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DescSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Name".to_string(),
                "Space(space_id)".to_string(),
                "Charset".to_string(),
                "Collate".to_string(),
                "Partition Number".to_string(),
                "Replica Factor".to_string(),
                "Vid Type".to_string(),
                "Atomic Edge".to_string(),
            ],
            cost: 0.0,
            space_name: space_name.to_string(),
        }
    }
}

impl Clone for DescSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_name: self.space_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DescSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DescSpace {
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

impl PlanNodeDependencies for DescSpace {
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

impl PlanNodeDependenciesExt for DescSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DescSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DescSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DescSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DescSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示创建空间计划节点
#[derive(Debug)]
pub struct ShowCreateSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_name: String,
}

impl ShowCreateSpace {
    pub fn new(id: i64, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowCreateSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Space".to_string(), "Create Space".to_string()],
            cost: 0.0,
            space_name: space_name.to_string(),
        }
    }
}

impl Clone for ShowCreateSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_name: self.space_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ShowCreateSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowCreateSpace {
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

impl PlanNodeDependencies for ShowCreateSpace {
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

impl PlanNodeDependenciesExt for ShowCreateSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowCreateSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowCreateSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowCreateSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowCreateSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示空间列表计划节点
#[derive(Debug)]
pub struct ShowSpaces {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ShowSpaces {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowSpaces,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Name".to_string()],
            cost: 0.0,
        }
    }
}

impl Clone for ShowSpaces {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl PlanNodeIdentifiable for ShowSpaces {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShowSpaces {
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

impl PlanNodeDependencies for ShowSpaces {
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

impl PlanNodeDependenciesExt for ShowSpaces {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShowSpaces {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShowSpaces {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShowSpaces {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShowSpaces {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 切换空间计划节点
#[derive(Debug)]
pub struct SwitchSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_name: String,
}

impl SwitchSpace {
    pub fn new(id: i64, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::SwitchSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Session space changed to".to_string()],
            cost: 0.0,
            space_name: space_name.to_string(),
        }
    }
}

impl Clone for SwitchSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_name: self.space_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for SwitchSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for SwitchSpace {
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

impl PlanNodeDependencies for SwitchSpace {
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

impl PlanNodeDependenciesExt for SwitchSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for SwitchSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for SwitchSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for SwitchSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for SwitchSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除空间计划节点
#[derive(Debug)]
pub struct DropSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}

impl DropSpace {
    pub fn new(id: i64, if_exists: bool, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exists,
            space_name: space_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl Clone for DropSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exists: self.if_exists,
            space_name: self.space_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DropSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DropSpace {
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

impl PlanNodeDependencies for DropSpace {
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

impl PlanNodeDependenciesExt for DropSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DropSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DropSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DropSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 清空空间计划节点
#[derive(Debug)]
pub struct ClearSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}

impl ClearSpace {
    pub fn new(id: i64, if_exists: bool, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ClearSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exists,
            space_name: space_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl Clone for ClearSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exists: self.if_exists,
            space_name: self.space_name.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ClearSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ClearSpace {
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

impl PlanNodeDependencies for ClearSpace {
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

impl PlanNodeDependenciesExt for ClearSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ClearSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ClearSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ClearSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ClearSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 修改空间选项
#[derive(Debug, Clone)]
pub enum AlterSpaceOption {
    AddZone(String),
    RemoveZone(String),
    SetPartitionNum(i32),
    SetReplicaFactor(i32),
}

/// 修改空间计划节点
#[derive(Debug)]
pub struct AlterSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_name: String,
    pub alter_options: Vec<AlterSpaceOption>,
}

impl AlterSpace {
    pub fn new(id: i64, space_name: &str, alter_options: Vec<AlterSpaceOption>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::AlterSpace,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_name: space_name.to_string(),
            alter_options,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn alter_options(&self) -> &[AlterSpaceOption] {
        &self.alter_options
    }
}

impl Clone for AlterSpace {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_name: self.space_name.clone(),
            alter_options: self.alter_options.clone(),
        }
    }
}

impl PlanNodeIdentifiable for AlterSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for AlterSpace {
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

impl PlanNodeDependencies for AlterSpace {
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

impl PlanNodeDependenciesExt for AlterSpace {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for AlterSpace {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for AlterSpace {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for AlterSpace {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for AlterSpace {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
