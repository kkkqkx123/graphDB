//! API模块集成测试
//!
//! 测试范围:
//! - api::session - 会话管理、客户端会话、查询管理
//! - api::service - 认证、权限、查询处理、统计管理
//! - api::mod - 服务启动、查询执行

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::api::session::{
    ClientSession, GraphSessionManager, QueryManager, QueryStatus,
    DEFAULT_SESSION_IDLE_TIMEOUT,
};
use graphdb::api::session::client_session::{Session, SpaceInfo, RoleType as SessionRoleType};
use graphdb::api::service::{
    Authenticator, PasswordAuthenticator, PermissionManager, Permission, RoleType,
    StatsManager, MetricType, GraphService,
};
use graphdb::api::service::stats_manager::QueryMetrics;
use graphdb::config::Config;
use graphdb::storage::redb_storage::DefaultStorage;
use graphdb::query::optimizer::rule_registry::RuleRegistry;

// ==================== 会话管理测试 ====================

#[tokio::test]
async fn test_session_manager_creation() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // 验证初始状态
    assert_eq!(session_manager.list_sessions().len(), 0);
}

#[tokio::test]
async fn test_create_and_find_session() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // 创建会话
    let session = session_manager
        .create_session("testuser".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");

    assert_eq!(session.user(), "testuser");

    // 查找会话
    let found_session = session_manager
        .find_session(session.id())
        .expect("未找到会话");
    assert_eq!(found_session.user(), "testuser");

    // 查找不存在的会话
    assert!(session_manager.find_session(999999).is_none());
}

#[tokio::test]
async fn test_remove_session() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    let session = session_manager
        .create_session("testuser".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");
    let session_id = session.id();

    // 验证会话存在
    assert!(session_manager.find_session(session_id).is_some());

    // 移除会话
    session_manager.remove_session(session_id);

    // 验证会话已移除
    assert!(session_manager.find_session(session_id).is_none());
}

#[tokio::test]
async fn test_max_connections_limit() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        3,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // 创建3个会话（达到上限）
    for i in 0..3 {
        let _ = session_manager
            .create_session(format!("user{}", i), "127.0.0.1".to_string())
            .expect("创建会话失败");
    }

    // 验证已达到最大连接数
    assert!(session_manager.is_out_of_connections());

    // 尝试创建第4个会话应该失败
    let result = session_manager.create_session("user4".to_string(), "127.0.0.1".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_sessions() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // 创建多个会话
    let session1 = session_manager
        .create_session("user1".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");
    let _session2 = session_manager
        .create_session("user2".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");

    // 获取会话列表
    let sessions = session_manager.list_sessions();
    assert_eq!(sessions.len(), 2);

    // 验证会话信息
    let session_info = session_manager
        .get_session_info(session1.id())
        .expect("获取会话信息失败");
    assert_eq!(session_info.user_name, "user1");
}

#[tokio::test]
async fn test_kill_session() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // 创建会话并设置为Admin角色
    let session = session_manager
        .create_session("admin".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");
    session.set_role(0, SessionRoleType::ADMIN);
    let _session_id = session.id();

    // 创建另一个用户会话
    let target_session = session_manager
        .create_session("user1".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");
    let target_id = target_session.id();

    // Admin可以终止其他用户的会话
    let result = session_manager.kill_session(target_id, "admin", true);
    assert!(result.is_ok());
    assert!(session_manager.find_session(target_id).is_none());

    // 非Admin用户不能终止其他用户的会话
    let other_session = session_manager
        .create_session("user2".to_string(), "127.0.0.1".to_string())
        .expect("创建会话失败");
    let other_id = other_session.id();

    let result = session_manager.kill_session(other_id, "user1", false);
    assert!(result.is_err());
}

// ==================== 客户端会话测试 ====================

#[test]
fn test_client_session_properties() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    assert_eq!(client_session.id(), 123);
    assert_eq!(client_session.user(), "testuser");
    assert!(client_session.space().is_none());
}

#[test]
fn test_client_session_space_management() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // 设置Space
    let space = SpaceInfo {
        name: "test_space".to_string(),
        id: 1,
    };
    client_session.set_space(space.clone());

    assert_eq!(client_session.space().expect("Failed to get space info").name, "test_space");
    assert_eq!(client_session.space().expect("Failed to get space info").id, 1);
}

#[test]
fn test_client_session_roles() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // 初始没有角色
    assert!(!client_session.is_admin());

    // 设置Admin角色
    client_session.set_role(1, SessionRoleType::ADMIN);
    assert!(client_session.is_admin());
    assert!(matches!(
        client_session.role_with_space(1),
        Some(SessionRoleType::ADMIN)
    ));

    // 设置User角色
    client_session.set_role(2, SessionRoleType::USER);
    assert!(matches!(
        client_session.role_with_space(2),
        Some(SessionRoleType::USER)
    ));
}

