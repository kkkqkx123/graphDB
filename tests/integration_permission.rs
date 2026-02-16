//! 权限管理集成测试
//!
//! 测试范围:
//! - PermissionManager - 权限管理器核心功能
//! - PermissionChecker - 权限检查器
//! - Authenticator - 认证器
//! - 角色授予与撤销 (GRANT/REVOKE)
//! - 权限检查场景

mod common;

use std::sync::Arc;

use graphdb::api::service::{
    Authenticator, PasswordAuthenticator, PermissionManager, Permission, RoleType,
    PermissionChecker,
};
use graphdb::api::service::permission_checker::OperationType;
use graphdb::api::service::permission_manager::GOD_SPACE_ID;
use graphdb::api::session::ClientSession;
use graphdb::api::session::client_session::{Session, SpaceInfo};
use graphdb::config::AuthConfig;

// ==================== PermissionManager 核心测试 ====================

#[tokio::test]
async fn test_permission_manager_creation() {
    let pm = PermissionManager::new();

    // root用户应该自动成为God角色
    assert!(pm.is_god("root"), "root用户应该是God角色");
    assert!(pm.is_admin("root"), "root用户应该是Admin角色");
}

#[tokio::test]
async fn test_grant_and_revoke_role() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    // 授予User角色
    pm.grant_role("user1", space_id, RoleType::User)
        .expect("授予User角色应该成功");

    // 验证角色已授予
    let role = pm.get_role("user1", space_id);
    assert_eq!(role, Some(RoleType::User), "应该获取到User角色");

    // 撤销角色
    pm.revoke_role("user1", space_id)
        .expect("撤销角色应该成功");

    // 验证角色已撤销
    let role_after = pm.get_role("user1", space_id);
    assert_eq!(role_after, None, "角色应该已被撤销");
}

#[tokio::test]
async fn test_grant_multiple_roles_to_user() {
    let pm = PermissionManager::new();

    // 给用户在不同Space授予不同角色
    pm.grant_role("multi_role_user", 1, RoleType::Admin)
        .expect("授予Admin角色应该成功");
    pm.grant_role("multi_role_user", 2, RoleType::User)
        .expect("授予User角色应该成功");
    pm.grant_role("multi_role_user", 3, RoleType::Guest)
        .expect("授予Guest角色应该成功");

    // 验证各Space的角色
    assert_eq!(pm.get_role("multi_role_user", 1), Some(RoleType::Admin));
    assert_eq!(pm.get_role("multi_role_user", 2), Some(RoleType::User));
    assert_eq!(pm.get_role("multi_role_user", 3), Some(RoleType::Guest));

    // 列出用户所有角色
    let user_roles = pm.list_user_roles("multi_role_user");
    assert_eq!(user_roles.len(), 3, "用户应该有3个角色");
}

#[tokio::test]
async fn test_list_space_users() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    // 给多个用户授予角色
    pm.grant_role("user1", space_id, RoleType::User).unwrap();
    pm.grant_role("user2", space_id, RoleType::Admin).unwrap();
    pm.grant_role("user3", space_id, RoleType::Guest).unwrap();

    // 列出Space中的所有用户
    let space_users = pm.list_space_users(space_id);
    assert_eq!(space_users.len(), 3, "Space中应该有3个用户");

    // 验证包含正确的用户
    let usernames: Vec<String> = space_users.iter().map(|(name, _)| name.clone()).collect();
    assert!(usernames.contains(&"user1".to_string()));
    assert!(usernames.contains(&"user2".to_string()));
    assert!(usernames.contains(&"user3".to_string()));
}

// ==================== 角色权限检查测试 ====================

