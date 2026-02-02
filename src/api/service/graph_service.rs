use crate::api::service::{
    Authenticator, MetricType, PasswordAuthenticator, PermissionManager, QueryEngine, StatsManager,
};
use crate::api::session::{ClientSession, GraphSessionManager};
use crate::config::Config;
use crate::storage::{StorageClient, TransactionId};
use crate::core::error::{SessionError, SessionResult};
use crate::utils::safe_lock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 事务状态枚举
#[derive(Debug, Clone, PartialEq)]
enum TxState {
    Active,
    Committed,
    RolledBack,
}

/// 事务管理器
#[derive(Debug)]
pub struct TransactionManager {
    next_tx_id: Arc<Mutex<u64>>,
    tx_states: Arc<Mutex<HashMap<TransactionId, TxState>>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            next_tx_id: Arc::new(Mutex::new(1)),
            tx_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn begin_transaction(&self) -> TransactionId {
        let mut id = self.next_tx_id.lock().unwrap();
        let tx_id = *id;
        *id += 1;

        // 记录事务状态为活跃
        let mut states = self.tx_states.lock().unwrap();
        states.insert(TransactionId(tx_id), TxState::Active);

        TransactionId(tx_id)
    }

    pub fn commit_transaction(&self, tx_id: TransactionId) -> Result<(), String> {
        let mut states = self.tx_states.lock().unwrap();

        // 检查事务是否存在且处于活跃状态
        match states.get(&tx_id) {
            Some(TxState::Active) => {
                // 更新事务状态为已提交
                states.insert(tx_id, TxState::Committed);
                Ok(())
            }
            Some(TxState::Committed) => {
                Err(format!("事务 {} 已经提交过了", tx_id))
            }
            Some(TxState::RolledBack) => {
                Err(format!("事务 {} 已经回滚了，无法提交", tx_id))
            }
            None => {
                Err(format!("事务 {} 不存在", tx_id))
            }
        }
    }

    pub fn rollback_transaction(&self, tx_id: TransactionId) -> Result<(), String> {
        let mut states = self.tx_states.lock().unwrap();

        // 检查事务是否存在且处于活跃状态
        match states.get(&tx_id) {
            Some(TxState::Active) => {
                // 更新事务状态为已回滚
                states.insert(tx_id, TxState::RolledBack);
                Ok(())
            }
            Some(TxState::Committed) => {
                Err(format!("事务 {} 已经提交了，无法回滚", tx_id))
            }
            Some(TxState::RolledBack) => {
                Err(format!("事务 {} 已经回滚了", tx_id))
            }
            None => {
                Err(format!("事务 {} 不存在", tx_id))
            }
        }
    }

    /// 检查事务是否处于活跃状态
    pub fn is_active(&self, tx_id: TransactionId) -> bool {
        let states = self.tx_states.lock().unwrap();
        matches!(states.get(&tx_id), Some(TxState::Active))
    }

    /// 清理已完成的事务
    pub fn cleanup_completed_transactions(&self) {
        let mut states = self.tx_states.lock().unwrap();
        states.retain(|_, state| *state == TxState::Active);
    }

    /// 获取事务状态
    pub fn get_transaction_state(&self, tx_id: TransactionId) -> Option<TxState> {
        let states = self.tx_states.lock().unwrap();
        states.get(&tx_id).cloned()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine<S>>>,
    authenticator: Arc<PasswordAuthenticator>,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
    config: Config,
    transaction_manager: Arc<Mutex<TransactionManager>>,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.transaction_timeout * 10);
        let session_manager = GraphSessionManager::new(
            format!("{}:{}", config.host, config.port),
            config.max_connections,
            session_idle_timeout,
        );
        let query_engine = Arc::new(Mutex::new(QueryEngine::new(storage.clone())));
        let authenticator = Arc::new(PasswordAuthenticator::new());
        let permission_manager = Arc::new(PermissionManager::new());
        let stats_manager = Arc::new(StatsManager::new());
        let transaction_manager = Arc::new(Mutex::new(TransactionManager::new()));