#[test]
fn test_client_session_queries() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // 添加查询
    client_session.add_query(1, "SELECT * FROM users".to_string());
    client_session.add_query(2, "INSERT INTO users VALUES (...)".to_string());

    assert!(client_session.find_query(1));
    assert!(client_session.find_query(2));
    assert!(!client_session.find_query(3));
    assert_eq!(client_session.active_queries_count(), 2);

    // 删除查询
    client_session.delete_query(1);
    assert!(!client_session.find_query(1));
    assert_eq!(client_session.active_queries_count(), 1);

    // 终止所有查询
    client_session.mark_all_queries_killed();
    assert_eq!(client_session.active_queries_count(), 0);
}

#[test]
fn test_client_session_idle_time() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // 等待一小段时间
    std::thread::sleep(Duration::from_millis(10));
    let idle_time = client_session.idle_seconds();
    assert!(idle_time >= 0);

    // 重置空闲时间
    client_session.charge();
    assert!(client_session.idle_seconds() <= idle_time);
}

// ==================== 查询管理器测试 ====================

#[test]
fn test_query_manager_creation() {
    let query_manager = QueryManager::new();

    // 初始状态应该为空
    let queries = query_manager.get_all_queries().expect("获取查询失败");
    assert!(queries.is_empty());
}

#[test]
fn test_register_and_get_query() {
    let query_manager = QueryManager::new();

    // 注册查询
    let query_id = query_manager
        .register_query(1, "testuser".to_string(), Some("test_space".to_string()), "SELECT * FROM users".to_string())
        .expect("注册查询失败");

    // 获取查询信息
    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.session_id, 1);
    assert_eq!(query_info.user_name, "testuser");
    assert_eq!(query_info.query_text, "SELECT * FROM users");
    assert_eq!(query_info.status, QueryStatus::Running);

    // 获取不存在的查询
    assert!(query_manager.get_query(9999).is_err());
}

#[test]
fn test_query_status_transitions() {
    let query_manager = QueryManager::new();

    let query_id = query_manager
        .register_query(1, "testuser".to_string(), None, "SELECT * FROM users".to_string())
        .expect("注册查询失败");

    // 标记为完成
    query_manager.mark_query_finished(query_id).expect("标记完成失败");
    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.status, QueryStatus::Finished);
    assert!(query_info.duration_ms.is_some());

    // 注册新查询并标记为失败
    let query_id2 = query_manager
        .register_query(1, "testuser".to_string(), None, "INVALID QUERY".to_string())
        .expect("注册查询失败");

    query_manager.mark_query_failed(query_id2).expect("标记失败失败");
    let query_info2 = query_manager.get_query(query_id2).expect("获取查询失败");
    assert_eq!(query_info2.status, QueryStatus::Failed);
}

#[test]
fn test_kill_query() {
    let query_manager = QueryManager::new();

    let query_id = query_manager
        .register_query(1, "testuser".to_string(), None, "SELECT * FROM users".to_string())
        .expect("注册查询失败");

    // 终止查询
    query_manager.kill_query(query_id).expect("终止查询失败");

    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.status, QueryStatus::Killed);
}

#[test]
fn test_get_queries_by_session() {
    let query_manager = QueryManager::new();

    // 为不同会话注册查询
    query_manager
        .register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string())
        .expect("注册查询失败");
    query_manager
        .register_query(1, "user1".to_string(), None, "SELECT * FROM t2".to_string())
        .expect("注册查询失败");
    query_manager
        .register_query(2, "user2".to_string(), None, "SELECT * FROM t3".to_string())
        .expect("注册查询失败");

    // 获取会话1的查询
    let session1_queries = query_manager.get_session_queries(1).expect("获取查询失败");
    assert_eq!(session1_queries.len(), 2);

    // 获取会话2的查询
    let session2_queries = query_manager.get_session_queries(2).expect("获取查询失败");
    assert_eq!(session2_queries.len(), 1);

    // 获取不存在的会话查询
    let session3_queries = query_manager.get_session_queries(3).expect("获取查询失败");
    assert!(session3_queries.is_empty());
}

