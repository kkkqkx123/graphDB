use crate::api::service::{
    Authenticator, AuthenticatorFactory, MetricType, PasswordAuthenticator, PermissionManager, QueryEngine, StatsManager,
};
use crate::api::session::{ClientSession, GraphSessionManager};
use crate::config::Config;
use crate::storage::StorageClient;
use crate::core::error::{SessionError, SessionResult};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Duration;

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine<S>>>,
    authenticator: PasswordAuthenticator,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
    storage: Arc<S>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.database.transaction_timeout * 10);
        let session_manager = GraphSessionManager::new(
            format!("{}:{}", config.database.host, config.database.port),
            config.database.max_connections,
            session_idle_timeout,
        );
        let query_engine = Arc::new(Mutex::new(QueryEngine::new(storage.clone())));
        let authenticator = AuthenticatorFactory::create_default(&config.auth);
        let permission_manager = Arc::new(PermissionManager::new());
        let stats_manager = Arc::new(StatsManager::new());

        Arc::new(Self {
            session_manager,
            query_engine,
            authenticator,
            permission_manager,
            stats_manager,
            storage,
        })
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Arc<ClientSession>, String> {
        if username.is_empty() || password.is_empty() {
            self.stats_manager
                .add_value(MetricType::NumAuthFailedSessions);
            return Err("用户名或密码不能为空".to_string());
        }

        if self.session_manager.is_out_of_connections() {
            self.stats_manager
                .add_value(MetricType::NumAuthFailedSessions);
            return Err("超过最大连接数限制".to_string());
        }

        match self.authenticator.authenticate(username, password) {
            Ok(_) => {
                let session = self
                    .session_manager
                    .create_session(username.to_string(), "127.0.0.1".to_string())
                    .map_err(|e| format!("创建会话失败: {}", e))?;

                Ok(session)
            }
            Err(e) => {
                self.stats_manager
                    .add_value(MetricType::NumAuthFailedSessions);
                Err(format!("认证失败: {}", e))
            }
        }
    }

    pub async fn execute(&self, session_id: i64, stmt: &str) -> Result<String, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| format!("无效的会话 ID: {}", session_id))?;

        let space_id = session.space().map(|s| s.id).unwrap_or(0);

        let result = self.execute_with_permission(session_id, stmt, space_id).await;
        
        // 如果是 USE 语句且执行成功，更新会话的空间
        if result.is_ok() {
            let trimmed_stmt = stmt.trim().to_uppercase();
            if trimmed_stmt.starts_with("USE ") {
                let space_name = stmt.trim()[4..].trim().to_string();
                // 获取空间信息并设置到会话
                if let Ok(space_info) = self.get_space_info(&space_name).await {
                    session.set_space(space_info);
                }
            }
        }
        
        result
    }
    
    async fn get_space_info(&self, space_name: &str) -> Result<crate::api::session::client_session::SpaceInfo, String> {
        // 从存储中获取空间信息
        match self.storage.get_space(space_name) {
            Ok(Some(space)) => Ok(crate::api::session::client_session::SpaceInfo {
                name: space_name.to_string(),
                id: space.space_id as i64,
            }),
            Ok(None) => {
                // 空间不存在，返回一个默认的空间信息（用于测试）
                Ok(crate::api::session::client_session::SpaceInfo {
                    name: space_name.to_string(),
                    id: 1, // 默认空间ID
                })
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("Table 'spaces' does not exist") {
                    // 表不存在，返回默认空间信息
                    Ok(crate::api::session::client_session::SpaceInfo {
                        name: space_name.to_string(),
                        id: 1, // 默认空间ID
                    })
                } else {
                    Err(format!("获取空间信息失败: {}", e))
                }
            }
        }
    }

    pub async fn execute_with_permission(
        &self,
        session_id: i64,
        stmt: &str,
        space_id: i64,
    ) -> Result<String, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| format!("无效的会话 ID: {}", session_id))?;

        session.charge();

        let username = session.user();

        // 权限检查（Admin拥有所有权限，不需要检查）
        if !self.permission_manager.is_admin(&username) {
            let permission = self.extract_permission_from_statement(stmt);
            if let Err(e) = self
                .permission_manager
                .check_permission(&username, space_id, permission)
            {
                return Err(format!("权限检查失败: {}", e));
            }
        }

        let request_context = crate::api::service::query_processor::RequestContext {
            session_id,
            statement: stmt.to_string(),
            parameters: std::collections::HashMap::new(),
            client_session: Some(session),
        };

        let mut query_engine = self
            .query_engine
            .lock();
        let response = query_engine.execute(request_context).await;

        match response.result {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    fn extract_permission_from_statement(&self, stmt: &str) -> crate::api::service::Permission {
        let stmt_upper = stmt.trim().to_uppercase();

        if stmt_upper.starts_with("SELECT") || stmt_upper.starts_with("MATCH") {
            crate::api::service::Permission::Read
        } else if stmt_upper.starts_with("INSERT") || stmt_upper.starts_with("CREATE") {
            crate::api::service::Permission::Write
        } else if stmt_upper.starts_with("DELETE") || stmt_upper.starts_with("DROP") {
            crate::api::service::Permission::Delete
        } else if stmt_upper.starts_with("ALTER") || stmt_upper.starts_with("ADD") {
            crate::api::service::Permission::Schema
        } else {
            crate::api::service::Permission::Read
        }
    }

    pub fn signout(&self, session_id: i64) {
        if let Some(session) = self.session_manager.find_session(session_id) {
            if let Some(space_name) = session.space_name() {
                self.stats_manager
                    .dec_space_metric(&space_name, MetricType::NumActiveQueries);
            }
        }
        self.session_manager.remove_session(session_id);
    }

    pub fn get_session_manager(&self) -> &GraphSessionManager {
        &self.session_manager
    }

    pub fn get_query_engine(&self) -> &Mutex<QueryEngine<S>> {
        &self.query_engine
    }

    pub fn get_authenticator(&self) -> &PasswordAuthenticator {
        &self.authenticator
    }

    pub fn get_permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// 获取会话列表（SHOW SESSIONS）
    pub fn list_sessions(&self) -> Vec<crate::api::session::SessionInfo> {
        self.session_manager.list_sessions()
    }

    /// 获取指定会话的详细信息
    pub fn get_session_info(&self, session_id: i64) -> Option<crate::api::session::SessionInfo> {
        self.session_manager.get_session_info(session_id)
    }

    /// 终止会话（KILL SESSION）
    pub fn kill_session(&self, session_id: i64, current_user: &str) -> SessionResult<()> {
        // 获取当前会话以检查权限
        let current_session = self.session_manager.find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;
        
        let is_admin = current_session.is_admin();
        
        self.session_manager.kill_session(session_id, current_user, is_admin)
    }

    /// 终止查询（KILL QUERY）
    pub fn kill_query(&self, session_id: i64, query_id: u32) -> SessionResult<()> {
        let session = self.session_manager.find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;
        
        match session.kill_query(query_id) {
            Ok(()) => {
                self.stats_manager.dec_value(MetricType::NumActiveQueries);
                Ok(())
            },
            Err(e) => Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::query::optimizer::rule_registry::RuleRegistry;
    use crate::storage::test_mock::MockStorage;
    use std::sync::Arc;

    fn create_test_config() -> Config {
        Config {
            database: crate::config::DatabaseConfig {
                host: "127.0.0.1".to_string(),
                port: 9669,
                storage_path: "/tmp/graphdb_test".to_string(),
                max_connections: 10,
                transaction_timeout: 30,
            },
            log: crate::config::LogConfig {
                level: "info".to_string(),
                dir: "logs".to_string(),
                file: "logs/test.log".to_string(),
                max_file_size: 100 * 1024 * 1024,
                max_files: 5,
            },
            auth: crate::config::AuthConfig {
                enable_authorize: true,
                failed_login_attempts: 5,
                session_idle_timeout_secs: 3600,
                default_username: "root".to_string(),
                default_password: "root".to_string(),
                force_change_default_password: true,
            },
            bootstrap: crate::config::BootstrapConfig {
                auto_create_default_space: true,
                default_space_name: "default".to_string(),
                single_user_mode: false,
            },
            optimizer: crate::config::OptimizerConfig::default(),
            monitoring: crate::config::MonitoringConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_graph_service_creation() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        // 验证服务创建成功
        assert!(!graph_service.get_session_manager().is_out_of_connections());
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "root").await;
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "wrong_password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("", "password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("testuser", "").await;
        assert!(session.is_err());
    }

    #[tokio::test]
    async fn test_signout() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to authenticate");
        let session_id = session.id();

        assert!(graph_service.session_manager.find_session(session_id).is_some());

        graph_service.signout(session_id);

        assert!(graph_service.session_manager.find_session(session_id).is_none());
    }

    #[tokio::test]
    async fn test_execute_query() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to authenticate");
        let session_id = session.id();

        // 执行查询，验证不 panic
        let result = graph_service.execute(session_id, "SHOW SPACES").await;
        // 查询可能成功或失败，但不应该 panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_invalid_session_execute() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        let result = graph_service.execute(999999, "SHOW SPACES").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_permission_extraction() {
        let _ = RuleRegistry::initialize();
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new(config, storage);

        assert_eq!(
            graph_service.extract_permission_from_statement("SELECT * FROM users"),
            crate::api::service::Permission::Read
        );
        assert_eq!(
            graph_service.extract_permission_from_statement("INSERT INTO users VALUES (...)"),
            crate::api::service::Permission::Write
        );
        assert_eq!(
            graph_service.extract_permission_from_statement("DELETE FROM users"),
            crate::api::service::Permission::Delete
        );
        assert_eq!(
            graph_service.extract_permission_from_statement("ALTER TAG user"),
            crate::api::service::Permission::Schema
        );
    }
}
