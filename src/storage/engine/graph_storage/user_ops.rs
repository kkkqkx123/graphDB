//! User Operations
//!
//! Provides user and role management operations.

use crate::core::types::{PasswordInfo, UserAlterInfo, UserInfo};
use crate::core::{RoleType, StorageResult};

use super::context::GraphStorageContext;

pub struct UserOps<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> UserOps<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn create_user(&self, info: &UserInfo) -> StorageResult<bool> {
        self.ctx.user_storage.create_user(info)
    }

    pub fn drop_user(&self, username: &str) -> StorageResult<bool> {
        self.ctx.user_storage.drop_user(username)
    }

    pub fn alter_user(&self, info: &UserAlterInfo) -> StorageResult<bool> {
        self.ctx.user_storage.alter_user(info)
    }

    pub fn grant_role(&self, username: &str, space_id: u64, role: RoleType) -> StorageResult<bool> {
        self.ctx.user_storage.grant_role(username, space_id, role)
    }

    pub fn revoke_role(&self, username: &str, space_id: u64) -> StorageResult<bool> {
        self.ctx.user_storage.revoke_role(username, space_id)
    }

    pub fn change_password(&self, info: &PasswordInfo) -> StorageResult<bool> {
        self.ctx.user_storage.change_password(info)
    }
}
