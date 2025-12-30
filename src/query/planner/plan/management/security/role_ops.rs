//! 角色操作相关的计划节点
//! 包括创建/删除角色等操作

use crate::query::planner::plan::core::nodes::{ManagementNode, ManagementNodeEnum};

/// 创建角色计划节点
#[derive(Debug, Clone)]
pub struct CreateRole {
    pub id: i64,
    pub cost: f64,
    pub role_name: String,
    pub if_not_exists: bool,
}

impl CreateRole {
    pub fn new(id: i64, role_name: &str, if_not_exists: bool) -> Self {
        Self {
            id,
            cost: 0.0,
            role_name: role_name.to_string(),
            if_not_exists,
        }
    }

    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
}

impl ManagementNode for CreateRole {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateRole"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateRole(self)
    }
}

/// 删除角色计划节点
#[derive(Debug, Clone)]
pub struct DropRole {
    pub id: i64,
    pub cost: f64,
    pub if_exist: bool,
    pub role_name: String,
}

impl DropRole {
    pub fn new(id: i64, if_exist: bool, role_name: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            if_exist,
            role_name: role_name.to_string(),
        }
    }

    pub fn if_exist(&self) -> bool {
        self.if_exist
    }

    pub fn role_name(&self) -> &str {
        &self.role_name
    }
}

impl ManagementNode for DropRole {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropRole"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropRole(self)
    }
}

/// 授予角色计划节点
#[derive(Debug, Clone)]
pub struct GrantRole {
    pub id: i64,
    pub cost: f64,
    pub role_name: String,
    pub username: String,
}

impl GrantRole {
    pub fn new(id: i64, role_name: &str, username: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            role_name: role_name.to_string(),
            username: username.to_string(),
        }
    }

    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl ManagementNode for GrantRole {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "GrantRole"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::GrantRole(self)
    }
}

/// 撤销角色计划节点
#[derive(Debug, Clone)]
pub struct RevokeRole {
    pub id: i64,
    pub cost: f64,
    pub role_name: String,
    pub username: String,
}

impl RevokeRole {
    pub fn new(id: i64, role_name: &str, username: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            role_name: role_name.to_string(),
            username: username.to_string(),
        }
    }

    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl ManagementNode for RevokeRole {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "RevokeRole"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::RevokeRole(self)
    }
}

/// 显示角色计划节点
#[derive(Debug, Clone)]
pub struct ShowRoles {
    pub id: i64,
    pub cost: f64,
}

impl ShowRoles {
    pub fn new(id: i64) -> Self {
        Self { id, cost: 0.0 }
    }
}

impl ManagementNode for ShowRoles {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowRoles"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowRoles(self)
    }
}
