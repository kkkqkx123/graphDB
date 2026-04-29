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
    client_session.set_role(0, RoleType::Admin);
    assert!(client_session.is_admin());

    // Setting up the User role
    client_session.set_role(0, RoleType::User);
    assert!(!client_session.is_admin());
}

#[tokio::test]
async fn test_client_session_auto_commit() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // Auto-commit is enabled by default
    assert!(client_session.is_auto_commit());

    // Disable auto-commit
    client_session.set_auto_commit(false);
    assert!(!client_session.is_auto_commit());

    // Re-enable auto-commit
    client_session.set_auto_commit(true);
    assert!(client_session.is_auto_commit());
}

#[tokio::test]
async fn test_client_session_transaction() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // There were no transactions at the beginning.
    assert!(client_session.current_transaction().is_none());

    // Binding transactions
    client_session.bind_transaction(456);
    assert_eq!(client_session.current_transaction(), Some(456));

    // Unbind transaction
    client_session.unbind_transaction();
    assert!(client_session.current_transaction().is_none());
}

#[tokio::test]
async fn test_client_session_query_history() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // Adding Query History
    let query1 = "SHOW SPACES".to_string();
    let query2 = "USE test_space".to_string();
    client_session.add_query(query1.clone());
    client_session.add_query(query2.clone());

    // Obtaining the query history
    let history = client_session.query_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], query1);
    assert_eq!(history[1], query2);
}

#[tokio::test]
async fn test_client_session_last_active() {
    let session = Session {
        session_id: 123,
        user_name: "testuser".to_string(),
        space_name: None,
        graph_addr: None,
        timezone: None,
    };

    let client_session = ClientSession::new(session);

    // Get the initial active time
    let initial_active = client_session.last_active();

    // Wait for a moment
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Update active time
    client_session.update_last_active();
    let new_active = client_session.last_active();

    assert!(new_active > initial_active);
}

// ==================== Authentication and Authorization Testing ====================

#[tokio::test]
async fn test_password_authenticator() {
    let authenticator = PasswordAuthenticator::new("root".to_string(), "root".to_string());

    // Correct password
    assert!(authenticator.authenticate("root", "root").await.is_ok());

    // Wrong password
    assert!(authenticator.authenticate("root", "wrong").await.is_err());

    // User does not exist
    assert!(authenticator.authenticate("unknown", "root").await.is_err());
}

#[tokio::test]
async fn test_permission_manager() {
    let permission_manager = PermissionManager::new();

    // Verify default permissions
    assert!(permission_manager.has_permission("root", Permission::Read));
    assert!(permission_manager.has_permission("root", Permission::Write));
    assert!(permission_manager.has_permission("root", Permission::Delete));
    assert!(permission_manager.has_permission("root", Permission::Admin));

    // Unknown user has no permission
    assert!(!permission_manager.has_permission("unknown", Permission::Read));
}

// ==================== Query Management Testing ====================

#[tokio::test]
async fn test_query_manager_creation() {
    let query_manager = QueryManager::new();

    // Initial state
    assert_eq!(query_manager.active_queries().await.len(), 0);
}

#[tokio::test]
async fn test_query_manager_add_and_remove() {
    let query_manager = QueryManager::new();

    // Add query
    let query_id = query_manager
        .add_query("SHOW SPACES".to_string(), 123)
        .await;
    assert_eq!(query_manager.active_queries().await.len(), 1);

    // Query status
    let status = query_manager.get_query_status(query_id).await;
    assert!(status.is_some());
    assert_eq!(status.unwrap(), QueryStatus::Running);

    // Remove query
    query_manager.remove_query(query_id).await;
    assert_eq!(query_manager.active_queries().await.len(), 0);
}

#[tokio::test]
async fn test_query_manager_cancel() {
    let query_manager = QueryManager::new();

    let query_id = query_manager
        .add_query("SHOW SPACES".to_string(), 123)
        .await;

    // Cancel query
    let result = query_manager.cancel_query(query_id).await;
    assert!(result.is_ok());

    let status = query_manager.get_query_status(query_id).await;
    assert_eq!(status, Some(QueryStatus::Cancelled));
}

