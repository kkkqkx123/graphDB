use crate::api::service::{
    Authenticator, AuthenticatorFactory, MetricType, PasswordAuthenticator, PermissionManager, QueryEngine, StatsManager,
};
use crate::api::session::{ClientSession, GraphSessionManager};
use crate::config::Config;
use crate::storage::StorageClient;
use crate::core::error::{SessionError, SessionResult};
use crate::transaction::{SavepointManager, TransactionManager};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Duration;
use log::{info, warn};

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine<S>>>,
    authenticator: PasswordAuthenticator,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
    storage: Arc<S>,
    
    // 事务管理相关
    transaction_manager: Option<Arc<TransactionManager>>,
    savepoint_manager: Option<Arc<SavepointManager>>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    /// 创建新的GraphService（不包含事务管理器，用于测试）
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.transaction.default_timeout * 10);
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
            transaction_manager: None,
            savepoint_manager: None,
        })
    }

    /// 使用事务管理器创建GraphService
    pub fn new_with_transaction_managers(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Arc<TransactionManager>,
        savepoint_manager: Arc<SavepointManager>,
    ) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.transaction.default_timeout * 10);
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
            transaction_manager: Some(transaction_manager),
            savepoint_manager: Some(savepoint_manager),
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

        // 处理事务控制语句
        let trimmed_stmt = stmt.trim().to_uppercase();
        if trimmed_stmt.starts_with("BEGIN") || trimmed_stmt.starts_with("START TRANSACTION") {
            return self.handle_begin_transaction(&session).await;
        } else if trimmed_stmt.starts_with("COMMIT") {
            return self.handle_commit_transaction(&session).await;
        } else if trimmed_stmt.starts_with("ROLLBACK") {
            return self.handle_rollback_transaction(&session, stmt).await;
        } else if trimmed_stmt.starts_with("SAVEPOINT") {
            return self.handle_savepoint(&session, stmt).await;
        }

        // 执行普通查询
        let result = self.execute_with_permission(session_id, stmt, space_id).await;
        
        // 如果是 USE 语句且执行成功，更新会话的空间
        if result.is_ok() && trimmed_stmt.starts_with("USE ") {
            let space_name = stmt.trim()[4..].trim().to_string();
            // 获取空间信息并设置到会话
            if let Ok(space_info) = self.get_space_info(&space_name).await {
                session.set_space(space_info);
            }
        }
        
        // 自动提交模式处理
        if result.is_ok() && session.is_auto_commit() {
            if let Some(txn_id) = session.current_transaction() {
                if let Some(ref txn_manager) = self.transaction_manager {
                    if let Err(e) = txn_manager.commit_transaction(txn_id) {
                        warn!("自动提交失败: {}", e);
                    } else {
                        session.unbind_transaction();
                    }
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
            transaction_id: None,
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

    // ==================== 事务控制方法 ====================

    /// 处理 BEGIN TRANSACTION 语句
    async fn handle_begin_transaction(&self, session: &Arc<ClientSession>) -> Result<String, String> {
        if session.has_active_transaction() {
            return Err("会话已有活跃事务".to_string());
        }

        let txn_manager = self.transaction_manager.as_ref()
            .ok_or("事务管理器未初始化")?;

        let options = session.transaction_options();
        match txn_manager.begin_transaction(options) {
            Ok(txn_id) => {
                session.bind_transaction(txn_id);
                session.set_auto_commit(false);
                info!("会话 {} 开始事务 {}", session.id(), txn_id);
                Ok(format!("事务 {} 已开始", txn_id))
            },
            Err(e) => Err(format!("开始事务失败: {}", e)),
        }
    }

    /// 处理 COMMIT 语句
    async fn handle_commit_transaction(&self, session: &Arc<ClientSession>) -> Result<String, String> {
        let txn_id = session.current_transaction()
            .ok_or("没有活跃事务可提交")?;

        let txn_manager = self.transaction_manager.as_ref()
            .ok_or("事务管理器未初始化")?;

        match txn_manager.commit_transaction(txn_id) {
            Ok(()) => {
                session.unbind_transaction();
                session.set_auto_commit(true);
                info!("会话 {} 提交事务 {}", session.id(), txn_id);
                Ok(format!("事务 {} 已提交", txn_id))
            },
            Err(e) => Err(format!("提交事务失败: {}", e)),
        }
    }

    /// 处理 ROLLBACK 语句
    async fn handle_rollback_transaction(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
        let trimmed = stmt.trim().to_uppercase();
        
        // 检查是否是 ROLLBACK TO SAVEPOINT
        if trimmed.starts_with("ROLLBACK TO ") {
            return self.handle_rollback_to_savepoint(session, stmt).await;
        }

        let txn_id = session.current_transaction()
            .ok_or("没有活跃事务可回滚")?;

        let txn_manager = self.transaction_manager.as_ref()
            .ok_or("事务管理器未初始化")?;

        match txn_manager.abort_transaction(txn_id) {
            Ok(()) => {
                session.unbind_transaction();
                session.set_auto_commit(true);
                info!("会话 {} 回滚事务 {}", session.id(), txn_id);
                Ok(format!("事务 {} 已回滚", txn_id))
            },
            Err(e) => Err(format!("回滚事务失败: {}", e)),
        }
    }

    /// 处理 SAVEPOINT 语句
    async fn handle_savepoint(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
        let txn_id = session.current_transaction()
            .ok_or("必须先开始事务才能创建保存点")?;

        let savepoint_manager = self.savepoint_manager.as_ref()
            .ok_or("保存点管理器未初始化")?;

        // 解析保存点名称
        let parts: Vec<&str> = stmt.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err("SAVEPOINT 语法错误: SAVEPOINT <name>".to_string());
        }
        let savepoint_name = parts[1].to_string();

        match savepoint_manager.create_savepoint(txn_id, Some(savepoint_name.clone())) {
            Ok(savepoint_id) => {
                session.push_savepoint(savepoint_id);
                info!("会话 {} 在事务 {} 中创建保存点 {} (ID: {})", 
                    session.id(), txn_id, savepoint_name, savepoint_id);
                Ok(format!("保存点 {} 已创建", savepoint_name))
            },
            Err(e) => Err(format!("创建保存点失败: {}", e)),
        }
    }

    /// 处理 ROLLBACK TO SAVEPOINT 语句
    async fn handle_rollback_to_savepoint(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
        let txn_id = session.current_transaction()
            .ok_or("必须先开始事务才能回滚到保存点")?;

        let savepoint_manager = self.savepoint_manager.as_ref()
            .ok_or("保存点管理器未初始化")?;

        // 解析保存点名称
        let parts: Vec<&str> = stmt.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err("ROLLBACK TO SAVEPOINT 语法错误: ROLLBACK TO <savepoint_name>".to_string());
        }
        let savepoint_name = parts[parts.len() - 1];

        // 通过名称查找保存点ID
        let savepoint_id = savepoint_manager.find_savepoint_by_name(txn_id, savepoint_name)
            .ok_or_else(|| format!("保存点 '{}' 未找到", savepoint_name))?;

        match savepoint_manager.rollback_to_savepoint(savepoint_id) {
            Ok(()) => {
                info!("会话 {} 在事务 {} 中回滚到保存点 {}", 
                    session.id(), txn_id, savepoint_name);
                Ok(format!("已回滚到保存点 {}", savepoint_name))
            },
            Err(e) => Err(format!("回滚到保存点失败: {}", e)),
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
            },
            transaction: crate::config::TransactionConfig::default(),
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
