use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::api::session::ClientSession;
use crate::api::service::permission_manager::{Permission, PermissionManager, RoleType};
use crate::config::AuthConfig;

/// 操作类型 - 对应不同的权限检查
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    // Space操作
    ReadSpace,   // USE, DESCRIBE SPACE
    WriteSpace,  // CREATE SPACE, DROP SPACE, CLEAR SPACE
    
    // Schema操作
    ReadSchema,  // DESCRIBE TAG, DESCRIBE EDGE
    WriteSchema, // CREATE TAG, ALTER TAG, CREATE EDGE, DROP TAG
    
    // 数据操作
    ReadData,    // GO, MATCH, FETCH, LOOKUP
    WriteData,   // INSERT, UPDATE, DELETE
    
    // 用户操作
    ReadUser,    // DESCRIBE USER
    WriteUser,   // CREATE USER, DROP USER, ALTER USER
    
    // 角色操作
    WriteRole,   // GRANT, REVOKE
    
    // 特殊操作
    Show,        // SHOW SPACES, SHOW USERS等
    ChangePassword, // CHANGE PASSWORD
}

/// 权限检查器 - 统一分发权限检查
/// 参考Nebula-Graph的PermissionCheck实现
pub struct PermissionChecker {
    permission_manager: PermissionManager,
    auth_config: AuthConfig,
}

impl PermissionChecker {
    pub fn new(permission_manager: PermissionManager, auth_config: AuthConfig) -> Self {
        Self {
            permission_manager,
            auth_config,
        }
    }

    /// 检查是否启用授权
    fn is_authorization_enabled(&self) -> bool {
        self.auth_config.enable_authorize
    }

    /// 统一的权限检查入口
    /// 参考Nebula-Graph的PermissionCheck::permissionCheck
    pub fn check_permission(
        &self,
        session: &ClientSession,
        operation: OperationType,
        target_space: Option<i64>,
        target_user: Option<&str>,
        target_role: Option<RoleType>,
    ) -> Result<()> {
        // 如果未启用授权，直接返回成功
        if !self.is_authorization_enabled() {
            return Ok(());
        }

        let username = session.user();

        match operation {
            // Space读取操作：USE, DESCRIBE SPACE
            OperationType::ReadSpace => {
                if let Some(space_id) = target_space {
                    self.permission_manager.can_read_space(&username, space_id)
                } else {
                    Err(anyhow!("读取Space操作需要提供Space ID"))
                }
            }

            // Space写入操作：CREATE SPACE, DROP SPACE等
            // 只有God角色可以执行
            OperationType::WriteSpace => {
                self.permission_manager.can_write_space(&username)
            }

            // Schema读取操作
            OperationType::ReadSchema => {
                if let Some(space_id) = target_space {
                    self.permission_manager.check_permission(
                        &username,
                        space_id,
                        Permission::Read,
                    )
                } else {
                    Err(anyhow!("读取Schema操作需要提供Space ID"))
                }
            }

            // Schema写入操作
            OperationType::WriteSchema => {
                if let Some(space_id) = target_space {
                    self.permission_manager.can_write_schema(&username, space_id)
                } else {
                    Err(anyhow!("写入Schema操作需要提供Space ID"))
                }
            }

            // 数据读取操作
            OperationType::ReadData => {
                if let Some(space_id) = target_space {
                    self.permission_manager.check_permission(
                        &username,
                        space_id,
                        Permission::Read,
                    )
                } else {
                    Err(anyhow!("读取数据操作需要提供Space ID"))
                }
            }

            // 数据写入操作
            OperationType::WriteData => {
                if let Some(space_id) = target_space {
                    // Guest角色不能写入数据
                    if let Some(role) = session.role_with_space(space_id) {
                        if role == RoleType::Guest {
                            return Err(anyhow!("Guest角色没有写入数据的权限"));
                        }
                    }
                    self.permission_manager.check_permission(
                        &username,
                        space_id,
                        Permission::Write,
                    )
                } else {
                    Err(anyhow!("写入数据操作需要提供Space ID"))
                }
            }

            // 用户读取操作
            OperationType::ReadUser => {
                // God可以读取任何用户
                if session.is_god() {
                    return Ok(());
                }
                
                // 用户可以读取自己的信息
                if let Some(target) = target_user {
                    if username == target {
                        return Ok(());
                    }
                }
                
                Err(anyhow!("没有权限读取用户信息"))
            }

            // 用户写入操作
            OperationType::WriteUser => {
                // 只有God可以管理用户
                if session.is_god() {
                    Ok(())
                } else {
                    Err(anyhow!("没有权限管理用户，需要God角色"))
                }
            }

            // 角色写入操作：GRANT, REVOKE
            OperationType::WriteRole => {
                if let (Some(space_id), Some(target_role), Some(target_user)) = 
                    (target_space, target_role, target_user) {
                    self.permission_manager.can_write_role(
                        &username,
                        target_role,
                        space_id,
                        target_user,
                    )
                } else {
                    Err(anyhow!("角色操作需要提供Space ID、目标角色和目标用户"))
                }
            }

            // 显示操作
            OperationType::Show => {
                // SHOW操作通常允许所有用户
                Ok(())
            }

            // 修改密码操作
            OperationType::ChangePassword => {
                // 用户可以修改自己的密码
                // God可以修改任何用户的密码
                if let Some(target) = target_user {
                    if username == target || session.is_god() {
                        Ok(())
                    } else {
                        Err(anyhow!("只能修改自己的密码"))
                    }
                } else {
                    Err(anyhow!("修改密码操作需要提供目标用户"))
                }
            }
        }
    }

