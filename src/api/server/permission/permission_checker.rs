use anyhow::{anyhow, Result};

use crate::api::server::session::ClientSession;
use crate::api::server::permission::PermissionManager;
use crate::core::{Permission, RoleType};
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

/// 权限检查器 - 业务层
/// 
/// 职责：
/// 1. 提供统一的权限检查入口
/// 2. 实现业务逻辑判断（如God角色优先、Guest角色限制等）
/// 3. 管理授权配置（是否启用授权）
/// 4. 组合 PermissionManager 的基础操作完成复杂权限检查
/// 
/// 设计原则：
/// - 所有业务逻辑在此层实现
/// - 不直接操作权限数据，通过 PermissionManager 访问
pub struct PermissionChecker {
    permission_manager: PermissionManager,
    auth_config: AuthConfig,
}

impl PermissionChecker {
    /// 创建新的权限检查器
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

    // ==================== 统一权限检查入口 ====================

    /// 统一的权限检查入口
    /// 
    /// 根据操作类型进行相应的权限检查，包含业务逻辑：
    /// - God 角色拥有所有权限
    /// - Guest 角色限制写入
    /// - 用户只能修改自己的密码
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

        // God 角色拥有所有权限（除修改密码需要特殊检查外）
        if self.permission_manager.is_god(&username) {
            match operation {
                // 即使是 God，修改密码时也只能修改自己或明确授权的用户
                OperationType::ChangePassword => {
                    return self.check_change_password(&username, target_user, session);
                }
                _ => return Ok(()),
            }
        }

