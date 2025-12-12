//! 配置操作相关的计划节点
//! 包括显示、设置和获取配置等操作

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::validator::Variable;

/// 配置参数类型
#[derive(Debug, Clone)]
pub enum ConfigType {
    Mutable,
    Immutable,
    All,
}

/// 配置参数
#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub mutable: bool,
    pub description: String,
}

/// 显示配置计划节点
#[derive(Debug)]
pub struct ShowConfigs {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub config_type: ConfigType,
    pub module_name: Option<String>, // 可选的模块名称
}

impl ShowConfigs {
    pub fn new(id: i64, config_type: ConfigType, module_name: Option<String>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowConfigs,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Module".to_string(), "Name".to_string(), "Value".to_string(), "Default".to_string(), "Type".to_string()],
            cost: 0.0,
            config_type,
            module_name,
        }
    }
}

impl Clone for ShowConfigs {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            config_type: self.config_type.clone(),
            module_name: self.module_name.clone(),
        }
    }
}

impl BasePlanNode for ShowConfigs {
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

/// 设置配置计划节点
#[derive(Debug)]
pub struct SetConfig {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub module_name: String,
    pub config_name: String,
    pub config_value: String,
}

impl SetConfig {
    pub fn new(id: i64, module_name: &str, config_name: &str, config_value: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::SetConfig,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            module_name: module_name.to_string(),
            config_name: config_name.to_string(),
            config_value: config_value.to_string(),
        }
    }
}

impl Clone for SetConfig {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            module_name: self.module_name.clone(),
            config_name: self.config_name.clone(),
            config_value: self.config_value.clone(),
        }
    }
}

impl BasePlanNode for SetConfig {
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

/// 获取配置计划节点
#[derive(Debug)]
pub struct GetConfig {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub module_name: String,
    pub config_name: String,
}

impl GetConfig {
    pub fn new(id: i64, module_name: &str, config_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::GetConfig,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Module".to_string(), "Name".to_string(), "Value".to_string(), "Default".to_string(), "Type".to_string()],
            cost: 0.0,
            module_name: module_name.to_string(),
            config_name: config_name.to_string(),
        }
    }
}

impl Clone for GetConfig {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            module_name: self.module_name.clone(),
            config_name: self.config_name.clone(),
        }
    }
}

impl BasePlanNode for GetConfig {
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