#[tokio::test]
async fn test_god_role_has_all_permissions() {
    let pm = PermissionManager::new();

    // God角色拥有所有权限
    assert!(RoleType::God.has_permission(Permission::Read));
    assert!(RoleType::God.has_permission(Permission::Write));
    assert!(RoleType::God.has_permission(Permission::Delete));
    assert!(RoleType::God.has_permission(Permission::Schema));
    assert!(RoleType::God.has_permission(Permission::Admin));

    // God可以访问任何Space
    assert!(pm.can_read_space("root", 1).is_ok());
    assert!(pm.can_read_space("root", 999).is_ok());

    // God可以写入Space
    assert!(pm.can_write_space("root").is_ok());

    // God可以写入Schema
    assert!(pm.can_write_schema("root", 1).is_ok());
}

#[tokio::test]
async fn test_admin_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("admin1", space_id, RoleType::Admin).unwrap();

    // Admin拥有所有权限
    assert!(pm.check_permission("admin1", space_id, Permission::Read).is_ok());
    assert!(pm.check_permission("admin1", space_id, Permission::Write).is_ok());
    assert!(pm.check_permission("admin1", space_id, Permission::Delete).is_ok());
    assert!(pm.check_permission("admin1", space_id, Permission::Schema).is_ok());
    assert!(pm.check_permission("admin1", space_id, Permission::Admin).is_ok());
}

#[tokio::test]
async fn test_dba_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("dba1", space_id, RoleType::Dba).unwrap();

    // Dba拥有读写删和Schema权限
    assert!(pm.check_permission("dba1", space_id, Permission::Read).is_ok());
    assert!(pm.check_permission("dba1", space_id, Permission::Write).is_ok());
    assert!(pm.check_permission("dba1", space_id, Permission::Delete).is_ok());
    assert!(pm.check_permission("dba1", space_id, Permission::Schema).is_ok());

    // Dba没有Admin权限
    assert!(pm.check_permission("dba1", space_id, Permission::Admin).is_err());
}

#[tokio::test]
async fn test_user_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("user1", space_id, RoleType::User).unwrap();

    // User拥有读写删权限
    assert!(pm.check_permission("user1", space_id, Permission::Read).is_ok());
    assert!(pm.check_permission("user1", space_id, Permission::Write).is_ok());
    assert!(pm.check_permission("user1", space_id, Permission::Delete).is_ok());

    // User没有Schema和Admin权限
    assert!(pm.check_permission("user1", space_id, Permission::Schema).is_err());
    assert!(pm.check_permission("user1", space_id, Permission::Admin).is_err());
}

#[tokio::test]
async fn test_guest_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("guest1", space_id, RoleType::Guest).unwrap();

    // Guest只有读权限
    assert!(pm.check_permission("guest1", space_id, Permission::Read).is_ok());

    // Guest没有写、删、Schema、Admin权限
    assert!(pm.check_permission("guest1", space_id, Permission::Write).is_err());
    assert!(pm.check_permission("guest1", space_id, Permission::Delete).is_err());
    assert!(pm.check_permission("guest1", space_id, Permission::Schema).is_err());
    assert!(pm.check_permission("guest1", space_id, Permission::Admin).is_err());
}

// ==================== 角色授予权限测试 ====================

#[tokio::test]
async fn test_god_can_grant_any_role() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    // God可以授予任何角色
    assert!(pm.can_write_role("root", RoleType::God, space_id, "target").is_ok());
    assert!(pm.can_write_role("root", RoleType::Admin, space_id, "target").is_ok());
    assert!(pm.can_write_role("root", RoleType::Dba, space_id, "target").is_ok());
    assert!(pm.can_write_role("root", RoleType::User, space_id, "target").is_ok());
    assert!(pm.can_write_role("root", RoleType::Guest, space_id, "target").is_ok());
}

#[tokio::test]
async fn test_admin_grant_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("admin1", space_id, RoleType::Admin).unwrap();

    // Admin可以授予Dba、User、Guest
    assert!(pm.can_write_role("admin1", RoleType::Dba, space_id, "target").is_ok());
    assert!(pm.can_write_role("admin1", RoleType::User, space_id, "target").is_ok());
    assert!(pm.can_write_role("admin1", RoleType::Guest, space_id, "target").is_ok());

    // Admin不能授予God或Admin
    assert!(pm.can_write_role("admin1", RoleType::God, space_id, "target").is_err());
    assert!(pm.can_write_role("admin1", RoleType::Admin, space_id, "target").is_err());
}

