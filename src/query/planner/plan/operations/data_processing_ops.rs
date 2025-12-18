//! 数据处理操作节点
//! 包含Filter、Project、Unwind等数据处理相关的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::utils::join_params::JoinParams;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

// 过滤计划节点
#[derive(Debug)]
pub struct Filter {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub condition: String, // 过滤条件
}

impl Filter {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Filter,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            condition: condition.to_string(),
        }
    }
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            condition: self.condition.clone(),
        }
    }
}

impl PlanNodeIdentifiable for Filter {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Filter {
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

impl PlanNodeDependencies for Filter {
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

impl PlanNodeMutable for Filter {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Filter {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Filter {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_filter(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Filter {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 投影计划节点
#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub yield_expr: String, // 投影表达式
}

impl Project {
    pub fn new(id: i64, yield_expr: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Project,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            yield_expr: yield_expr.to_string(),
        }
    }
}

impl Clone for Project {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            yield_expr: self.yield_expr.clone(),
        }
    }
}

impl PlanNodeIdentifiable for Project {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Project {
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

impl PlanNodeDependencies for Project {
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

impl PlanNodeMutable for Project {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Project {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Project {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_project(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Project {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Unwind节点
#[derive(Debug)]
pub struct Unwind {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub alias: String,
    pub list_expr: String,
}

impl Unwind {
    pub fn new(id: i64, alias: &str, list_expr: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Unwind,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            alias: alias.to_string(),
            list_expr: list_expr.to_string(),
        }
    }
}

impl Clone for Unwind {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            alias: self.alias.clone(),
            list_expr: self.list_expr.clone(),
        }
    }
}

impl PlanNodeIdentifiable for Unwind {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Unwind {
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

impl PlanNodeDependencies for Unwind {
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

impl PlanNodeMutable for Unwind {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Unwind {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Unwind {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_unwind(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Unwind {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 去重节点
#[derive(Debug)]
pub struct Dedup {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Dedup {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Dedup,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl Clone for Dedup {
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

impl PlanNodeIdentifiable for Dedup {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Dedup {
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

impl PlanNodeDependencies for Dedup {
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

impl PlanNodeMutable for Dedup {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Dedup {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Dedup {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_dedup(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Dedup {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 联合节点
#[derive(Debug)]
pub struct Union {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub distinct: bool,
}

impl Union {
    pub fn new(id: i64, distinct: bool) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Union,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            distinct,
        }
    }
}

impl Clone for Union {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            distinct: self.distinct,
        }
    }
}

impl PlanNodeIdentifiable for Union {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Union {
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

impl PlanNodeDependencies for Union {
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

impl PlanNodeMutable for Union {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Union {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Union {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_union(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Union {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// RollUpApply节点
#[derive(Debug)]
pub struct RollUpApply {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub collect_exprs: Vec<String>,
    pub lambda_vars: Vec<String>,
    pub join_params: Option<JoinParams>,
}

impl RollUpApply {
    pub fn new(id: i64, collect_exprs: Vec<String>, lambda_vars: Vec<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::RollUpApply,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            collect_exprs,
            lambda_vars,
            join_params: None,
        }
    }

    pub fn with_join_params(mut self, join_params: JoinParams) -> Self {
        self.join_params = Some(join_params);
        self
    }
}

impl Clone for RollUpApply {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            collect_exprs: self.collect_exprs.clone(),
            lambda_vars: self.lambda_vars.clone(),
            join_params: self.join_params.clone(),
        }
    }
}

impl PlanNodeIdentifiable for RollUpApply {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for RollUpApply {
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

impl PlanNodeDependencies for RollUpApply {
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

impl PlanNodeMutable for RollUpApply {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for RollUpApply {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for RollUpApply {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_roll_up_apply(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for RollUpApply {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// PatternApply节点
#[derive(Debug)]
pub struct PatternApply {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub pattern: String,
    pub join_type: String,
    pub join_params: Option<JoinParams>,
}

impl PatternApply {
    pub fn new(id: i64, pattern: &str, join_type: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::PatternApply,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            pattern: pattern.to_string(),
            join_type: join_type.to_string(),
            join_params: None,
        }
    }

    pub fn with_join_params(mut self, join_params: JoinParams) -> Self {
        self.join_params = Some(join_params);
        self
    }
}

impl Clone for PatternApply {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            pattern: self.pattern.clone(),
            join_type: self.join_type.clone(),
            join_params: self.join_params.clone(),
        }
    }
}

impl PlanNodeIdentifiable for PatternApply {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for PatternApply {
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

impl PlanNodeDependencies for PatternApply {
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

impl PlanNodeMutable for PatternApply {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for PatternApply {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for PatternApply {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_pattern_apply(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for PatternApply {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 数据收集节点
#[derive(Debug)]
pub struct DataCollect {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub collect_kind: String, // "kCollect", "kCollectLimit", "kUnionAll"等
}

impl DataCollect {
    pub fn new(id: i64, collect_kind: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DataCollect,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            collect_kind: collect_kind.to_string(),
        }
    }
}

impl Clone for DataCollect {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            collect_kind: self.collect_kind.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DataCollect {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DataCollect {
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

impl PlanNodeDependencies for DataCollect {
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

impl PlanNodeMutable for DataCollect {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DataCollect {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DataCollect {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_data_collect(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DataCollect {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