#[test]
fn test_get_queries_by_user() {
    let query_manager = QueryManager::new();

    query_manager
        .register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string())
        .expect("注册查询失败");
    query_manager
        .register_query(2, "user1".to_string(), None, "SELECT * FROM t2".to_string())
        .expect("注册查询失败");
    query_manager
        .register_query(3, "user2".to_string(), None, "SELECT * FROM t3".to_string())
        .expect("注册查询失败");

    let user1_queries = query_manager.get_user_queries("user1").expect("获取查询失败");
    assert_eq!(user1_queries.len(), 2);

    let user2_queries = query_manager.get_user_queries("user2").expect("获取查询失败");
    assert_eq!(user2_queries.len(), 1);
}

#[test]
fn test_get_running_queries() {
    let query_manager = QueryManager::new();

    let query_id1 = query_manager
        .register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string())
        .expect("注册查询失败");
    let query_id2 = query_manager
        .register_query(1, "user1".to_string(), None, "SELECT * FROM t2".to_string())
        .expect("注册查询失败");

    // 两个查询都在运行
    let running = query_manager.get_running_queries().expect("获取查询失败");
    assert_eq!(running.len(), 2);

    // 完成一个查询
    query_manager.mark_query_finished(query_id1).expect("标记完成失败");

    let running = query_manager.get_running_queries().expect("获取查询失败");
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].query_id, query_id2);
}

// ==================== 认证器测试 ====================

#[test]
fn test_password_authenticator_creation() {
    let authenticator = PasswordAuthenticator::new();

    // 验证默认用户
    assert!(authenticator.verify_password("root", "root"));
    assert!(authenticator.verify_password("nebula", "nebula"));
    assert!(!authenticator.verify_password("root", "wrong"));
}

#[test]
fn test_authenticate_success() {
    let authenticator = PasswordAuthenticator::new();

    let result = authenticator.authenticate("root", "root");
    assert!(result.is_ok());
}

#[test]
fn test_authenticate_failure() {
    let authenticator = PasswordAuthenticator::new();

    // 错误密码
    let result = authenticator.authenticate("root", "wrong_password");
    assert!(result.is_err());

    // 不存在的用户
    let result = authenticator.authenticate("nonexistent", "password");
    assert!(result.is_err());

    // 空用户名
    let result = authenticator.authenticate("", "password");
    assert!(result.is_err());

    // 空密码
    let result = authenticator.authenticate("root", "");
    assert!(result.is_err());
}

#[test]
fn test_add_and_remove_user() {
    let authenticator = PasswordAuthenticator::new();

    // 添加用户
    authenticator
        .add_user("testuser".to_string(), "testpass".to_string())
        .expect("添加用户失败");
    assert!(authenticator.verify_password("testuser", "testpass"));

    // 验证新用户可以认证
    let result = authenticator.authenticate("testuser", "testpass");
    assert!(result.is_ok());

    // 删除用户
    authenticator.remove_user("testuser").expect("删除用户失败");
    assert!(!authenticator.verify_password("testuser", "testpass"));
}

// ==================== 权限管理器测试 ====================

#[test]
fn test_permission_manager_creation() {
    let permission_manager = PermissionManager::new();

    // root用户默认是Admin
    assert!(permission_manager.is_admin("root"));
    assert!(!permission_manager.is_admin("nonexistent"));
}

#[test]
fn test_role_type_permissions() {
    // Admin拥有所有权限
    assert!(RoleType::Admin.has_permission(Permission::Read));
    assert!(RoleType::Admin.has_permission(Permission::Write));
    assert!(RoleType::Admin.has_permission(Permission::Delete));
    assert!(RoleType::Admin.has_permission(Permission::Schema));
    assert!(RoleType::Admin.has_permission(Permission::Admin));

    // User只有读写删权限
    assert!(RoleType::User.has_permission(Permission::Read));
    assert!(RoleType::User.has_permission(Permission::Write));
    assert!(RoleType::User.has_permission(Permission::Delete));
    assert!(!RoleType::User.has_permission(Permission::Schema));
    assert!(!RoleType::User.has_permission(Permission::Admin));
}

