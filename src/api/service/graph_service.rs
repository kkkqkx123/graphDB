use crate::api::service::{
    Authenticator, MetricType, PasswordAuthenticator, PermissionManager, QueryEngine, StatsManager,
};
use crate::api::session::{ClientSession, GraphSessionManager};
use crate::config::Config;
use crate::storage::StorageEngine;
use crate::core::error::{SessionError, SessionResult};
use std::sync::{Arc, Mutex};

pub struct GraphService<S: StorageEngine + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine<S>>>,
    authenticator: Arc<PasswordAuthenticator>,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
    config: Config,
}

impl<S: StorageEngine + Clone + 'static> GraphService<S> {
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        let session_manager = GraphSessionManager::new(format!("{}:{}", config.host, config.port));
        let query_engine = Arc::new(Mutex::new(QueryEngine::new(storage)));
        let authenticator = Arc::new(PasswordAuthenticator::new());
        let permission_manager = Arc::new(PermissionManager::new());
        let stats_manager = Arc::new(StatsManager::new());

        Arc::new(Self {
            session_manager,
            query_engine,
            authenticator,
            permission_manager,
            stats_manager,
            config,
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
            self.stats_manager
                .add_value(MetricType::NumAuthFailedSessionsOutOfMaxAllowed);
            return Err("超过最大连接数限制".to_string());
        }

        match self.authenticator.authenticate(username, password) {
            Ok(_) => {
                let session = self
                    .session_manager
                    .create_session(username.to_string(), "127.0.0.1".to_string())
                    .map_err(|e| format!("创建会话失败: {}", e))?;

                self.stats_manager.add_value(MetricType::NumOpenedSessions);
                self.stats_manager.add_value(MetricType::NumActiveSessions);

                Ok(session)
            }
            Err(e) => {
                self.stats_manager
                    .add_value(MetricType::NumAuthFailedSessions);
                self.stats_manager
                    .add_value(MetricType::NumAuthFailedSessionsBadUserNamePassword);
                Err(format!("认证失败: {}", e))
            }
        }
    }

    pub async fn execute(&self, session_id: i64, stmt: &str) -> Result<String, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| "无效的会话 ID".to_string())?;

        session.charge();

        self.stats_manager.add_value(MetricType::NumQueries);
        self.stats_manager.add_value(MetricType::NumActiveQueries);

        let space_name = session.space_name();
        if let Some(ref name) = space_name {
            self.stats_manager
                .add_space_metric(name, MetricType::NumQueries);
            self.stats_manager
                .add_space_metric(name, MetricType::NumActiveQueries);
        }

        let request_context = crate::api::service::query_engine::RequestContext {
            session_id,
            statement: stmt.to_string(),
            parameters: std::collections::HashMap::new(),
            client_session: Some(session),
        };

        let mut query_engine = self
            .query_engine
            .lock()
            .expect("查询引擎锁被污染");
        let response = query_engine.execute(request_context).await;

        self.stats_manager.dec_value(MetricType::NumActiveQueries);
        if let Some(ref name) = space_name {
            self.stats_manager
                .dec_space_metric(name, MetricType::NumActiveQueries);
        }

        match response.result {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
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
            .ok_or_else(|| "无效的会话 ID".to_string())?;

        session.charge();

        let username = session.user();

        if !self.permission_manager.is_god(&username) {
            let permission = self.extract_permission_from_statement(stmt);
            if let Err(e) = self
                .permission_manager
                .check_permission(&username, space_id, permission)
            {
                return Err(format!("权限检查失败: {}", e));
            }
        }

        let request_context = crate::api::service::query_engine::RequestContext {
            session_id,
            statement: stmt.to_string(),
            parameters: std::collections::HashMap::new(),
            client_session: Some(session),
        };

        let mut query_engine = self
            .query_engine
            .lock()
            .expect("查询引擎锁被污染");
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
            self.stats_manager.dec_value(MetricType::NumActiveSessions);
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
        
        let is_god = current_session.is_god();
        
        match self.session_manager.kill_session(session_id, current_user, is_god) {
            Ok(()) => {
                self.stats_manager.dec_value(MetricType::NumActiveSessions);
                Ok(())
            },
            Err(e) => Err(e)
        }
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
    use crate::storage::MemoryStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_graph_service_creation() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        assert_eq!(graph_service.config.host, "127.0.0.1");
        assert_eq!(graph_service.config.port, 9669);
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "root").await;
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "wrong_password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("", "password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("testuser", "").await;
        assert!(session.is_err());
    }

    #[tokio::test]
    async fn test_signout() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

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
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to authenticate");
        let session_id = session.id();

        let _result = graph_service.execute(session_id, "SHOW SPACES").await;
    }

    #[tokio::test]
    async fn test_invalid_session_execute() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        let result = graph_service.execute(999999, "SHOW SPACES").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_permission_extraction() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

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
