//! 用户管理节点实现
//!
//! 提供用户管理相关的计划节点定义

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::core::types::metadata::PasswordInfo;
use crate::query::context::validate::types::Variable;

/// 创建用户计划节点
#[derive(Debug, Clone)]
pub struct CreateUserNode {
    id: i64,
    username: String,
    password: String,
    role: String,
}

impl CreateUserNode {
    pub fn new(id: i64, username: String, password: String) -> Self {
        Self {
            id,
            username,
            password,
            role: "user".to_string(),
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}

impl PlanNode for CreateUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateUser(self)
    }
}

/// 修改用户计划节点
#[derive(Debug, Clone)]
pub struct AlterUserNode {
    id: i64,
    username: String,
    new_role: Option<String>,
    is_locked: Option<bool>,
}

impl AlterUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self {
            id,
            username,
            new_role: None,
            is_locked: None,
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.new_role = Some(role);
        self
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn new_role(&self) -> Option<&String> {
        self.new_role.as_ref()
    }

    pub fn is_locked(&self) -> Option<bool> {
        self.is_locked
    }
}

impl PlanNode for AlterUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::AlterUser(self)
    }
}

/// 删除用户计划节点
#[derive(Debug, Clone)]
pub struct DropUserNode {
    id: i64,
    username: String,
}

impl DropUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl PlanNode for DropUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropUser(self)
    }
}

/// 修改密码计划节点
#[derive(Debug, Clone)]
pub struct ChangePasswordNode {
    id: i64,
    password_info: PasswordInfo,
}

impl ChangePasswordNode {
    pub fn new(id: i64, password_info: PasswordInfo) -> Self {
        Self {
            id,
            password_info,
        }
    }

    pub fn password_info(&self) -> &PasswordInfo {
        &self.password_info
    }
}

impl PlanNode for ChangePasswordNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ChangePassword"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ChangePassword(self)
    }
}