#[test]
fn test_grant_and_revoke_role() {
    let permission_manager = PermissionManager::new();

    // 授予User角色
    permission_manager
        .grant_role("testuser", 1, RoleType::User)
        .expect("授权失败");

    // 验证角色
    let role = permission_manager.get_role("testuser", 1);
    assert!(matches!(role, Some(RoleType::User)));

    // 验证权限检查
    permission_manager
        .check_permission("testuser", 1, Permission::Read)
        .expect("权限检查失败");

    // User没有Schema权限
    assert!(permission_manager
        .check_permission("testuser", 1, Permission::Schema)
        .is_err());

    // 撤销角色
    permission_manager.revoke_role("testuser", 1).expect("撤销角色失败");
    assert!(permission_manager.get_role("testuser", 1).is_none());
}

#[test]
fn test_admin_permissions() {
    let permission_manager = PermissionManager::new();

    // root默认是Admin
    permission_manager
        .check_permission("root", 0, Permission::Admin)
        .expect("Admin权限检查失败");
    permission_manager
        .check_permission("root", 0, Permission::Schema)
        .expect("Admin权限检查失败");
    permission_manager
        .check_permission("root", 0, Permission::Read)
        .expect("Admin权限检查失败");
}

#[test]
fn test_custom_permissions() {
    let permission_manager = PermissionManager::new();

    // 授予自定义权限
    permission_manager
        .grant_permission(1, "testuser", Permission::Read)
        .expect("授予权限失败");
    permission_manager
        .grant_permission(1, "testuser", Permission::Write)
        .expect("授予权限失败");

    // 验证自定义权限
    permission_manager
        .check_custom_permission(1, "testuser", Permission::Read)
        .expect("权限检查失败");
    permission_manager
        .check_custom_permission(1, "testuser", Permission::Write)
        .expect("权限检查失败");

    // 未授予的权限
    assert!(permission_manager
        .check_custom_permission(1, "testuser", Permission::Delete)
        .is_err());

    // 撤销权限
    permission_manager
        .revoke_permission(1, "testuser", Permission::Read)
        .expect("撤销权限失败");
    assert!(permission_manager
        .check_custom_permission(1, "testuser", Permission::Read)
        .is_err());
}

// ==================== 统计管理器测试 ====================

#[test]
fn test_stats_manager_creation() {
    let _stats_manager = StatsManager::new();

    // 初始状态
    // 统计管理器刚创建时没有指标值
}

#[test]
fn test_metric_operations() {
    let stats_manager = StatsManager::new();

    // 增加指标值
    stats_manager.add_value(MetricType::NumOpenedSessions);
    stats_manager.add_value(MetricType::NumOpenedSessions);

    // 减少指标值
    stats_manager.dec_value(MetricType::NumActiveSessions);

    // 批量增加
    stats_manager.add_value_with_amount(MetricType::NumQueries, 5);
}

#[test]
fn test_space_metrics() {
    let stats_manager = StatsManager::new();

    // 添加Space指标
    stats_manager.add_space_metric("test_space", MetricType::NumActiveQueries);
    stats_manager.add_space_metric("test_space", MetricType::NumActiveQueries);

    // 减少Space指标
    stats_manager.dec_space_metric("test_space", MetricType::NumActiveQueries);
}

#[test]
fn test_query_metrics() {
    let stats_manager = StatsManager::new();

    use std::time::Duration;

    // 记录查询指标
    let mut metrics = QueryMetrics::new();
    metrics.record_parse_time(Duration::from_micros(100));
    metrics.record_execute_time(Duration::from_micros(500));
    stats_manager.record_query_metrics(&metrics);
}

// ==================== GraphService集成测试 ====================

fn create_test_config() -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 9669,
        storage_path: "/tmp/graphdb_test".to_string(),
        max_connections: 10,
        transaction_timeout: 30,
        log_level: "info".to_string(),
        log_dir: "logs".to_string(),
        log_file: "logs/test.log".to_string(),
        max_log_file_size: 100 * 1024 * 1024,
        max_log_files: 5,
    }
}

#[tokio::test]
async fn test_graph_service_creation() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));

    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 验证服务创建成功
    assert!(!graph_service.get_session_manager().is_out_of_connections());
}

