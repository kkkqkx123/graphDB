//! 角色操作相关的计划节点
//! 包括创建/删除角色、授权/撤销权限等操作

use crate::query::planner::plan::core::{
    PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use crate::query::validator::Variable;

// 定义角色类型
#[derive(Debug, Clone)]
pub enum RoleType {
    Guest,
    User,
    Admin,
    DBA,
}

/// 创建角色计划节点
#[derive(Debug)]
pub struct CreateRole {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub space_name: String,
    pub role_type: RoleType,
}

impl CreateRole {
    pub fn new(id: i64, username: &str, space_name: &str, role_type: RoleType) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateRole,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            space_name: space_name.to_string(),
            role_type,
        }
    }
}

impl Clone for CreateRole {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            space_name: self.space_name.clone(),
            role_type: self.role_type.clone(),
        }
    }
}

impl BasePlanNode for CreateRole {
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
}

/// 删除角色计划节点
#[derive(Debug)]
pub struct DropRole {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub space_name: String,
}

impl DropRole {
    pub fn new(id: i64, username: &str, space_name: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropRole,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            space_name: space_name.to_string(),
        }
    }
}

impl Clone for DropRole {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            space_name: self.space_name.clone(),
        }
    }
}

impl BasePlanNode for DropRole {
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
}

/// 授权角色计划节点
#[derive(Debug)]
pub struct GrantRole {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub space_name: String,
    pub role_type: RoleType,
}

impl GrantRole {
    pub fn new(id: i64, username: &str, space_name: &str, role_type: RoleType) -> Self {
        Self {
            id,
            kind: PlanNodeKind::GrantRole,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            space_name: space_name.to_string(),
            role_type,
        }
    }
}

impl Clone for GrantRole {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            space_name: self.space_name.clone(),
            role_type: self.role_type.clone(),
        }
    }
}

impl BasePlanNode for GrantRole {
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
}

/// 撤销角色计划节点
#[derive(Debug)]
pub struct RevokeRole {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub space_name: String,
    pub role_type: RoleType,
}

impl RevokeRole {
    pub fn new(id: i64, username: &str, space_name: &str, role_type: RoleType) -> Self {
        Self {
            id,
            kind: PlanNodeKind::RevokeRole,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            space_name: space_name.to_string(),
            role_type,
        }
    }
}

impl Clone for RevokeRole {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            space_name: self.space_name.clone(),
            role_type: self.role_type.clone(),
        }
    }
}

impl BasePlanNode for RevokeRole {
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
}

/// 显示用户角色计划节点
#[derive(Debug)]
pub struct ShowRoles {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}

impl ShowRoles {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShowRoles,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Account".to_string(),
                "Space".to_string(),
                "Role".to_string(),
            ],
            cost: 0.0,
            username: username.to_string(),
        }
    }
}

impl Clone for ShowRoles {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
        }
    }
}

impl BasePlanNode for ShowRoles {
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
}
