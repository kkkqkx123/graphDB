//! 角色操作相关的计划节点
//! 包括创建/删除角色等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 创建角色计划节点
#[derive(Debug, Clone)]
pub struct CreateRole {
    pub role_name: String,
    pub if_not_exists: bool,
}

impl CreateRole {
    pub fn new(role_name: &str, if_not_exists: bool) -> Self {
        Self {
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

impl From<CreateRole> for PlanNodeEnum {
    fn from(role: CreateRole) -> Self {
        PlanNodeEnum::CreateRole(role)
    }
}

/// 删除角色计划节点
#[derive(Debug, Clone)]
pub struct DropRole {
    pub if_exist: bool,
    pub role_name: String,
}

impl DropRole {
    pub fn new(if_exist: bool, role_name: &str) -> Self {
        Self {
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

impl From<DropRole> for PlanNodeEnum {
    fn from(role: DropRole) -> Self {
        PlanNodeEnum::DropRole(Arc::new(role))
    }
}

/// 授予角色计划节点
#[derive(Debug, Clone)]
pub struct GrantRole {
    pub role_name: String,
    pub username: String,
}

impl GrantRole {
    pub fn new(role_name: &str, username: &str) -> Self {
        Self {
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

impl From<GrantRole> for PlanNodeEnum {
    fn from(role: GrantRole) -> Self {
        PlanNodeEnum::GrantRole(Arc::new(role))
    }
}

/// 撤销角色计划节点
#[derive(Debug, Clone)]
pub struct RevokeRole {
    pub role_name: String,
    pub username: String,
}

impl RevokeRole {
    pub fn new(role_name: &str, username: &str) -> Self {
        Self {
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

impl From<RevokeRole> for PlanNodeEnum {
    fn from(role: RevokeRole) -> Self {
        PlanNodeEnum::RevokeRole(Arc::new(role))
    }
}

/// 显示角色计划节点
#[derive(Debug, Clone)]
pub struct ShowRoles;

impl ShowRoles {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowRoles> for PlanNodeEnum {
    fn from(roles: ShowRoles) -> Self {
        PlanNodeEnum::ShowRoles(Arc::new(roles))
    }
}