#[tokio::test]
async fn test_dba_grant_role_permissions() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("dba1", space_id, RoleType::Dba).unwrap();

    // Dba可以授予User、Guest
    assert!(pm.can_write_role("dba1", RoleType::User, space_id, "target").is_ok());
    assert!(pm.can_write_role("dba1", RoleType::Guest, space_id, "target").is_ok());

    // Dba不能授予God、Admin、Dba
    assert!(pm.can_write_role("dba1", RoleType::God, space_id, "target").is_err());
    assert!(pm.can_write_role("dba1", RoleType::Admin, space_id, "target").is_err());
    assert!(pm.can_write_role("dba1", RoleType::Dba, space_id, "target").is_err());
}

#[tokio::test]
async fn test_user_cannot_grant_any_role() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("user1", space_id, RoleType::User).unwrap();

    // User不能授予任何角色
    assert!(pm.can_write_role("user1", RoleType::User, space_id, "target").is_err());
    assert!(pm.can_write_role("user1", RoleType::Guest, space_id, "target").is_err());
    assert!(pm.can_write_role("user1", RoleType::Dba, space_id, "target").is_err());
}

#[tokio::test]
async fn test_cannot_modify_own_role() {
    let pm = PermissionManager::new();
    let space_id = 1i64;

    pm.grant_role("admin1", space_id, RoleType::Admin).unwrap();

    // 不能修改自己的角色
    assert!(pm.can_write_role("admin1", RoleType::User, space_id, "admin1").is_err());
}

// ==================== PermissionChecker 测试 ====================

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

fn create_test_session(username: &str) -> Session {
    Session {
        session_id: 1,
        user_name: username.to_string(),
        space_name: None,
        graph_addr: Some("127.0.0.1:1234".to_string()),
        timezone: None,
    }
}

fn create_client_session_with_role(username: &str, space_id: i64, role: RoleType) -> Arc<ClientSession> {
    let session = create_test_session(username);
    let client_session = ClientSession::new(session);
    client_session.set_role(space_id, role);
    client_session
}

#[tokio::test]
async fn test_permission_checker_with_disabled_auth() {
    let pm = PermissionManager::new();
    let mut config = create_test_config();
    config.enable_authorize = false; // 禁用授权

    let checker = PermissionChecker::new(pm, config);
    let session = create_client_session_with_role("user1", 1, RoleType::User);

    // 禁用授权时，所有检查都应该通过
    assert!(checker.can_read_space(&session, 1).is_ok());
    assert!(checker.can_write_space(&session).is_ok());
    assert!(checker.can_write_schema(&session, 1).is_ok());
}

#[tokio::test]
async fn test_permission_checker_space_operations() {
    let pm = PermissionManager::new();
    pm.grant_role("user1", 1, RoleType::User).unwrap();
    pm.grant_role("admin1", 1, RoleType::Admin).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let user_session = create_client_session_with_role("user1", 1, RoleType::User);
    let admin_session = create_client_session_with_role("admin1", 1, RoleType::Admin);

    // User可以读取分配的Space
    assert!(checker.can_read_space(&user_session, 1).is_ok());
    // User不能读取未分配的Space
    assert!(checker.can_read_space(&user_session, 2).is_err());

    // User不能写入Space（创建Space）
    assert!(checker.can_write_space(&user_session).is_err());
    // Admin也不能写入Space（只有God可以）
    assert!(checker.can_write_space(&admin_session).is_err());
}