#[tokio::test]
async fn test_graph_service_authentication() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 成功认证
    let session = graph_service.authenticate("root", "root").await;
    assert!(session.is_ok());

    // 失败认证
    let session = graph_service.authenticate("root", "wrong").await;
    assert!(session.is_err());
}

#[tokio::test]
async fn test_graph_service_signout() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 认证并获取会话
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // 验证会话存在
    assert!(graph_service.get_session_manager().find_session(session_id).is_some());

    // 登出
    graph_service.signout(session_id);

    // 验证会话已移除
    assert!(graph_service.get_session_manager().find_session(session_id).is_none());
}

#[tokio::test]
async fn test_graph_service_execute_query() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 认证并获取会话
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // 执行查询
    let _result = graph_service.execute(session_id, "SHOW SPACES").await;
    // 查询可能成功或失败，但不应该panic
}

#[tokio::test]
async fn test_graph_service_invalid_session() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 使用无效的会话ID执行查询
    let result = graph_service.execute(999999, "SHOW SPACES").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graph_service_list_sessions() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 初始没有会话
    let sessions = graph_service.list_sessions();
    assert_eq!(sessions.len(), 0);

    // 创建会话
    let _session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");

    // 验证会话列表
    let sessions = graph_service.list_sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].user_name, "root");
}

#[tokio::test]
async fn test_graph_service_kill_session() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 创建两个会话
    let admin_session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let admin_session_id = admin_session.id();

    // root默认是Admin，可以终止会话
    let result = graph_service.kill_session(admin_session_id, "root");
    assert!(result.is_ok());

    // 验证会话已终止
    assert!(graph_service.get_session_manager().find_session(admin_session_id).is_none());
}

// ==================== API模块功能集成测试 ====================

#[tokio::test]
async fn test_full_session_lifecycle() {
    let _ = RuleRegistry::initialize();
    let config = create_test_config();
    let storage = Arc::new(DefaultStorage::new().expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    // 1. 认证创建会话
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // 2. 验证会话存在
    assert!(graph_service.get_session_manager().find_session(session_id).is_some());

    // 3. 执行查询
    let _ = graph_service.execute(session_id, "SHOW SPACES").await;

    // 4. 获取会话信息
    let session_info = graph_service.get_session_info(session_id);
    assert!(session_info.is_some());
    assert_eq!(session_info.expect("Failed to get session info").user_name, "root");

    // 5. 登出
    graph_service.signout(session_id);

    // 6. 验证会话已移除
    assert!(graph_service.get_session_manager().find_session(session_id).is_none());
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    let mut handles = vec![];

    // 并发创建会话
    for i in 0..10 {
        let manager = Arc::clone(&session_manager);
        let handle = tokio::spawn(async move {
            let session = manager
                .create_session(format!("user{}", i), "127.0.0.1".to_string())
                .expect("创建会话失败");
            session.set_role(1, SessionRoleType::USER);
            session.id()
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let mut session_ids = vec![];
    for handle in handles {
        session_ids.push(handle.await.expect("任务失败"));
    }

    // 验证所有会话都创建成功
    assert_eq!(session_ids.len(), 10);

    // 验证可以查找到所有会话
    for session_id in &session_ids {
        assert!(session_manager.find_session(*session_id).is_some());
    }

    // 验证会话列表
    let sessions = session_manager.list_sessions();
    assert_eq!(sessions.len(), 10);
}

#[test]
fn test_query_manager_concurrent_operations() {
    use std::thread;

    let query_manager = Arc::new(QueryManager::new());
    let mut handles = vec![];

    // 并发注册查询
    for i in 0..10 {
        let manager = Arc::clone(&query_manager);
        let handle = thread::spawn(move || {
            manager
                .register_query(
                    1,
                    "testuser".to_string(),
                    None,
                    format!("SELECT * FROM table{}", i),
                )
                .expect("注册查询失败")
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    let query_ids: Vec<i64> = handles
        .into_iter()
        .map(|h| h.join().expect("线程panic"))
        .collect();

    // 验证所有查询都注册成功
    assert_eq!(query_ids.len(), 10);

    // 验证可以查找到所有查询
    let all_queries = query_manager.get_all_queries().expect("获取查询失败");
    assert_eq!(all_queries.len(), 10);

    // 验证运行中的查询
    let running_queries = query_manager.get_running_queries().expect("获取查询失败");
    assert_eq!(running_queries.len(), 10);
}