        Arc::new(Self {
            session_manager,
            query_engine,
            authenticator,
            permission_manager,
            stats_manager,
            config,
            transaction_manager,
            storage: Arc::new(Mutex::new(storage.as_ref().clone())),
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

        let space_id = session.space().map(|s| s.id).unwrap_or(0);

        self.execute_with_permission(session_id, stmt, space_id).await
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

        let request_context = crate::api::service::query_processor::RequestContext {
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

    /// 开始新事务
    pub fn begin_transaction(&self, _session_id: i64) -> Result<TransactionId, String> {
        let mut tx_manager = self.transaction_manager
            .lock()
            .map_err(|e| format!("获取事务管理器锁失败: {}", e))?;

        let tx_id = tx_manager.begin_transaction();

        // 同时在存储层开始事务
        let mut storage = self.storage.lock().unwrap();
        storage
            .begin_transaction("")  // 使用默认空间名
            .map_err(|e| format!("在存储层开始事务失败: {:?}", e))?;

        drop(storage);
        Ok(tx_id)
    }

    /// 提交事务
    pub fn commit_transaction(&self, tx_id: TransactionId) -> Result<(), String> {
        // 先验证事务状态
        let tx_manager = self.transaction_manager
            .lock()
            .map_err(|e| format!("获取事务管理器锁失败: {}", e))?;

        if !tx_manager.is_active(tx_id) {
            return Err(format!("事务 {} 不处于活跃状态，无法提交", tx_id));
        }

        // 释放锁，避免死锁
        drop(tx_manager);

        // 在存储层提交事务
        let mut storage = self.storage.lock().unwrap();
        let result = storage
            .commit_transaction("", tx_id)  // 使用默认空间名
            .map_err(|e| format!("在存储层提交事务失败: {:?}", e));

        drop(storage);

        // 更新服务层事务状态
        let mut tx_manager = self.transaction_manager
            .lock()
            .map_err(|e| format!("获取事务管理器锁失败: {}", e))?;

        match result {
            Ok(()) => {
                tx_manager.commit_transaction(tx_id)
                    .map_err(|e| format!("提交事务失败: {}", e))
            },
            Err(e) => {
                // 如果存储层提交失败，也要在服务层标记为失败
                Err(e)
            }
        }
    }

    /// 回滚事务
    pub fn rollback_transaction(&self, tx_id: TransactionId) -> Result<(), String> {
        // 先验证事务状态
        let tx_manager = self.transaction_manager
            .lock()
            .map_err(|e| format!("获取事务管理器锁失败: {}", e))?;

        if !tx_manager.is_active(tx_id) {
            return Err(format!("事务 {} 不处于活跃状态，无法回滚", tx_id));
        }

        // 释放锁，避免死锁
        drop(tx_manager);

        // 在存储层回滚事务
        let mut storage = self.storage.lock().unwrap();
        let result = storage
            .rollback_transaction("", tx_id)  // 使用默认空间名
            .map_err(|e| format!("在存储层回滚事务失败: {:?}", e));

        drop(storage);

        // 更新服务层事务状态
        let mut tx_manager = self.transaction_manager
            .lock()
            .map_err(|e| format!("获取事务管理器锁失败: {}", e))?;

        match result {
            Ok(()) => {
                tx_manager.rollback_transaction(tx_id)
                    .map_err(|e| format!("回滚事务失败: {}", e))
            },
            Err(e) => {
                // 如果存储层回滚失败，也要在服务层标记为失败
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::query::optimizer::rule_registry::RuleRegistry;
    use crate::storage::redb_storage::DefaultStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_graph_service_creation() {
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
        };

        let storage = Arc::new(DefaultStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<DefaultStorage>::new(config, storage);

        assert_eq!(graph_service.config.host, "127.0.0.1");
        assert_eq!(graph_service.config.port, 9669);
    }

    #[tokio::test]
    async fn test_authentication_success() {
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
        };

        let storage = Arc::new(DefaultStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<DefaultStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "root").await;
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
        };

        let storage = Arc::new(DefaultStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<DefaultStorage>::new(config, storage);

        let session = graph_service.authenticate("root", "wrong_password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("", "password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("testuser", "").await;
        assert!(session.is_err());
    }

    #[tokio::test]
    async fn test_signout() {
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
        };

        let storage = Arc::new(DefaultStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<DefaultStorage>::new(config, storage);

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
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
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
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
        };

        let storage = Arc::new(MemoryStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MemoryStorage>::new(config, storage);

        let result = graph_service.execute(999999, "SHOW SPACES").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_permission_extraction() {
        RuleRegistry::initialize();
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: "/tmp/graphdb_test".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_file: "logs/test.log".to_string(),
            max_log_file_size: 100 * 1024 * 1024,
            max_log_files: 5,
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
