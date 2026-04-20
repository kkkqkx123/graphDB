//! API Module Integration Testing
//!
//! Test scope:
//! - api::session - session management, client session, query management
//! - `api::service` – Authentication, authorization, query processing, and statistical management.
//! - api::mod - service startup, query execution

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::api::server::auth::{Authenticator, PasswordAuthenticator};
use graphdb::api::server::graph_service::GraphService;
use graphdb::api::server::permission::{Permission, PermissionManager};
use graphdb::api::server::session::{
    ClientSession, GraphSessionManager, Session, SpaceInfo, DEFAULT_SESSION_IDLE_TIMEOUT,
};
use graphdb::config::Config;
use graphdb::core::{MetricType, QueryMetrics, RoleType, StatsManager};
use graphdb::query::{QueryManager, QueryStatus};
use graphdb::search::FulltextConfig;
use graphdb::storage::DefaultStorage;

// ==================== Session Management Test ====================

#[tokio::test]
async fn test_session_manager_creation() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // Validating the initial state
    assert_eq!(session_manager.list_sessions().await.len(), 0);
}

#[tokio::test]
async fn test_create_and_find_session() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // Create a session
    let session = session_manager
        .create_session("testuser".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");

    assert_eq!(session.user(), "testuser");

    // Find Sessions
    let found_session = session_manager
        .find_session(session.id())
        .expect("未找到会话");
    assert_eq!(found_session.user(), "testuser");

    // Trying to find a session that does not exist.
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
        .await
        .expect("创建会话失败");
    let session_id = session.id();

    // Verify Session Existence
    assert!(session_manager.find_session(session_id).is_some());

    // Remove the session.
    session_manager.remove_session(session_id).await;

    // Verify that the session has been removed
    assert!(session_manager.find_session(session_id).is_none());
}

#[tokio::test]
async fn test_max_connections_limit() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        3,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // Create 3 sessions (to reach the maximum limit).
    for i in 0..3 {
        let _ = session_manager
            .create_session(format!("user{}", i), "127.0.0.1".to_string())
            .await
            .expect("创建会话失败");
    }

    // Verify that the maximum number of connections has been reached
    assert!(session_manager.is_out_of_connections().await);

    // Trying to create the fourth session should fail.
    let result = session_manager.create_session("user4".to_string(), "127.0.0.1".to_string());
    assert!(result.await.is_err());
}