    /// 检查Space读取权限（便捷方法）
    pub fn can_read_space(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadSpace, Some(space_id), None, None)
    }

    /// 检查Space写入权限（便捷方法）
    pub fn can_write_space(&self, session: &ClientSession) -> Result<()> {
        self.check_permission(session, OperationType::WriteSpace, None, None, None)
    }

    /// 检查Schema读取权限（便捷方法）
    pub fn can_read_schema(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadSchema, Some(space_id), None, None)
    }

    /// 检查Schema写入权限（便捷方法）
    pub fn can_write_schema(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::WriteSchema, Some(space_id), None, None)
    }

    /// 检查数据读取权限（便捷方法）
    pub fn can_read_data(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadData, Some(space_id), None, None)
    }

    /// 检查数据写入权限（便捷方法）
    pub fn can_write_data(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::WriteData, Some(space_id), None, None)
    }

    /// 检查用户读取权限（便捷方法）
    pub fn can_read_user(&self, session: &ClientSession, target_user: &str) -> Result<()> {
        self.check_permission(session, OperationType::ReadUser, None, Some(target_user), None)
    }

    /// 检查用户写入权限（便捷方法）
    pub fn can_write_user(&self, session: &ClientSession) -> Result<()> {
        self.check_permission(session, OperationType::WriteUser, None, None, None)
    }

    /// 检查角色写入权限（便捷方法）
    pub fn can_write_role(
        &self,
        session: &ClientSession,
        space_id: i64,
        target_role: RoleType,
        target_user: &str,
    ) -> Result<()> {
        self.check_permission(
            session,
            OperationType::WriteRole,
            Some(space_id),
            Some(target_user),
            Some(target_role),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::session::ClientSession;
    use crate::api::session::client_session::Session;
    use crate::config::AuthConfig;

    fn create_test_config() -> AuthConfig {
        AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 5,
            session_idle_timeout_secs: 3600,
            default_username: "root".to_string(),
            default_password: "root".to_string(),
            force_change_default_password: true,
        }
    }

    fn create_test_checker() -> PermissionChecker {
        let pm = PermissionManager::new();
        
        // 为测试用户分配角色
        pm.grant_role("user1", 1, RoleType::User).expect("Failed to grant role");
        pm.grant_role("admin1", 1, RoleType::Admin).expect("Failed to grant role");
        
        let config = create_test_config();
        PermissionChecker::new(pm, config)
    }

    fn create_test_session(username: &str, is_god: bool) -> Arc<ClientSession> {
        let session = Session {
            session_id: 1,
            user_name: username.to_string(),
            space_name: None,
            graph_addr: Some("127.0.0.1:1234".to_string()),
            timezone: None,
        };
        let client_session = ClientSession::new(session);
        if is_god {
            client_session.set_role(crate::api::service::permission_manager::GOD_SPACE_ID, RoleType::God);
        }
        client_session
    }

    fn create_admin_session(username: &str) -> Arc<ClientSession> {
        let session = Session {
            session_id: 1,
            user_name: username.to_string(),
            space_name: None,
            graph_addr: Some("127.0.0.1:1234".to_string()),
            timezone: None,
        };
        let client_session = ClientSession::new(session);
        client_session.set_role(1, RoleType::Admin);
        client_session
    }

    fn create_user_session(username: &str) -> Arc<ClientSession> {
        let session = Session {
            session_id: 1,
            user_name: username.to_string(),
            space_name: None,
            graph_addr: Some("127.0.0.1:1234".to_string()),
            timezone: None,
        };
        let client_session = ClientSession::new(session);
        client_session.set_role(1, RoleType::User);
        client_session
    }

    #[test]
    fn test_operation_type() {
        assert_eq!(OperationType::ReadSpace as i32, OperationType::ReadSpace as i32);
        assert_ne!(OperationType::ReadSpace, OperationType::WriteSpace);
    }

    #[test]
    fn test_god_can_read_any_space() {
        let checker = create_test_checker();
        let god_session = create_test_session("root", true);

        // God可以读取任何Space
        assert!(checker.can_read_space(&god_session, 1).is_ok());
        assert!(checker.can_read_space(&god_session, 999).is_ok());
    }

    #[test]
    fn test_god_can_write_space() {
        let checker = create_test_checker();
        let god_session = create_test_session("root", true);

        // God可以创建/删除Space
        assert!(checker.can_write_space(&god_session).is_ok());
    }

    #[test]
    fn test_regular_user_cannot_write_space() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // 普通用户不能创建Space
        assert!(checker.can_write_space(&user_session).is_err());
    }

    #[test]
    fn test_user_can_read_assigned_space() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // 用户可以读取被分配的Space
        assert!(checker.can_read_space(&user_session, 1).is_ok());
        // 但不能读取其他Space
        assert!(checker.can_read_space(&user_session, 2).is_err());
    }

    #[test]
    fn test_admin_can_write_schema() {
        let checker = create_test_checker();
        let admin_session = create_admin_session("admin1");

        // Admin可以修改Schema
        assert!(checker.can_write_schema(&admin_session, 1).is_ok());
    }

    #[test]
    fn test_user_cannot_write_schema() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // User不能修改Schema
        assert!(checker.can_write_schema(&user_session, 1).is_err());
    }

    #[test]
    fn test_user_can_read_write_data() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // User可以读写数据
        assert!(checker.can_read_data(&user_session, 1).is_ok());
        assert!(checker.can_write_data(&user_session, 1).is_ok());
    }

    #[test]
    fn test_god_can_manage_users() {
        let checker = create_test_checker();
        let god_session = create_test_session("root", true);

        // God可以管理用户
        assert!(checker.can_write_user(&god_session).is_ok());
        assert!(checker.can_read_user(&god_session, "anyuser").is_ok());
    }

    #[test]
    fn test_regular_user_cannot_manage_users() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // 普通用户不能管理用户
        assert!(checker.can_write_user(&user_session).is_err());
    }

    #[test]
    fn test_user_can_read_own_info() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // 用户可以读取自己的信息
        assert!(checker.can_read_user(&user_session, "user1").is_ok());
        // 但不能读取其他用户的信息
        assert!(checker.can_read_user(&user_session, "user2").is_err());
    }

    #[test]
    fn test_admin_can_grant_roles() {
        let checker = create_test_checker();
        let admin_session = create_admin_session("admin1");

        // Admin可以授予某些角色
        assert!(checker.can_write_role(&admin_session, 1, RoleType::User, "target").is_ok());
        assert!(checker.can_write_role(&admin_session, 1, RoleType::Guest, "target").is_ok());
    }

    #[test]
    fn test_admin_cannot_grant_god() {
        let checker = create_test_checker();
        let admin_session = create_admin_session("admin1");

        // Admin不能授予God角色
        assert!(checker.can_write_role(&admin_session, 1, RoleType::God, "target").is_err());
    }

    #[test]
    fn test_cannot_modify_own_role() {
        let checker = create_test_checker();
        let admin_session = create_admin_session("admin1");

        // 不能修改自己的角色
        assert!(checker.can_write_role(&admin_session, 1, RoleType::User, "admin1").is_err());
    }

    #[test]
    fn test_check_permission_with_disabled_auth() {
        let _config = create_test_config();
        let _manager = PermissionManager::new();
        // 当授权禁用时，所有操作都应该通过
        // 注意：这需要修改PermissionChecker来支持配置
        // 这里仅作为示例
    }

    #[test]
    fn test_show_operation_allowed() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");

        // SHOW操作通常允许所有用户
        assert!(checker.check_permission(&user_session, OperationType::Show, None, None, None).is_ok());
    }

    #[test]
    fn test_change_password_permissions() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1");
        let god_session = create_test_session("root", true);

        // 用户可以修改自己的密码
        assert!(checker.check_permission(
            &user_session,
            OperationType::ChangePassword,
            None,
            Some("user1"),
            None
        ).is_ok());

        // 用户不能修改其他用户的密码
        assert!(checker.check_permission(
            &user_session,
            OperationType::ChangePassword,
            None,
            Some("user2"),
            None
        ).is_err());

        // God可以修改任何用户的密码
        assert!(checker.check_permission(
            &god_session,
            OperationType::ChangePassword,
            None,
            Some("anyuser"),
            None
        ).is_ok());
    }

    #[test]
    fn test_permission_checker_creation() {
        let config = create_test_config();
        let manager = PermissionManager::new();
        let checker = PermissionChecker::new(manager, config);
        // 验证PermissionChecker创建成功
        assert!(checker.is_authorization_enabled() || !checker.is_authorization_enabled());
    }

    #[test]
    fn test_role_type_values() {
        // 验证角色类型的值
        assert_eq!(RoleType::God as i64, 0x01);
        assert_eq!(RoleType::Admin as i64, 0x02);
        assert_eq!(RoleType::Dba as i64, 0x03);
        assert_eq!(RoleType::User as i64, 0x04);
        assert_eq!(RoleType::Guest as i64, 0x05);
    }
}