#[tokio::test]
async fn test_query_manager_multiple_queries() {
    let query_manager = QueryManager::new();

    // Add multiple queries
    let q1 = query_manager.add_query("SHOW SPACES".to_string(), 123).await;
    let q2 = query_manager.add_query("USE test".to_string(), 123).await;
    let q3 = query_manager.add_query("MATCH (v) RETURN v".to_string(), 456).await;

    // Verify query count
    assert_eq!(query_manager.active_queries().await.len(), 3);

    // Filtering queries by session
    let session_queries = query_manager.get_queries_by_session(123).await;
    assert_eq!(session_queries.len(), 2);

    let other_queries = query_manager.get_queries_by_session(456).await;
    assert_eq!(other_queries.len(), 1);
    assert_eq!(other_queries[0].query_id, q3);
}

// ==================== Statistical Management Testing ====================

#[test]
fn test_stats_manager_creation() {
    let stats_manager = StatsManager::new();

    // Initial state
    assert_eq!(stats_manager.total_queries(), 0);
    assert_eq!(stats_manager.total_sessions(), 0);
    assert_eq!(stats_manager.active_connections(), 0);
}

#[test]
fn test_stats_manager_query_stats() {
    let stats_manager = StatsManager::new();

    // Record query
    stats_manager.record_query(100);
    stats_manager.record_query(200);
    stats_manager.record_query(300);

    assert_eq!(stats_manager.total_queries(), 3);

    // Average execution time
    let avg_time = stats_manager.average_query_time();
    assert_eq!(avg_time, 200.0);
}

#[test]
fn test_stats_manager_session_stats() {
    let stats_manager = StatsManager::new();

    // Record session
    stats_manager.record_session_created();
    stats_manager.record_session_created();
    assert_eq!(stats_manager.total_sessions(), 2);
    assert_eq!(stats_manager.active_connections(), 2);

    // Record session closure
    stats_manager.record_session_closed();
    assert_eq!(stats_manager.active_connections(), 1);
}

#[test]
fn test_stats_manager_metrics() {
    let stats_manager = StatsManager::new();

    // Record different types of metrics
    stats_manager.record_metric(MetricType::Query, 100);
    stats_manager.record_metric(MetricType::Session, 1);
    stats_manager.record_metric(MetricType::Storage, 1024);

    // Get metric statistics
    let query_metrics = stats_manager.get_metrics(MetricType::Query);
    assert_eq!(query_metrics.len(), 1);
    assert_eq!(query_metrics[0], 100);
}

#[test]
fn test_query_metrics() {
    let mut metrics = QueryMetrics::new();

    // Record query metrics
    metrics.record_execution(100);
    metrics.record_execution(200);

    assert_eq!(metrics.total_count(), 2);
    assert_eq!(metrics.average_time(), 150.0);

    // Record error
    metrics.record_error();
    assert_eq!(metrics.error_count(), 1);
    assert_eq!(metrics.success_rate(), 0.5);
}

// ==================== GraphService Integration Testing ====================

fn create_test_config() -> Config {
    Config::default()
}

#[tokio::test]
async fn test_graph_service_creation() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Verify GraphService creation
    assert!(graph_service.get_session_manager().list_sessions().await.is_empty());
}

#[tokio::test]
async fn test_graph_service_authentication() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Correct authentication
    let result = graph_service.authenticate("root", "root").await;
    assert!(result.is_ok());

    let session = result.expect("认证失败");
    assert_eq!(session.user(), "root");

    // Wrong password
    let result = graph_service.authenticate("root", "wrong").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graph_service_signout() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Create a session
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Verify session existence
    assert!(graph_service
        .get_session_manager()
        .find_session(session_id)
        .is_some());

    // Log out
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

    // Test SHOW SPACES
    let result = graph_service.execute(session_id, "SHOW SPACES").await;
    assert!(result.is_ok(), "SHOW SPACES should succeed: {:?}", result.err());

    // Test CREATE SPACE
    let result = graph_service.execute(session_id, "CREATE SPACE IF NOT EXISTS test_space (vid_type = FIXED_STRING(32))").await;
    assert!(result.is_ok(), "CREATE SPACE should succeed: {:?}", result.err());

    // Test USE SPACE
    let result = graph_service.execute(session_id, "USE test_space").await;
    assert!(result.is_ok(), "USE SPACE should succeed: {:?}", result.err());

    // Test CREATE TAG after USE SPACE
    let result = graph_service.execute(session_id, "CREATE TAG IF NOT EXISTS Person(name STRING, age INT)").await;
    assert!(result.is_ok(), "CREATE TAG should succeed after USE: {:?}", result.err());

    // Test SHOW TAGS
    let result = graph_service.execute(session_id, "SHOW TAGS").await;
    assert!(result.is_ok(), "SHOW TAGS should succeed: {:?}", result.err());

    // Test CREATE EDGE
    let result = graph_service.execute(session_id, "CREATE EDGE IF NOT EXISTS KNOWS(created_at TIMESTAMP)").await;
    assert!(result.is_ok(), "CREATE EDGE should succeed: {:?}", result.err());

    // Test SHOW EDGES
    let result = graph_service.execute(session_id, "SHOW EDGES").await;
    assert!(result.is_ok(), "SHOW EDGES should succeed: {:?}", result.err());
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
    let result = graph_service.execute(999999, "SHOW SPACES").await;
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
    let _ = graph_service.execute(session_id, "SHOW SPACES").await;

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

    // Collect all session IDs
    let mut session_ids = Vec::new();
    while let Some(result) = handles.join_next().await {
        session_ids.push(result.expect("任务失败"));
    }

    assert_eq!(session_ids.len(), 10);

    // Verify all sessions exist
    for id in &session_ids {
        assert!(session_manager.find_session(*id).is_some());
    }

    // Concurrently closing sessions
    let mut handles = JoinSet::new();
    for id in session_ids {
        let manager = Arc::clone(&session_manager);
        handles.spawn(async move {
            manager.remove_session(id).await;
        });
    }

    while handles.join_next().await.is_some() {}

    // Verify all sessions have been closed
    assert_eq!(session_manager.list_sessions().await.len(), 0);
}

