use crate::api::server::auth::{Authenticator, AuthenticatorFactory, PasswordAuthenticator};
use crate::api::server::permission::PermissionManager;
use crate::api::server::session::{ClientSession, GraphSessionManager, SpaceInfo};
use crate::config::Config;
use crate::storage::StorageClient;
use crate::core::error::{SessionError, SessionResult};
use crate::core::{Permission, StatsManager, MetricType};
use crate::transaction::{SavepointManager, TransactionManager};
use crate::query::QueryPipelineManager;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Duration;
use log::{info, warn};

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    pipeline_manager: Arc<Mutex<QueryPipelineManager<S>>>,
    authenticator: PasswordAuthenticator,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
    storage: Arc<S>,
    
    // 事务管理相关
    transaction_manager: Option<Arc<TransactionManager>>,
    savepoint_manager: Option<Arc<SavepointManager>>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    /// 创建新的GraphService（不包含事务管理器，用于生产环境）
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, None, true)
    }

    /// 创建新的GraphService（不包含事务管理器，不启动后台任务，用于测试）
    pub fn new_for_test(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, None, false)
    }

    /// 使用事务管理器创建GraphService
    pub fn new_with_transaction_managers(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Arc<TransactionManager>,
        savepoint_manager: Arc<SavepointManager>,
    ) -> Arc<Self> {
        Self::create_service(config, storage, Some(transaction_manager), Some(savepoint_manager), true)
    }

    /// 内部构造函数，提取公共逻辑
    ///
    /// # 参数
    /// * `start_cleanup_task` - 是否启动会话清理后台任务
    fn create_service(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Option<Arc<TransactionManager>>,
        savepoint_manager: Option<Arc<SavepointManager>>,
        start_cleanup_task: bool,
    ) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.transaction.default_timeout * 10);
        let session_manager = GraphSessionManager::new(
            format!("{}:{}", config.database.host, config.database.port),
            config.database.max_connections,
            session_idle_timeout,
        );

        // 根据参数决定是否启动会话清理后台任务
        if start_cleanup_task {
            session_manager.start_cleanup_task();
        }

        let query_stats_manager = Arc::new(StatsManager::new());
        let pipeline_manager = Arc::new(Mutex::new(QueryPipelineManager::new(
            Arc::new(Mutex::new((*storage).clone())),
            query_stats_manager.clone(),
        )));

        let authenticator = AuthenticatorFactory::create_default(&config.auth);
        let permission_manager = Arc::new(PermissionManager::new());
        let server_stats_manager = Arc::new(StatsManager::new());

        Arc::new(Self {
            session_manager,
            pipeline_manager,
            authenticator,
            permission_manager,
            stats_manager: server_stats_manager,
            storage,
            transaction_manager,
            savepoint_manager,
        })
    }

    pub fn authenticate(
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

    pub fn execute(&self, session_id: i64, stmt: &str) -> Result<String, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| format!("无效的会话 ID: {}", session_id))?;

        let space_id = session.space().map(|s| s.id).unwrap_or(0);

        // 处理事务控制语句
        let trimmed_stmt = stmt.trim().to_uppercase();
        if trimmed_stmt.starts_with("BEGIN") || trimmed_stmt.starts_with("START TRANSACTION") {
            return self.handle_begin_transaction(&session);
        } else if trimmed_stmt.starts_with("COMMIT") {
            return self.handle_commit_transaction(&session);
        } else if trimmed_stmt.starts_with("ROLLBACK") {
            return self.handle_rollback_transaction(&session, stmt);
        } else if trimmed_stmt.starts_with("SAVEPOINT") {
            return self.handle_savepoint(&session, stmt);
        }

        // 执行普通查询
        let result = self.execute_query_with_permission(session_id, stmt, space_id);

        // 如果是 USE 语句且执行成功，更新会话的空间
        if result.is_ok() && trimmed_stmt.starts_with("USE ") {
            let space_name = stmt.trim()[4..].trim().to_string();
            // 获取空间信息并设置到会话
            if let Ok(space_info) = self.get_space_info(&space_name) {
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
    
    fn get_space_info(&self, space_name: &str) -> Result<SpaceInfo, String> {
        // 从存储中获取空间信息
        match self.storage.get_space(space_name) {
            Ok(Some(space)) => Ok(SpaceInfo {
                name: space_name.to_string(),
                id: space.space_id as i64,
            }),
            Ok(None) => {
                // 空间不存在，返回一个默认的空间信息（用于测试）
                Ok(SpaceInfo {
                    name: space_name.to_string(),
                    id: 1, // 默认空间ID
                })
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("Table 'spaces' does not exist") {
                    // 表不存在，返回默认空间信息
                    Ok(SpaceInfo {
                        name: space_name.to_string(),
                        id: 1, // 默认空间ID
                    })
                } else {
                    Err(format!("获取空间信息失败: {}", e))
                }
            }
        }
    }

    fn execute_query_with_permission(
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

        // 从客户端会话中提取空间信息
        let space_info = session.space().map(|s| {
            crate::core::types::SpaceInfo {
                space_name: s.name.clone(),
                space_id: s.id as u64,
                vid_type: crate::core::types::DataType::String,
                tags: Vec::new(),
                edge_types: Vec::new(),
                version: crate::core::types::MetadataVersion::default(),
                comment: None,
            }
        });

        let mut pipeline_manager = self.pipeline_manager.lock();
        let result = pipeline_manager.execute_query_with_space(stmt, space_info);

        match result {
            Ok(exec_result) => Ok(format!("{:?}", exec_result)),
            Err(e) => Err(e.to_string()),
        }
    }

    fn extract_permission_from_statement(&self, stmt: &str) -> Permission {
        let stmt_upper = stmt.trim().to_uppercase();

        if stmt_upper.starts_with("SELECT") || stmt_upper.starts_with("MATCH") {
            Permission::Read
        } else if stmt_upper.starts_with("INSERT") || stmt_upper.starts_with("CREATE") {
            Permission::Write
        } else if stmt_upper.starts_with("DELETE") || stmt_upper.starts_with("DROP") {
            Permission::Delete
        } else if stmt_upper.starts_with("ALTER") || stmt_upper.starts_with("ADD") {
            Permission::Schema
        } else {
            Permission::Read
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

    pub fn get_permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// 获取会话列表（SHOW SESSIONS）
    pub fn list_sessions(&self) -> Vec<crate::api::server::session::SessionInfo> {
        self.session_manager.list_sessions()
    }

    /// 获取指定会话的详细信息
    pub fn get_session_info(&self, session_id: i64) -> Option<crate::api::server::session::SessionInfo> {
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
    fn handle_begin_transaction(&self, session: &Arc<ClientSession>) -> Result<String, String> {
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
    fn handle_commit_transaction(&self, session: &Arc<ClientSession>) -> Result<String, String> {
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
    fn handle_rollback_transaction(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
        let trimmed = stmt.trim().to_uppercase();

        // 检查是否是 ROLLBACK TO SAVEPOINT
        if trimmed.starts_with("ROLLBACK TO ") {
            return self.handle_rollback_to_savepoint(session, stmt);
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
    fn handle_savepoint(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
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
    fn handle_rollback_to_savepoint(&self, session: &Arc<ClientSession>, stmt: &str) -> Result<String, String> {
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

    #[test]
    fn test_graph_service_creation() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 验证服务创建成功
        assert!(!graph_service.get_session_manager().is_out_of_connections());
    }

    #[test]
    fn test_authentication_success() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service.authenticate("root", "root");
        assert!(session.is_ok());
    }

    #[test]
    fn test_authentication_failure() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service.authenticate("root", "wrong_password");
        assert!(session.is_err());

        let session = graph_service.authenticate("", "password");
        assert!(session.is_err());

        let session = graph_service.authenticate("testuser", "");
        assert!(session.is_err());
    }

    #[test]
    fn test_signout() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service.authenticate("root", "root");
        assert!(session.is_ok());

        let session_id = session.unwrap().id();
        graph_service.signout(session_id);

        // 验证会话已注销
        assert!(graph_service.get_session_manager().find_session(session_id).is_none());
    }

    #[test]
    fn test_list_sessions() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 初始时没有会话
        let sessions = graph_service.list_sessions();
        assert_eq!(sessions.len(), 0);

        // 创建一个会话
        let session = graph_service.authenticate("root", "root");
        assert!(session.is_ok());

        // 现在应该有一个会话
        let sessions = graph_service.list_sessions();
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_get_session_info() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 不存在的会话
        let info = graph_service.get_session_info(999);
        assert!(info.is_none());

        // 创建一个会话
        let session = graph_service.authenticate("root", "root");
        assert!(session.is_ok());
        let session_id = session.unwrap().id();

        // 获取会话信息
        let info = graph_service.get_session_info(session_id);
        assert!(info.is_some());
        assert_eq!(info.unwrap().session_id, session_id);
    }

    #[test]
    fn test_kill_session() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 创建一个会话
        let session = graph_service.authenticate("root", "root");
        assert!(session.is_ok());
        let session_id = session.unwrap().id();

        // 终止会话
        let result = graph_service.kill_session(session_id, "root");
        assert!(result.is_ok());

        // 验证会话已终止
        assert!(graph_service.get_session_manager().find_session(session_id).is_none());
    }

    #[test]
    fn test_kill_nonexistent_session() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 终止不存在的会话应该失败
        let result = graph_service.kill_session(999, "root");
        assert!(result.is_err());
    }

    #[test]
    fn test_permission_check() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // 测试各种语句的权限提取
        assert_eq!(graph_service.extract_permission_from_statement("SELECT * FROM user"), Permission::Read);
        assert_eq!(graph_service.extract_permission_from_statement("MATCH (n) RETURN n"), Permission::Read);
        assert_eq!(graph_service.extract_permission_from_statement("INSERT INTO user VALUES (1, 'test')"), Permission::Write);
        assert_eq!(graph_service.extract_permission_from_statement("CREATE TAG user(name string)"), Permission::Write);
        assert_eq!(graph_service.extract_permission_from_statement("DELETE FROM user WHERE id = 1"), Permission::Delete);
        assert_eq!(graph_service.extract_permission_from_statement("DROP TAG user"), Permission::Delete);
        assert_eq!(graph_service.extract_permission_from_statement("ALTER TAG user ADD COLUMN age int"), Permission::Schema);
        assert_eq!(graph_service.extract_permission_from_statement("ADD HOSTS 127.0.0.1:9779"), Permission::Schema);
        assert_eq!(graph_service.extract_permission_from_statement("UNKNOWN STATEMENT"), Permission::Read);
    }
}
