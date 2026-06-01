use crate::core::types::{PasswordInfo, UserAlterInfo, UserInfo};
use crate::core::{RoleType, StorageResult};

use super::context::GraphStorageContext;

pub(crate) fn create_user(ctx: &GraphStorageContext, info: &UserInfo) -> StorageResult<bool> {
    ctx.user_storage.create_user(info)
}

pub(crate) fn drop_user(ctx: &GraphStorageContext, username: &str) -> StorageResult<bool> {
    ctx.user_storage.drop_user(username)
}

pub(crate) fn alter_user(ctx: &GraphStorageContext, info: &UserAlterInfo) -> StorageResult<bool> {
    ctx.user_storage.alter_user(info)
}

pub(crate) fn grant_role(
    ctx: &GraphStorageContext,
    username: &str,
    space_id: u64,
    role: RoleType,
) -> StorageResult<bool> {
    ctx.user_storage.grant_role(username, space_id, role)
}

pub(crate) fn revoke_role(
    ctx: &GraphStorageContext,
    username: &str,
    space_id: u64,
) -> StorageResult<bool> {
    ctx.user_storage.revoke_role(username, space_id)
}

pub(crate) fn change_password(
    ctx: &GraphStorageContext,
    info: &PasswordInfo,
) -> StorageResult<bool> {
    ctx.user_storage.change_password(info)
}