#[tokio::test]
async fn test_permission_checker_schema_operations() {
    let pm = PermissionManager::new();
    pm.grant_role("admin1", 1, RoleType::Admin).unwrap();
    pm.grant_role("user1", 1, RoleType::User).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let admin_session = create_client_session_with_role("admin1", 1, RoleType::Admin);
    let user_session = create_client_session_with_role("user1", 1, RoleType::User);

    // Admin可以读写Schema
    assert!(checker.can_read_schema(&admin_session, 1).is_ok());
    assert!(checker.can_write_schema(&admin_session, 1).is_ok());

    // User可以读取Schema但不能写入
    assert!(checker.can_read_schema(&user_session, 1).is_ok());
    assert!(checker.can_write_schema(&user_session, 1).is_err());
}

#[tokio::test]
async fn test_permission_checker_data_operations() {
    let pm = PermissionManager::new();
    pm.grant_role("user1", 1, RoleType::User).unwrap();
    pm.grant_role("guest1", 1, RoleType::Guest).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let user_session = create_client_session_with_role("user1", 1, RoleType::User);
    let guest_session = create_client_session_with_role("guest1", 1, RoleType::Guest);

    // User可以读写数据
    assert!(checker.can_read_data(&user_session, 1).is_ok());
    assert!(checker.can_write_data(&user_session, 1).is_ok());

    // Guest只能读取数据
    assert!(checker.can_read_data(&guest_session, 1).is_ok());
    assert!(checker.can_write_data(&guest_session, 1).is_err());
}

#[tokio::test]
async fn test_permission_checker_user_operations() {
    let pm = PermissionManager::new();
    pm.grant_role("admin1", 1, RoleType::Admin).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let god_session = create_client_session_with_role("root", GOD_SPACE_ID, RoleType::God);
    let admin_session = create_client_session_with_role("admin1", 1, RoleType::Admin);

    // God可以管理用户
    assert!(checker.can_write_user(&god_session).is_ok());
    assert!(checker.can_read_user(&god_session, "anyuser").is_ok());

    // Admin不能管理用户（只有God可以）
    assert!(checker.can_write_user(&admin_session).is_err());

    // Admin可以读取自己的信息
    assert!(checker.can_read_user(&admin_session, "admin1").is_ok());
    // Admin不能读取其他用户的信息
    assert!(checker.can_read_user(&admin_session, "otheruser").is_err());
}

#[tokio::test]
async fn test_permission_checker_role_operations() {
    let pm = PermissionManager::new();
    pm.grant_role("admin1", 1, RoleType::Admin).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let god_session = create_client_session_with_role("root", GOD_SPACE_ID, RoleType::God);
    let admin_session = create_client_session_with_role("admin1", 1, RoleType::Admin);

    // God可以授予任何角色
    assert!(checker.can_write_role(&god_session, 1, RoleType::Admin, "target").is_ok());

    // Admin可以授予某些角色
    assert!(checker.can_write_role(&admin_session, 1, RoleType::User, "target").is_ok());
    assert!(checker.can_write_role(&admin_session, 1, RoleType::Guest, "target").is_ok());

    // Admin不能授予God
    assert!(checker.can_write_role(&admin_session, 1, RoleType::God, "target").is_err());

    // Admin不能修改自己的角色
    assert!(checker.can_write_role(&admin_session, 1, RoleType::User, "admin1").is_err());
}