#[tokio::test]
async fn test_list_sessions() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    // Creating Multiple Sessions
    let session1 = session_manager
        .create_session("user1".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");
    let _session2 = session_manager
        .create_session("user2".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");

    // Obtain the session list
    let sessions = session_manager.list_sessions();
    let sessions = sessions.await;
    assert_eq!(sessions.len(), 2);

    // Verify session information
    let session_info = session_manager
        .get_session_info(session1.id())
        .await
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

    // Create a session and set it to the Admin role.
    let session = session_manager
        .create_session("admin".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");
    session.set_role(0, RoleType::Admin);
    let _session_id = session.id();

    // Create another user session
    let target_session = session_manager
        .create_session("user1".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");
    let target_id = target_session.id();

    // The admin can terminate the sessions of other users.
    let result = session_manager.kill_session(target_id, "admin", true);
    assert!(result.await.is_ok());
    assert!(session_manager.find_session(target_id).is_none());

    // Non-Admin users cannot terminate another user's session
    let other_session = session_manager
        .create_session("user2".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");
    let other_id = other_session.id();

    let result = session_manager.kill_session(other_id, "user1", false);
    assert!(result.await.is_err());
}

// ==================== Client Session Testing ====================

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

#[tokio::test]
async fn test_client_session_space_management() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // Setting Space
    let space = SpaceInfo {
        name: "test_space".to_string(),
        id: 1,
    };
    client_session.set_space(space.clone());

    assert_eq!(
        client_session
            .space()
            .expect("Failed to get space info")
            .name,
        "test_space"
    );
    assert_eq!(
        client_session.space().expect("Failed to get space info").id,
        1
    );
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

    // There were no characters initially.
    assert!(!client_session.is_admin());

    // Setting up the Admin role
    client_session.set_role(1, RoleType::Admin);
    assert!(client_session.is_admin());
    assert!(matches!(
        client_session.role_with_space(1),
        Some(RoleType::Admin)
    ));

    // Setting up the User role
    client_session.set_role(2, RoleType::User);
    assert!(matches!(
        client_session.role_with_space(2),
        Some(RoleType::User)
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

    // Add Query
    client_session.add_query(1, "SELECT * FROM users".to_string());
    client_session.add_query(2, "INSERT INTO users VALUES (...)".to_string());

    assert!(client_session.find_query(1));
    assert!(client_session.find_query(2));
    assert!(!client_session.find_query(3));
    assert_eq!(client_session.active_queries_count(), 2);

    // Delete the query.
    client_session.delete_query(1);
    assert!(!client_session.find_query(1));
    assert_eq!(client_session.active_queries_count(), 1);

    // Terminate all queries
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

    // Wait for a short while.
    std::thread::sleep(Duration::from_millis(10));
    let idle_time = client_session.idle_seconds();

    // Reset Idle Time
    client_session.charge();
    assert!(client_session.idle_seconds() <= idle_time);
}

// ==================== Query Manager Test ====================

#[test]
fn test_query_manager_creation() {
    let query_manager = QueryManager::new();

    // The initial state should be empty
    let queries = query_manager.get_all_queries();
    assert!(queries.is_empty());
}

#[tokio::test]
async fn test_register_and_get_query() {
    let query_manager = QueryManager::new();

    // Registration Inquiry
    let query_id = query_manager.register_query(
        1,
        "testuser".to_string(),
        Some("test_space".to_string()),
        "SELECT * FROM users".to_string(),
    );

    // Getting Query Information
    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.session_id, 1);
    assert_eq!(query_info.user_name, "testuser");
    assert_eq!(query_info.query_text, "SELECT * FROM users");
    assert_eq!(query_info.status, QueryStatus::Running);

    // Trying to retrieve a query that does not exist.
    assert!(query_manager.get_query(9999).is_none());
}

#[tokio::test]
async fn test_query_status_transitions() {
    let query_manager = QueryManager::new();

    let query_id = query_manager.register_query(
        1,
        "testuser".to_string(),
        None,
        "SELECT * FROM users".to_string(),
    );

    // Marked as completed
    query_manager.finish_query(query_id).expect("标记完成失败");
    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.status, QueryStatus::Finished);
    assert!(query_info.duration_ms.is_some());

    // Register a new query and mark it as a failure.
    let query_id2 =
        query_manager.register_query(1, "testuser".to_string(), None, "INVALID QUERY".to_string());

    query_manager.fail_query(query_id2).expect("标记失败失败");
    let query_info2 = query_manager.get_query(query_id2).expect("获取查询失败");
    assert_eq!(query_info2.status, QueryStatus::Failed);
}

#[tokio::test]
async fn test_kill_query() {
    let query_manager = QueryManager::new();

    let query_id = query_manager.register_query(
        1,
        "testuser".to_string(),
        None,
        "SELECT * FROM users".to_string(),
    );

    // Termination of inquiries
    query_manager.kill_query(query_id).expect("终止查询失败");

    let query_info = query_manager.get_query(query_id).expect("获取查询失败");
    assert_eq!(query_info.status, QueryStatus::Killed);
}

#[test]
fn test_get_queries_by_session() {
    let query_manager = QueryManager::new();

    // Register queries for different sessions
    query_manager.register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string());
    query_manager.register_query(1, "user1".to_string(), None, "SELECT * FROM t2".to_string());
    query_manager.register_query(2, "user2".to_string(), None, "SELECT * FROM t3".to_string());

    // Get all queries and filter by session
    let all_queries = query_manager.get_all_queries();
    let session1_queries: Vec<_> = all_queries.iter().filter(|q| q.session_id == 1).collect();
    assert_eq!(session1_queries.len(), 2);

    let session2_queries: Vec<_> = all_queries.iter().filter(|q| q.session_id == 2).collect();
    assert_eq!(session2_queries.len(), 1);

    // Attempting to retrieve a session that does not exist.
    let session3_queries: Vec<_> = all_queries.iter().filter(|q| q.session_id == 3).collect();
    assert!(session3_queries.is_empty());
}

#[test]
fn test_get_queries_by_user() {
    let query_manager = QueryManager::new();

    query_manager.register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string());
    query_manager.register_query(2, "user1".to_string(), None, "SELECT * FROM t2".to_string());
    query_manager.register_query(3, "user2".to_string(), None, "SELECT * FROM t3".to_string());

    let all_queries = query_manager.get_all_queries();
    let user1_queries: Vec<_> = all_queries
        .iter()
        .filter(|q| q.user_name == "user1")
        .collect();
    assert_eq!(user1_queries.len(), 2);

    let user2_queries: Vec<_> = all_queries
        .iter()
        .filter(|q| q.user_name == "user2")
        .collect();
    assert_eq!(user2_queries.len(), 1);
}

#[tokio::test]
async fn test_get_running_queries() {
    let query_manager = QueryManager::new();

    let query_id1 =
        query_manager.register_query(1, "user1".to_string(), None, "SELECT * FROM t1".to_string());
    let query_id2 =
        query_manager.register_query(1, "user1".to_string(), None, "SELECT * FROM t2".to_string());

    // Complete the first query
    query_manager.finish_query(query_id1).expect("标记完成失败");

    // Obtain the queries that are currently running.
    let running_queries = query_manager.get_running_queries();
    assert_eq!(running_queries.len(), 1);
    assert_eq!(running_queries[0].query_id, query_id2);
}

// ==================== Certifier Testing ====================

#[test]
fn test_password_authenticator_creation() {
    let config = graphdb::config::AuthConfig::default();
    let authenticator = PasswordAuthenticator::new_default(config);

    // Verify that the default user can be authenticated.
    let result = authenticator.authenticate("root", "root");
    assert!(result.is_ok());
}

#[test]
fn test_authenticate_success() {
    let config = graphdb::config::AuthConfig::default();
    let authenticator = PasswordAuthenticator::new_default(config);

    let result = authenticator.authenticate("root", "root");
    assert!(result.is_ok());
}

#[test]
fn test_authenticate_failure() {
    let config = graphdb::config::AuthConfig::default();
    let authenticator = PasswordAuthenticator::new_default(config);

    // incorrect password
    let result = authenticator.authenticate("root", "wrong_password");
    assert!(result.is_err());

    // Non-existent user
    let result = authenticator.authenticate("nonexistent", "password");
    assert!(result.is_err());

    // empty username
    let result = authenticator.authenticate("", "password");
    assert!(result.is_err());

    // empty password
    let result = authenticator.authenticate("root", "");
    assert!(result.is_err());
}

#[test]
fn test_custom_user_verifier() {
    let config = graphdb::config::AuthConfig {
        enable_authorize: true,
        failed_login_attempts: 0,
        session_idle_timeout_secs: 3600,
        default_username: "admin".to_string(),
        default_password: "admin123".to_string(),
        force_change_default_password: false,
    };

    let authenticator = PasswordAuthenticator::new(
        |username: &str, password: &str| Ok(username == "testuser" && password == "testpass"),
        config,
    );

    // Customized User Authentication
    let result = authenticator.authenticate("testuser", "testpass");
    assert!(result.is_ok());

    // incorrect password
    let result = authenticator.authenticate("testuser", "wrong");
    assert!(result.is_err());
}

// ==================== Permission Manager Test ====================

#[test]
fn test_permission_manager_creation() {
    let permission_manager = PermissionManager::new();

    // The root user is Admin by default
    assert!(permission_manager.is_admin("root"));
    assert!(!permission_manager.is_admin("nonexistent"));
}

#[test]
fn test_role_type_permissions() {
    // Admin has all privileges
    assert!(RoleType::Admin.has_permission(Permission::Read));
    assert!(RoleType::Admin.has_permission(Permission::Write));
    assert!(RoleType::Admin.has_permission(Permission::Delete));
    assert!(RoleType::Admin.has_permission(Permission::Schema));
    assert!(RoleType::Admin.has_permission(Permission::Admin));

    // User only has read/write/delete privileges
    assert!(RoleType::User.has_permission(Permission::Read));
    assert!(RoleType::User.has_permission(Permission::Write));
    assert!(RoleType::User.has_permission(Permission::Delete));
    assert!(!RoleType::User.has_permission(Permission::Schema));
    assert!(!RoleType::User.has_permission(Permission::Admin));
}

#[test]
fn test_grant_and_revoke_role() {
    let permission_manager = PermissionManager::new();

    // Granting the User role
    permission_manager
        .grant_role("testuser", 1, RoleType::User)
        .expect("授权失败");

    // Verify Roles
    let role = permission_manager.get_role("testuser", 1);
    assert!(matches!(role, Some(RoleType::User)));

    // Verify Permission Checking
    permission_manager
        .check_permission("testuser", 1, Permission::Read)
        .expect("权限检查失败");

    // User has no Schema privileges
    assert!(permission_manager
        .check_permission("testuser", 1, Permission::Schema)
        .is_err());

    // Withdrawal of roles
    permission_manager
        .revoke_role("testuser", 1)
        .expect("撤销角色失败");
    assert!(permission_manager.get_role("testuser", 1).is_none());
}

#[test]
fn test_admin_permissions() {
    let permission_manager = PermissionManager::new();

    // root is Admin by default
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

    // Grant the user role first (using the Guest role, with Read access only)
    permission_manager
        .grant_role("testuser", 1, RoleType::Guest)
        .expect("授予角色失败");

    // Granting additional custom permissions (explicitly granting Read and Write)
    permission_manager
        .grant_permission("testuser", 1, Permission::Read)
        .expect("授予权限失败");
    permission_manager
        .grant_permission("testuser", 1, Permission::Write)
        .expect("授予权限失败");

    // Validate custom permissions (use has_permission to check fine-grained permissions)
    assert!(permission_manager.has_permission("testuser", 1, Permission::Read));
    assert!(permission_manager.has_permission("testuser", 1, Permission::Write));

    // Ungranted permissions (the Guest role does not have Delete permissions and is not explicitly granted)
    assert!(!permission_manager.has_permission("testuser", 1, Permission::Delete));

    // Revocation of authority
    permission_manager
        .revoke_permission("testuser", 1, Permission::Write)
        .expect("撤销权限失败");
    assert!(!permission_manager.has_permission("testuser", 1, Permission::Write));
}

// ==================== Statistical Manager Test ====================

#[test]
fn test_stats_manager_creation() {
    let _stats_manager = StatsManager::new();

    // initial state
    // Statistics Manager has no indicator values when it is first created
}

#[test]
fn test_metric_operations() {
    let stats_manager = StatsManager::new();

    // Increased value of indicators
    stats_manager.add_value(MetricType::NumQueries);
    stats_manager.add_value(MetricType::NumQueries);

    // Reduction in the value of indicators
    stats_manager.dec_value(MetricType::NumActiveQueries);

    // Batch increase
    stats_manager.add_value_with_amount(MetricType::NumQueries, 5);
}

#[test]
fn test_space_metrics() {
    let stats_manager = StatsManager::new();

    // Add Space Indicator
    stats_manager.add_space_metric("test_space", MetricType::NumActiveQueries);
    stats_manager.add_space_metric("test_space", MetricType::NumActiveQueries);

    // Reduction of Space indicators
    stats_manager.dec_space_metric("test_space", MetricType::NumActiveQueries);
}

#[test]
fn test_query_metrics() {
    let stats_manager = StatsManager::new();

    use std::time::Duration;

    // Records search indicators
    let mut metrics = QueryMetrics::new();
    metrics.record_parse_time(Duration::from_micros(100));
    metrics.record_execute_time(Duration::from_micros(500));
    stats_manager.record_query_metrics(&metrics);
}

// ==================== GraphService集成测试 ====================

fn create_test_config() -> Config {
    Config {
        common: graphdb::config::CommonConfig {
            database: graphdb::config::DatabaseConfig {
                host: "127.0.0.1".to_string(),
                port: 9669,
                storage_path: "/tmp/graphdb_test".to_string(),
                max_connections: 10,
            },
            transaction: graphdb::config::TransactionConfig {
                default_timeout: 30,
                max_concurrent_transactions: 1000,
            },
            log: graphdb::config::LogConfig {
                level: "info".to_string(),
                dir: "logs".to_string(),
                file: "test".to_string(),
                max_file_size: 100 * 1024 * 1024,
                max_files: 5,
            },
            storage: graphdb::config::StorageConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
            monitoring: graphdb::config::MonitoringConfig::default(),
            query_resource: graphdb::config::QueryResourceConfig::default(),
        },
        #[cfg(feature = "server")]
        server: graphdb::config::ServerConfig {
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            ..Default::default()
        },
        #[cfg(feature = "embedded")]
        embedded: graphdb::config::EmbeddedConfig::default(),
        fulltext: FulltextConfig::default(),
        vector: vector_client::config::VectorClientConfig::default(),
    }
}

#[tokio::test]
async fn test_graph_service_creation() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));

    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Verify that the service was created successfully
    assert!(
        !graph_service
            .get_session_manager()
            .is_out_of_connections()
            .await
    );
}

