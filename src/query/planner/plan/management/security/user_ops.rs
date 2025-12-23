//! 用户操作相关的计划节点
//! 包括创建/删除用户等操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties,
    },
};
use crate::query::planner::plan::PlanNodeEnum;

/// 创建用户计划节点
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub if_not_exists: bool,
}

impl CreateUser {
    pub fn new(id: i64, username: &str, password: &str, if_not_exists: bool) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            password: password.to_string(),
            if_not_exists,
        }
    }
}

impl PlanNodeIdentifiable for CreateUser {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "CreateUser"
    }
}

impl PlanNodeProperties for CreateUser {
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

impl PlanNodeDependencies for CreateUser {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for CreateUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::CreateUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::CreateUser(cloned)
    }
}

impl PlanNode for CreateUser {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateUser(self)
    }
}

/// 删除用户计划节点
#[derive(Debug, Clone)]
pub struct DropUser {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exist: bool,
    pub username: String,
}

impl DropUser {
    pub fn new(id: i64, if_exist: bool, username: &str) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exist,
            username: username.to_string(),
        }
    }
}

impl PlanNodeIdentifiable for DropUser {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "DropUser"
    }
}

impl PlanNodeProperties for DropUser {
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

impl PlanNodeDependencies for DropUser {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for DropUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::DropUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::DropUser(cloned)
    }
}

impl PlanNode for DropUser {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropUser(self)
    }
}

/// 修改用户密码计划节点
#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub new_password: String,
}

impl UpdateUser {
    pub fn new(id: i64, username: &str, new_password: &str) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

impl PlanNodeIdentifiable for UpdateUser {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "UpdateUser"
    }
}

impl PlanNodeProperties for UpdateUser {
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

impl PlanNodeDependencies for UpdateUser {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for UpdateUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::UpdateUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::UpdateUser(cloned)
    }
}

impl PlanNode for UpdateUser {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::UpdateUser(self)
    }
}

/// 修改密码计划节点
#[derive(Debug, Clone)]
pub struct ChangePassword {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub fn new(id: i64, username: &str, password: &str, new_password: &str) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            password: password.to_string(),
            new_password: new_password.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn new_password(&self) -> &str {
        &self.new_password
    }
}

impl PlanNodeIdentifiable for ChangePassword {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "ChangePassword"
    }
}

impl PlanNodeProperties for ChangePassword {
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

impl PlanNodeDependencies for ChangePassword {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for ChangePassword {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ChangePassword {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ChangePassword(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ChangePassword(cloned)
    }
}

impl PlanNode for ChangePassword {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ChangePassword(self)
    }
}

/// 列出用户计划节点
#[derive(Debug, Clone)]
pub struct ListUsers {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ListUsers {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Account".to_string()],
            cost: 0.0,
        }
    }
}

impl PlanNodeIdentifiable for ListUsers {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "ListUsers"
    }
}

impl PlanNodeProperties for ListUsers {
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

impl PlanNodeDependencies for ListUsers {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for ListUsers {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ListUsers {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ListUsers(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ListUsers(cloned)
    }
}

impl PlanNode for ListUsers {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ListUsers(self)
    }
}

/// 列出用户角色计划节点
#[derive(Debug, Clone)]
pub struct ListUserRoles {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}

impl ListUserRoles {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
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

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl PlanNodeIdentifiable for ListUserRoles {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "ListUserRoles"
    }
}

impl PlanNodeProperties for ListUserRoles {
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

impl PlanNodeDependencies for ListUserRoles {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for ListUserRoles {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ListUserRoles {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ListUserRoles(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ListUserRoles(cloned)
    }
}

impl PlanNode for ListUserRoles {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ListUserRoles(self)
    }
}

/// 描述用户计划节点
#[derive(Debug, Clone)]
pub struct DescribeUser {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}

impl DescribeUser {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Account".to_string(),
                "Role".to_string(),
                "Time Zone".to_string(),
                "Locked".to_string(),
            ],
            cost: 0.0,
            username: username.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl PlanNodeIdentifiable for DescribeUser {
    fn id(&self) -> i64 {
        self.id
    }
    
    fn name(&self) -> &'static str {
        "DescribeUser"
    }
}

impl PlanNodeProperties for DescribeUser {
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

impl PlanNodeDependencies for DescribeUser {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        self.deps.iter().map(|dep| Box::new(dep.clone())).collect()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
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

impl PlanNodeMutable for DescribeUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DescribeUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::DescribeUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::DescribeUser(cloned)
    }
}

impl PlanNode for DescribeUser {
    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescribeUser(self)
    }
}