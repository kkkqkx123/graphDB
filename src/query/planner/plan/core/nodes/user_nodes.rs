//! 用户管理节点实现
//!
//! 提供用户管理相关的计划节点定义

use crate::define_plan_node;
use crate::core::types::PasswordInfo;

define_plan_node! {
    pub struct CreateUserNode {
        username: String,
        password: String,
        role: String,
    }
    enum: CreateUser
    input: ZeroInputNode
}

impl CreateUserNode {
    pub fn new(id: i64, username: String, password: String) -> Self {
        Self {
            id,
            username,
            password,
            role: "user".to_string(),
            output_var: None,
            col_names: Vec::new(),
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

define_plan_node! {
    pub struct AlterUserNode {
        username: String,
        new_role: Option<String>,
        is_locked: Option<bool>,
    }
    enum: AlterUser
    input: ZeroInputNode
}

impl AlterUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self {
            id,
            username,
            new_role: None,
            is_locked: None,
            output_var: None,
            col_names: Vec::new(),
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

define_plan_node! {
    pub struct DropUserNode {
        username: String,
    }
    enum: DropUser
    input: ZeroInputNode
}

impl DropUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self {
            id,
            username,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

define_plan_node! {
    pub struct ChangePasswordNode {
        password_info: PasswordInfo,
    }
    enum: ChangePassword
    input: ZeroInputNode
}

impl ChangePasswordNode {
    pub fn new(id: i64, password_info: PasswordInfo) -> Self {
        Self {
            id,
            password_info,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn password_info(&self) -> &PasswordInfo {
        &self.password_info
    }
}