#[tokio::test]
async fn test_permission_checker_change_password() {
    let pm = PermissionManager::new();
    pm.grant_role("user1", 1, RoleType::User).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let god_session = create_client_session_with_role("root", GOD_SPACE_ID, RoleType::God);
    let user_session = create_client_session_with_role("user1", 1, RoleType::User);

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
        Some("otheruser"),
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

#[tokio::test]
async fn test_permission_checker_show_operation() {
    let pm = PermissionManager::new();
    pm.grant_role("guest1", 1, RoleType::Guest).unwrap();

    let config = create_test_config();
    let checker = PermissionChecker::new(pm, config);

    let guest_session = create_client_session_with_role("guest1", 1, RoleType::Guest);

    // SHOW操作通常允许所有用户
    assert!(checker.check_permission(
        &guest_session,
        OperationType::Show,
        None,
        None,
        None
    ).is_ok());
}

// ==================== Authenticator 测试 ====================

#[tokio::test]
async fn test_password_authenticator_success() {
    let config = create_test_config();
    let auth = PasswordAuthenticator::new(
        |_username: &str, _password: &str| Ok(true),
        config,
    );

    assert!(auth.authenticate("user", "pass").is_ok());
}

#[tokio::test]
async fn test_password_authenticator_failure() {
    let config = create_test_config();
    let auth = PasswordAuthenticator::new(
        |_username: &str, _password: &str| Ok(false),
        config,
    );

    assert!(auth.authenticate("user", "wrong_pass").is_err());
}

#[tokio::test]
async fn test_password_authenticator_default() {
    let config = AuthConfig {
        enable_authorize: true,
        failed_login_attempts: 0, // 禁用登录限制
        session_idle_timeout_secs: 3600,
        default_username: "admin".to_string(),
        default_password: "admin123".to_string(),
        force_change_default_password: false,
    };

    let auth = PasswordAuthenticator::new_default(config);

    // 使用正确的默认凭据
    assert!(auth.authenticate("admin", "admin123").is_ok());

    // 使用错误的凭据
    assert!(auth.authenticate("admin", "wrong").is_err());
}

#[tokio::test]
async fn test_password_authenticator_empty_credentials() {
    let config = create_test_config();
    let auth = PasswordAuthenticator::new(
        |_username: &str, _password: &str| Ok(true),
        config,
    );

    // 空用户名
    assert!(auth.authenticate("", "pass").is_err());

    // 空密码
    assert!(auth.authenticate("user", "").is_err());

    // 都为空
    assert!(auth.authenticate("", "").is_err());
}

#[tokio::test]
async fn test_password_authenticator_disabled() {
    let mut config = create_test_config();
    config.enable_authorize = false; // 禁用授权

    let auth = PasswordAuthenticator::new(
        |_username: &str, _password: &str| Ok(false), // 即使验证器返回false
        config,
    );

    // 禁用授权时，任何凭据都应该通过
    assert!(auth.authenticate("any", "any").is_ok());
}

#[tokio::test]
async fn test_password_authenticator_login_attempts_limit() {
    let config = AuthConfig {
        enable_authorize: true,
        failed_login_attempts: 3, // 最多3次尝试
        session_idle_timeout_secs: 3600,
        default_username: "root".to_string(),
        default_password: "root".to_string(),
        force_change_default_password: false,
    };

    let auth = PasswordAuthenticator::new(
        |_username: &str, _password: &str| Ok(false),
        config,
    );

    // 第一次失败
    let result1 = auth.authenticate("user", "wrong");
    assert!(result1.is_err());
    assert!(result1.unwrap_err().to_string().contains("还剩 2 次尝试机会"));

    // 第二次失败
    let result2 = auth.authenticate("user", "wrong");
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("还剩 1 次尝试机会"));

    // 第三次失败
    let result3 = auth.authenticate("user", "wrong");
    assert!(result3.is_err());
    assert!(result3.unwrap_err().to_string().contains("已达到最大尝试次数"));
}

// ==================== ClientSession 角色管理测试 ====================

#[tokio::test]
async fn test_client_session_role_management() {
    let session = create_test_session("testuser");
    let client_session = ClientSession::new(session);

    // 初始没有角色
    assert!(!client_session.is_god());
    assert!(!client_session.is_admin());
    assert_eq!(client_session.role_with_space(1), None);

    // 设置角色
    client_session.set_role(1, RoleType::User);
    assert_eq!(client_session.role_with_space(1), Some(RoleType::User));

    // 设置God角色
    client_session.set_role(GOD_SPACE_ID, RoleType::God);
    assert!(client_session.is_god());
    assert!(client_session.is_admin());
}

