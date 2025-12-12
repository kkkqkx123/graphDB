//! 系统管理操作相关的计划节点
//! 包括提交任务、创建快照等维护操作

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, SingleDependencyNode, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::validator::Variable;

// 任务类型枚举
#[derive(Debug, Clone)]
pub enum JobType {
    Compaction,
    Flush,
    Stats,
    DataBalance,
    ZoneBalance,
}

/// 提交任务计划节点
#[derive(Debug)]
pub struct SubmitJob {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub job_type: JobType,       // 任务类型
    pub parameters: Vec<String>, // 任务参数
}

impl SubmitJob {
    pub fn new(id: i64, job_type: JobType, parameters: Vec<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::SubmitJob,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "JobId".to_string(),
                "Type".to_string(),
                "Status".to_string(),
                "Start Time".to_string(),
                "Stop Time".to_string(),
            ],
            cost: 0.0,
            job_type,
            parameters,
        }
    }

    pub fn job_type(&self) -> &JobType {
        &self.job_type
    }

    pub fn parameters(&self) -> &Vec<String> {
        &self.parameters
    }
}

impl Clone for SubmitJob {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            job_type: self.job_type.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

impl BasePlanNode for SubmitJob {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 创建快照计划节点
#[derive(Debug)]
pub struct CreateSnapshot {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub name: String,            // 快照名称
    pub comment: Option<String>, // 快照说明
}

impl CreateSnapshot {
    pub fn new(id: i64, name: &str, comment: Option<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateSnapshot,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Result".to_string()],
            cost: 0.0,
            name: name.to_string(),
            comment,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }
}

impl Clone for CreateSnapshot {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            name: self.name.clone(),
            comment: self.comment.clone(),
        }
    }
}

impl BasePlanNode for CreateSnapshot {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除快照计划节点
#[derive(Debug)]
pub struct DropSnapshot {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub name: String, // 快照名称
}

impl DropSnapshot {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropSnapshot,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Result".to_string()],
            cost: 0.0,
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Clone for DropSnapshot {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            name: self.name.clone(),
        }
    }
}

impl BasePlanNode for DropSnapshot {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 显示快照计划节点
#[derive(Debug)]
pub struct ShowSnapshots {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ShowSnapshots {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowSnapshots,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Name".to_string(),
                "Status".to_string(),
                "Hosts".to_string(),
            ],
            cost: 0.0,
        }
    }
}

impl Clone for ShowSnapshots {
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

impl BasePlanNode for ShowSnapshots {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}