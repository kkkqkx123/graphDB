//! 用户操作相关的计划节点
//! 包括创建/删除用户等操作

use crate::query::planner::plan::core::nodes::{
    ManagementNode, ManagementNodeClonable, ManagementNodeEnum,
};

/// 创建用户计划节点
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub id: i64,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub if_not_exists: bool,
}

impl CreateUser {
    pub fn new(id: i64, username: &str, password: &str, if_not_exists: bool) -> Self {
        Self {
            id,
            cost: 0.0,
            username: username.to_string(),
            password: password.to_string(),
            if_not_exists,
        }
    }
}

impl ManagementNodeClonable for CreateUser {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::CreateUser(cloned)
    }
}

impl ManagementNode for CreateUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateUser"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateUser(self)
    }
}

/// 删除用户计划节点
#[derive(Debug, Clone)]
pub struct DropUser {
    pub id: i64,
    pub cost: f64,
    pub if_exist: bool,
    pub username: String,
}

impl DropUser {
    pub fn new(id: i64, if_exist: bool, username: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            if_exist,
            username: username.to_string(),
        }
    }
}

impl ManagementNodeClonable for DropUser {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::DropUser(cloned)
    }
}

impl ManagementNode for DropUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropUser"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropUser(self)
    }
}

/// 修改用户密码计划节点
#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub id: i64,
    pub cost: f64,
    pub username: String,
    pub new_password: String,
}

impl UpdateUser {
    pub fn new(id: i64, username: &str, new_password: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            username: username.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

impl ManagementNodeClonable for UpdateUser {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::UpdateUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::UpdateUser(cloned)
    }
}

impl ManagementNode for UpdateUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "UpdateUser"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::UpdateUser(self)
    }
}

/// 修改密码计划节点
#[derive(Debug, Clone)]
pub struct ChangePassword {
    pub id: i64,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub fn new(id: i64, username: &str, password: &str, new_password: &str) -> Self {
        Self {
            id,
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

impl ManagementNodeClonable for ChangePassword {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::ChangePassword(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::ChangePassword(cloned)
    }
}

impl ManagementNode for ChangePassword {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ChangePassword"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ChangePassword(self)
    }
}

/// 列出用户计划节点
#[derive(Debug, Clone)]
pub struct ListUsers {
    pub id: i64,
    pub cost: f64,
}

impl ListUsers {
    pub fn new(id: i64) -> Self {
        Self { id, cost: 0.0 }
    }
}

impl ManagementNodeClonable for ListUsers {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::ListUsers(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::ListUsers(cloned)
    }
}

impl ManagementNode for ListUsers {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ListUsers"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ListUsers(self)
    }
}

/// 列出用户角色计划节点
#[derive(Debug, Clone)]
pub struct ListUserRoles {
    pub id: i64,
    pub cost: f64,
    pub username: String,
}

impl ListUserRoles {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            username: username.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl ManagementNodeClonable for ListUserRoles {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::ListUserRoles(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::ListUserRoles(cloned)
    }
}

impl ManagementNode for ListUserRoles {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ListUserRoles"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ListUserRoles(self)
    }
}

/// 描述用户计划节点
#[derive(Debug, Clone)]
pub struct DescribeUser {
    pub id: i64,
    pub cost: f64,
    pub username: String,
}

impl DescribeUser {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            cost: 0.0,
            username: username.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl ManagementNodeClonable for DescribeUser {
    fn clone_management_node(&self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescribeUser(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        ManagementNodeEnum::DescribeUser(cloned)
    }
}

impl ManagementNode for DescribeUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescribeUser"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescribeUser(self)
    }
}