#[tokio::test]
async fn test_client_session_multiple_spaces() {
    let session = create_test_session("testuser");
    let client_session = ClientSession::new(session);

    // 在不同Space设置不同角色
    client_session.set_role(1, RoleType::Admin);
    client_session.set_role(2, RoleType::User);
    client_session.set_role(3, RoleType::Guest);

    // 验证各Space的角色
    assert_eq!(client_session.role_with_space(1), Some(RoleType::Admin));
    assert_eq!(client_session.role_with_space(2), Some(RoleType::User));
    assert_eq!(client_session.role_with_space(3), Some(RoleType::Guest));

    // 获取所有角色
    let roles = client_session.roles();
    assert_eq!(roles.len(), 3);
}

#[tokio::test]
async fn test_client_session_space_info() {
    let session = create_test_session("testuser");
    let client_session = ClientSession::new(session);

    // 初始没有Space
    assert!(client_session.space().is_none());

    // 设置Space
    let space_info = SpaceInfo {
        name: "test_space".to_string(),
        id: 1,
    };
    client_session.set_space(space_info.clone());

    // 验证Space信息
    let space = client_session.space();
    assert!(space.is_some());
    assert_eq!(space.unwrap().name, "test_space");
    // space_name() 从 Session 结构体读取，set_space() 设置到 SpaceInfo 结构体
    // 两者是独立的存储，这里只验证 space() 返回正确的 SpaceInfo
}

// ==================== 综合场景测试 ====================

#[tokio::test]
async fn test_complete_permission_workflow() {
    // 创建权限管理器
    let pm = PermissionManager::new();
    let space_id = 1i64;

    // 1. God创建Space（模拟）
    assert!(pm.can_write_space("root").is_ok());

    // 2. God创建Admin用户并授予Admin角色
    pm.grant_role("admin1", space_id, RoleType::Admin).unwrap();

    // 3. Admin创建Dba用户并授予Dba角色
    assert!(pm.can_write_role("admin1", RoleType::Dba, space_id, "dba1").is_ok());
    pm.grant_role("dba1", space_id, RoleType::Dba).unwrap();

    // 4. Dba创建普通用户并授予User角色
    assert!(pm.can_write_role("dba1", RoleType::User, space_id, "user1").is_ok());
    pm.grant_role("user1", space_id, RoleType::User).unwrap();

    // 5. 验证各用户权限
    // Admin可以管理Schema
    assert!(pm.check_permission("admin1", space_id, Permission::Schema).is_ok());

    // Dba可以管理Schema
    assert!(pm.check_permission("dba1", space_id, Permission::Schema).is_ok());

    // 普通用户不能管理Schema
    assert!(pm.check_permission("user1", space_id, Permission::Schema).is_err());

    // 6. 列出Space中的所有用户
    let users = pm.list_space_users(space_id);
    assert_eq!(users.len(), 3);

    // 7. 撤销角色
    pm.revoke_role("user1", space_id).unwrap();
    assert_eq!(pm.get_role("user1", space_id), None);
}

#[tokio::test]
async fn test_cross_space_permission_isolation() {
    let pm = PermissionManager::new();

    // 用户在不同Space拥有不同角色
    pm.grant_role("user1", 1, RoleType::Admin).unwrap();
    pm.grant_role("user1", 2, RoleType::User).unwrap();

    // 在Space 1（Admin）拥有所有权限
    assert!(pm.check_permission("user1", 1, Permission::Schema).is_ok());
    assert!(pm.check_permission("user1", 1, Permission::Admin).is_ok());

    // 在Space 2（User）只有读写删权限
    assert!(pm.check_permission("user1", 2, Permission::Read).is_ok());
    assert!(pm.check_permission("user1", 2, Permission::Write).is_ok());
    assert!(pm.check_permission("user1", 2, Permission::Schema).is_err());

    // 在Space 3（无角色）没有任何权限
    assert!(pm.check_permission("user1", 3, Permission::Read).is_err());
}