        match operation {
            // Space读取操作：USE, DESCRIBE SPACE
            OperationType::ReadSpace => {
                self.check_read_space(&username, target_space)
            }

            // Space写入操作：CREATE SPACE, DROP SPACE等
            // 只有God角色可以执行
            OperationType::WriteSpace => {
                Err(anyhow!("Permission denied: only GOD role can create/drop spaces"))
            }

            // Schema读取操作
            OperationType::ReadSchema => {
                self.check_read_schema(&username, target_space)
            }

            // Schema写入操作
            OperationType::WriteSchema => {
                self.check_write_schema(&username, target_space)
            }

            // 数据读取操作
            OperationType::ReadData => {
                self.check_read_data(&username, target_space)
            }

            // 数据写入操作
            OperationType::WriteData => {
                self.check_write_data(session, &username, target_space)
            }

            // 用户读取操作
            OperationType::ReadUser => {
                self.check_read_user(&username, target_user, session)
            }

            // 用户写入操作
            OperationType::WriteUser => {
                Err(anyhow!("Permission denied: only GOD role can manage users"))
            }

            // 角色写入操作：GRANT, REVOKE
            OperationType::WriteRole => {
                self.check_write_role(&username, target_space, target_role)
            }

            // 显示操作
            OperationType::Show => {
                // SHOW操作通常允许所有认证用户
                Ok(())
            }

            // 修改密码操作
            OperationType::ChangePassword => {
                self.check_change_password(&username, target_user, session)
            }
        }
    }

    // ==================== 具体业务逻辑检查方法 ====================

    /// 检查Space读取权限
    fn check_read_space(&self, username: &str, target_space: Option<i64>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("读取Space操作需要提供Space ID"))?;
        
        // 使用 PermissionManager 的基础检查
        self.permission_manager.check_permission(username, space_id, Permission::Read)
    }

    /// 检查Schema读取权限
    fn check_read_schema(&self, username: &str, target_space: Option<i64>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("读取Schema操作需要提供Space ID"))?;
        
        self.permission_manager.check_permission(username, space_id, Permission::Read)
    }

    /// 检查Schema写入权限
    /// 业务逻辑：只有 God 和 Admin 可以写入 Schema
    fn check_write_schema(&self, username: &str, target_space: Option<i64>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("写入Schema操作需要提供Space ID"))?;

        // 检查是否是管理员
        if !self.permission_manager.is_admin(username) {
            return Err(anyhow!(
                "Permission denied: write schema in space {} for user {}", 
                space_id, username
            ));
        }

        // 管理员需要 Write 权限
        self.permission_manager.check_permission(username, space_id, Permission::Write)
    }

    /// 检查数据读取权限
    fn check_read_data(&self, username: &str, target_space: Option<i64>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("读取数据操作需要提供Space ID"))?;
        
        self.permission_manager.check_permission(username, space_id, Permission::Read)
    }

    /// 检查数据写入权限
    /// 业务逻辑：Guest角色不能写入数据
    fn check_write_data(&self, session: &ClientSession, username: &str, target_space: Option<i64>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("写入数据操作需要提供Space ID"))?;

        // Guest角色不能写入数据
        if let Some(role) = session.role_with_space(space_id) {
            if role == RoleType::Guest {
                return Err(anyhow!("Guest角色没有写入数据的权限"));
            }
        }

        self.permission_manager.check_permission(username, space_id, Permission::Write)
    }

    /// 检查用户读取权限
    /// 业务逻辑：用户可以读取自己的信息
    fn check_read_user(&self, username: &str, target_user: Option<&str>, _session: &ClientSession) -> Result<()> {
        // 用户可以读取自己的信息
        if let Some(target) = target_user {
            if username == target {
                return Ok(());
            }
        }
        
        // Admin 可以读取其他用户信息
        if self.permission_manager.is_admin(username) {
            return Ok(());
        }
        
        Err(anyhow!("没有权限读取用户信息"))
    }

    /// 检查角色写入权限
    /// 业务逻辑：需要 God 或 Admin 角色，且目标角色不能高于操作者
    fn check_write_role(&self, username: &str, target_space: Option<i64>, target_role: Option<RoleType>) -> Result<()> {
        let space_id = target_space
            .ok_or_else(|| anyhow!("角色操作需要提供Space ID"))?;
        let role = target_role
            .ok_or_else(|| anyhow!("角色操作需要提供目标角色"))?;

        // 只有 Admin 及以上可以管理角色
        if !self.permission_manager.is_admin(username) {
            return Err(anyhow!("Permission denied: only Admin or God can manage roles"));
        }

        // 检查是否可以授予目标角色
        if !self.permission_manager.can_grant_role(username, space_id, role) {
            return Err(anyhow!("Permission denied: cannot grant role {:?}", role));
        }

        Ok(())
    }

    /// 检查修改密码权限
    /// 业务逻辑：用户可以修改自己的密码，God可以修改任何用户的密码
    fn check_change_password(&self, username: &str, target_user: Option<&str>, session: &ClientSession) -> Result<()> {
        let target = target_user.ok_or_else(|| anyhow!("修改密码操作需要提供目标用户"))?;
        
        // 用户可以修改自己的密码
        if username == target {
            return Ok(());
        }
        
        // God可以修改任何用户的密码
        if session.is_god() {
            return Ok(());
        }
        
        Err(anyhow!("只能修改自己的密码"))
    }

    // ==================== 便捷方法（供外部调用） ====================

    /// 检查Space读取权限
    pub fn can_read_space(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadSpace, Some(space_id), None, None)
    }

    /// 检查Space写入权限
    pub fn can_write_space(&self, session: &ClientSession) -> Result<()> {
        self.check_permission(session, OperationType::WriteSpace, None, None, None)
    }

    /// 检查Schema读取权限
    pub fn can_read_schema(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadSchema, Some(space_id), None, None)
    }

    /// 检查Schema写入权限
    pub fn can_write_schema(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::WriteSchema, Some(space_id), None, None)
    }

    /// 检查数据读取权限
    pub fn can_read_data(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::ReadData, Some(space_id), None, None)
    }

    /// 检查数据写入权限
    pub fn can_write_data(&self, session: &ClientSession, space_id: i64) -> Result<()> {
        self.check_permission(session, OperationType::WriteData, Some(space_id), None, None)
    }

    /// 检查用户读取权限
    pub fn can_read_user(&self, session: &ClientSession, target_user: &str) -> Result<()> {
        self.check_permission(session, OperationType::ReadUser, None, Some(target_user), None)
    }

    /// 检查用户写入权限
    pub fn can_write_user(&self, session: &ClientSession) -> Result<()> {
        self.check_permission(session, OperationType::WriteUser, None, None, None)
    }

    /// 检查角色写入权限
    pub fn can_write_role(
        &self,
        session: &ClientSession,
        space_id: i64,
        target_role: RoleType,
    ) -> Result<()> {
        self.check_permission(
            session,
            OperationType::WriteRole,
            Some(space_id),
            None,
            Some(target_role),
        )
    }

    /// 检查修改密码权限
    pub fn can_change_password(&self, session: &ClientSession, target_user: &str) -> Result<()> {
        self.check_permission(
            session,
            OperationType::ChangePassword,
            None,
            Some(target_user),
            None,
        )
    }

    /// 检查Show操作权限
    pub fn can_show(&self, session: &ClientSession) -> Result<()> {
        self.check_permission(session, OperationType::Show, None, None, None)
    }

    // ==================== 获取内部组件（用于高级场景） ====================

    /// 获取权限管理器的引用
    pub fn permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// 获取配置
    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::server::session::{ClientSession, Session};
    use crate::api::server::permission::GOD_SPACE_ID;
    use crate::config::AuthConfig;
    use std::sync::Arc;

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
        pm.grant_role("guest1", 1, RoleType::Guest).expect("Failed to grant role");
        
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
            client_session.set_role(GOD_SPACE_ID, RoleType::God);
        }
        client_session
    }

    fn create_user_session(username: &str, role: RoleType, space_id: i64) -> Arc<ClientSession> {
        let session = Session {
            session_id: 1,
            user_name: username.to_string(),
            space_name: None,
            graph_addr: Some("127.0.0.1:1234".to_string()),
            timezone: None,
        };
        let client_session = ClientSession::new(session);
        client_session.set_role(space_id, role);
        client_session
    }

    #[test]
    fn test_disabled_authorization() {
        let mut config = create_test_config();
        config.enable_authorize = false;
        
        let pm = PermissionManager::new();
        let checker = PermissionChecker::new(pm, config);
        let session = create_test_session("any_user", false);
        
        // 禁用授权时，任何操作都应该通过
        assert!(checker.can_write_space(&session).is_ok());
        assert!(checker.can_write_schema(&session, 1).is_ok());
        assert!(checker.can_write_data(&session, 1).is_ok());
    }

    #[test]
    fn test_god_role_has_all_permissions() {
        let checker = create_test_checker();
        let god_session = create_test_session("root", true);
        
        // God 可以执行任何操作
        assert!(checker.can_write_space(&god_session).is_ok());
        assert!(checker.can_write_schema(&god_session, 1).is_ok());
        assert!(checker.can_write_data(&god_session, 1).is_ok());
        assert!(checker.can_write_user(&god_session).is_ok());
        assert!(checker.can_write_role(&god_session, 1, RoleType::Admin).is_ok());
    }

    #[test]
    fn test_user_cannot_write_space() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1", RoleType::User, 1);
        
        // 普通用户不能创建/删除空间
        assert!(checker.can_write_space(&user_session).is_err());
    }

    #[test]
    fn test_user_cannot_write_schema() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1", RoleType::User, 1);
        
        // 普通用户不能修改 Schema
        assert!(checker.can_write_schema(&user_session, 1).is_err());
    }

    #[test]
    fn test_admin_can_write_schema() {
        let checker = create_test_checker();
        let admin_session = create_user_session("admin1", RoleType::Admin, 1);
        
        // Admin 可以修改 Schema
        assert!(checker.can_write_schema(&admin_session, 1).is_ok());
    }

    #[test]
    fn test_guest_cannot_write_data() {
        let checker = create_test_checker();
        let guest_session = create_user_session("guest1", RoleType::Guest, 1);
        
        // Guest 不能写入数据
        assert!(checker.can_write_data(&guest_session, 1).is_err());
        // Guest 可以读取数据
        assert!(checker.can_read_data(&guest_session, 1).is_ok());
    }

    #[test]
    fn test_user_can_read_own_info() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1", RoleType::User, 1);
        
        // 用户可以读取自己的信息
        assert!(checker.can_read_user(&user_session, "user1").is_ok());
        // 用户不能读取其他用户的信息
        assert!(checker.can_read_user(&user_session, "user2").is_err());
    }

    #[test]
    fn test_change_password() {
        let checker = create_test_checker();
        let user_session = create_user_session("user1", RoleType::User, 1);
        
        // 用户可以修改自己的密码
        assert!(checker.can_change_password(&user_session, "user1").is_ok());
        // 用户不能修改其他用户的密码
        assert!(checker.can_change_password(&user_session, "user2").is_err());
    }

    #[test]
    fn test_admin_can_grant_lower_roles() {
        let checker = create_test_checker();
        let admin_session = create_user_session("admin1", RoleType::Admin, 1);
        
        // Admin 可以授予 User 和 Guest 角色
        assert!(checker.can_write_role(&admin_session, 1, RoleType::User).is_ok());
        assert!(checker.can_write_role(&admin_session, 1, RoleType::Guest).is_ok());
        // Admin 不能授予 Admin 或 God 角色
        assert!(checker.can_write_role(&admin_session, 1, RoleType::Admin).is_err());
    }
}