#[tokio::test]
async fn test_graph_service_authentication() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Successful certification
    let session = graph_service.authenticate("root", "root");
    assert!(session.await.is_ok());

    // Failure to Certify
    let session = graph_service.authenticate("root", "wrong");
    assert!(session.await.is_err());
}

#[tokio::test]
async fn test_graph_service_signout() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication and session acquisition
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Verify Session Existence
    assert!(graph_service
        .get_session_manager()
        .find_session(session_id)
        .is_some());

    // appear (in a newspaper etc)
    graph_service.signout(session_id).await;

    // Verify that the session has been removed
    assert!(graph_service
        .get_session_manager()
        .find_session(session_id)
        .is_none());
}

#[tokio::test]
async fn test_graph_service_execute_query() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication and session acquisition
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // perform a search
    let _result = graph_service.execute(session_id, "SHOW SPACES");
    // The query may succeed or fail, but should not be panic
}

#[tokio::test]
async fn test_graph_service_invalid_session() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Execute a query with an invalid session ID
    let result = graph_service.execute(999999, "SHOW SPACES");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graph_service_list_sessions() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Initially no session
    let sessions = graph_service.list_sessions();
    assert_eq!(sessions.await.len(), 0);

    // Creating a session
    let _session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");

    // Validating session lists
    let sessions = graph_service.list_sessions();
    let sessions = sessions.await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].user_name, "root");
}