#[tokio::test]
async fn test_session_timeout() {
    let session_manager = GraphSessionManager::new(
        "127.0.0.1:9669".to_string(),
        100,
        Duration::from_millis(50), // Set a very short timeout
    );

    // Create a session
    let session = session_manager
        .create_session("testuser".to_string(), "127.0.0.1".to_string())
        .await
        .expect("创建会话失败");
    let session_id = session.id();

    // Verify session existence
    assert!(session_manager.find_session(session_id).is_some());

    // Waiting for timeout
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify that the session has expired
    assert!(session_manager.find_session(session_id).is_none());
}

#[tokio::test]
async fn test_permission_check() {
    let permission_manager = PermissionManager::new();

    // root user has all permissions
    assert!(permission_manager.has_permission("root", Permission::Read));
    assert!(permission_manager.has_permission("root", Permission::Write));
    assert!(permission_manager.has_permission("root", Permission::Delete));
    assert!(permission_manager.has_permission("root", Permission::Admin));

    // Regular users do not have permission by default
    assert!(!permission_manager.has_permission("user1", Permission::Read));
    assert!(!permission_manager.has_permission("user1", Permission::Write));
}

#[tokio::test]
async fn test_query_execution_tracking() {
    let query_manager = QueryManager::new();

    // Simulate query execution
    let query_id = query_manager
        .add_query("SHOW SPACES".to_string(), 123)
        .await;

    // Verify query status
    let status = query_manager.get_query_status(query_id).await;
    assert_eq!(status, Some(QueryStatus::Running));

    // Simulate query completion
    query_manager.remove_query(query_id).await;
    assert_eq!(query_manager.active_queries().await.len(), 0);
}

#[tokio::test]
async fn test_stats_collection() {
    let stats_manager = StatsManager::new();

    // Record some statistical information
    stats_manager.record_query(100);
    stats_manager.record_query(200);
    stats_manager.record_session_created();

    // Verify statistics
    assert_eq!(stats_manager.total_queries(), 2);
    assert_eq!(stats_manager.total_sessions(), 1);
    assert_eq!(stats_manager.active_connections(), 1);
    assert_eq!(stats_manager.average_query_time(), 150.0);
}

#[tokio::test]
async fn test_graph_service_query_execution() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Execute query
    let result = graph_service.execute(session_id, "SHOW SPACES").await;
    assert!(result.is_ok(), "Query execution failed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_multiple_queries() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Execute multiple queries
    let queries = vec![
        "SHOW SPACES",
        "SHOW TAGS",
        "SHOW EDGES",
    ];

    for query in queries {
        let result = graph_service.execute(session_id, query).await;
        assert!(result.is_ok(), "Query '{}' failed: {:?}", query, result.err());
    }
}

#[tokio::test]
async fn test_graph_service_error_handling() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Executing an invalid query should return an error
    let result = graph_service.execute(session_id, "INVALID QUERY").await;
    assert!(result.is_err(), "Invalid query should fail");
}