#[tokio::test]
async fn test_graph_service_kill_session() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Create two sessions
    let admin_session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let admin_session_id = admin_session.id();

    // root is Admin by default and can terminate the session
    let result = graph_service.kill_session(admin_session_id, "root");
    assert!(result.await.is_ok());

    // Verify that the session has been terminated
    assert!(graph_service
        .get_session_manager()
        .find_session(admin_session_id)
        .is_none());
}

// ==================== API模块功能集成测试 ====================

#[tokio::test]
async fn test_full_session_lifecycle() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // 1. Authentication creation session
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // 2. Verify session existence
    assert!(graph_service
        .get_session_manager()
        .find_session(session_id)
        .is_some());

    // 3. Execution of queries
    let _ = graph_service.execute(session_id, "SHOW SPACES");

    // 4. Obtaining session information
    let session_info = graph_service.get_session_info(session_id);
    let session_info = session_info.await;
    assert!(session_info.is_some());
    assert_eq!(
        session_info.expect("Failed to get session info").user_name,
        "root"
    );

    // 5. Log out
    graph_service.signout(session_id).await;

    // 6. It has been verified that the session has been removed.
    assert!(graph_service
        .get_session_manager()
        .find_session(session_id)
        .is_none());
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    use tokio::task::JoinSet;

    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        DEFAULT_SESSION_IDLE_TIMEOUT,
    );

    let mut handles = JoinSet::new();

    // Concurrent session creation
    for i in 0..10 {
        let manager = Arc::clone(&session_manager);
        handles.spawn(async move {
            let session = manager
                .create_session(format!("user{}", i), "127.0.0.1".to_string())
                .await
                .expect("创建会话失败");
            session.set_role(1, RoleType::User);
            session.id()
        });
    }

    // Wait for all tasks to be completed.
    let mut session_ids = vec![];
    while let Some(result) = handles.join_next().await {
        session_ids.push(result.expect("任务失败"));
    }

    // Verify that all sessions were created successfully.
    assert_eq!(session_ids.len(), 10);

    // Verification ensures that all sessions can be found.
    for session_id in &session_ids {
        assert!(session_manager.find_session(*session_id).is_some());
    }

    // Verify the session list
    let sessions = session_manager.list_sessions();
    let sessions = sessions.await;
    assert_eq!(sessions.len(), 10);
}

#[test]
fn test_query_manager_concurrent_operations() {
    use std::thread;

    let query_manager = Arc::new(QueryManager::new());
    let mut handles = vec![];

    // Concurrent registration and query operations
    for i in 0..10 {
        let manager = Arc::clone(&query_manager);
        let handle = thread::spawn(move || {
            manager.register_query(
                1,
                "testuser".to_string(),
                None,
                format!("SELECT * FROM table{}", i),
            )
        });
        handles.push(handle);
    }

    // Wait for all threads to complete, with a timeout option.
    let mut query_ids = vec![];
    for handle in handles {
        let result = handle.join();
        match result {
            Ok(id) => query_ids.push(id),
            Err(_) => panic!("Thread panic"),
        }
    }

    // Verify that all queries have been successfully registered.
    assert_eq!(query_ids.len(), 10);

    // Verification ensures that all queries can be found.
    let all_queries = query_manager.get_all_queries();
    assert_eq!(all_queries.len(), 10);

    // Verify the queries that are currently running.
    let running_queries = query_manager.get_running_queries();
    assert_eq!(running_queries.len(), 10);
}