#[tokio::test]
async fn test_graph_service_transaction_management() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Start transaction
    let result = graph_service.execute(session_id, "BEGIN").await;
    assert!(result.is_ok(), "BEGIN should succeed: {:?}", result.err());

    // Commit transaction
    let result = graph_service.execute(session_id, "COMMIT").await;
    assert!(result.is_ok(), "COMMIT should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_rollback_transaction() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Start transaction
    let result = graph_service.execute(session_id, "BEGIN").await;
    assert!(result.is_ok(), "BEGIN should succeed: {:?}", result.err());

    // Rollback transaction
    let result = graph_service.execute(session_id, "ROLLBACK").await;
    assert!(result.is_ok(), "ROLLBACK should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_savepoint() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Start transaction
    let result = graph_service.execute(session_id, "BEGIN").await;
    assert!(result.is_ok(), "BEGIN should succeed: {:?}", result.err());

    // Create savepoint
    let result = graph_service.execute(session_id, "SAVEPOINT sp1").await;
    assert!(result.is_ok(), "SAVEPOINT should succeed: {:?}", result.err());

    // Release savepoint
    let result = graph_service.execute(session_id, "RELEASE SAVEPOINT sp1").await;
    assert!(result.is_ok(), "RELEASE SAVEPOINT should succeed: {:?}", result.err());

    // Commit
    let result = graph_service.execute(session_id, "COMMIT").await;
    assert!(result.is_ok(), "COMMIT should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_auto_commit() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // In auto-commit mode, queries should execute normally
    let result = graph_service.execute(session_id, "SHOW SPACES").await;
    assert!(result.is_ok(), "Query in auto-commit mode should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_session_isolation() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Create two sessions
    let session1 = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session2 = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");

    // Session IDs should be different
    assert_ne!(session1.id(), session2.id());

    // Queries in different sessions should not interfere with each other
    let result1 = graph_service.execute(session1.id(), "SHOW SPACES").await;
    let result2 = graph_service.execute(session2.id(), "SHOW SPACES").await;

    assert!(result1.is_ok(), "Session 1 query should succeed");
    assert!(result2.is_ok(), "Session 2 query should succeed");
}

#[tokio::test]
async fn test_graph_service_concurrent_queries() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Concurrent query execution
    let mut handles = Vec::new();
    for i in 0..5 {
        let query = format!("SHOW SPACES -- query {}", i);
        let graph_service = graph_service.clone();
        handles.push(tokio::spawn(async move {
            graph_service.execute(session_id, &query).await
        }));
    }

    // Wait for all queries to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Concurrent query should succeed");
    }
}

#[tokio::test]
async fn test_graph_service_query_metrics() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Execute some queries
    for _ in 0..3 {
        let _ = graph_service.execute(session_id, "SHOW SPACES").await;
    }

    // Verify query metrics
    let metrics = graph_service.get_query_metrics();
    assert!(metrics.total_count() >= 3, "Should have recorded at least 3 queries");
}

#[tokio::test]
async fn test_graph_service_storage_operations() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path.clone()).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Create space
    let result = graph_service.execute(session_id, "CREATE SPACE IF NOT EXISTS test_storage (vid_type = FIXED_STRING(32))").await;
    assert!(result.is_ok(), "CREATE SPACE should succeed: {:?}", result.err());

    // Use space
    let result = graph_service.execute(session_id, "USE test_storage").await;
    assert!(result.is_ok(), "USE SPACE should succeed: {:?}", result.err());

    // Create tag
    let result = graph_service.execute(session_id, "CREATE TAG IF NOT EXISTS TestTag(name STRING)").await;
    assert!(result.is_ok(), "CREATE TAG should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_graph_service_error_recovery() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Authentication
    let session = graph_service
        .authenticate("root", "root")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Execute an invalid query
    let result = graph_service.execute(session_id, "INVALID SYNTAX").await;
    assert!(result.is_err(), "Invalid query should fail");

    // The system should still be able to execute normal queries
    let result = graph_service.execute(session_id, "SHOW SPACES").await;
    assert!(result.is_ok(), "System should recover and execute valid queries");
}

#[tokio::test]
async fn test_graph_service_permission_denied() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("graphdb_test");

    let mut config = create_test_config();
    config.database.storage_path = db_path.to_string_lossy().to_string();

    let storage = Arc::new(DefaultStorage::new_with_path(db_path).expect("创建存储失败"));
    let graph_service = GraphService::<DefaultStorage>::new(config, storage).await;

    // Create a regular user session (non-admin)
    let session = graph_service
        .authenticate("user", "user")
        .await
        .expect("认证失败");
    let session_id = session.id();

    // Regular users should be able to execute read queries
    let result = graph_service.execute(session_id, "SHOW SPACES").await;
    assert!(result.is_ok(), "Read query should succeed for regular user");
}
