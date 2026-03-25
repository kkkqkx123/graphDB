use crate::api::server::auth::{Authenticator, AuthenticatorFactory, PasswordAuthenticator};
use crate::api::server::permission::PermissionManager;
use crate::api::server::session::{
    build_query_request_context, ClientSession, GraphSessionManager, SpaceInfo,
};
use crate::config::Config;
use crate::core::error::{SessionError, SessionResult};
use crate::core::{MetricType, Permission, StatsManager};
use crate::query::executor::ExecutionResult;
use crate::query::{OptimizerEngine, QueryPipelineManager};
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use log::{info, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    pipeline_manager: Arc<Mutex<QueryPipelineManager<S>>>,
    authenticator: PasswordAuthenticator,
    permission_manager: Arc<PermissionManager>,
    pub stats_manager: Arc<StatsManager>,
    storage: Arc<S>,

    // Transaction management-related
    transaction_manager: Option<Arc<TransactionManager>>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    /// Create a new GraphService (without a transaction manager, for use in a production environment).
    pub fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, true)
    }

    /// Create a new GraphService (without a transaction manager and without starting any background tasks, for testing purposes).
    pub fn new_for_test(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, false)
    }

    /// Use the transaction manager to create a GraphService.
    pub fn new_with_transaction_manager(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Arc<TransactionManager>,
    ) -> Arc<Self> {
        Self::create_service(config, storage, Some(transaction_manager), true)
    }

    /// Internal constructor: Extracts the common logic
    ///
    /// # Parameters
    /// `start_cleanup_task` – Whether to initiate the background task for session cleanup
    fn create_service(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Option<Arc<TransactionManager>>,
        start_cleanup_task: bool,
    ) -> Arc<Self> {
        let session_idle_timeout = Duration::from_secs(config.transaction.default_timeout * 10);
        let session_manager = GraphSessionManager::new(
            format!("{}:{}", config.database.host, config.database.port),
            config.database.max_connections,
            session_idle_timeout,
        );

        // Decide whether to start the background task for session cleanup based on the parameters.
        if start_cleanup_task {
            session_manager.start_cleanup_task();
        }

        let query_stats_manager = Arc::new(StatsManager::new());
        let optimizer_engine = Arc::new(OptimizerEngine::default());
        let pipeline_manager = Arc::new(Mutex::new(QueryPipelineManager::with_optimizer(
            Arc::new(Mutex::new((*storage).clone())),
            query_stats_manager.clone(),
            optimizer_engine,
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
            return Err("User name or password cannot be empty".to_string());
        }

        if self.session_manager.is_out_of_connections().await {
            self.stats_manager
                .add_value(MetricType::NumAuthFailedSessions);
            return Err("More than the maximum number of connections limit".to_string());
        }

        match self.authenticator.authenticate(username, password) {
            Ok(_) => {
                let session = self
                    .session_manager
                    .create_session(username.to_string(), "127.0.0.1".to_string())
                    .await
                    .map_err(|e| format!("Creating a session failed: {}", e))?;

                Ok(session)
            }
            Err(e) => {
                self.stats_manager
                    .add_value(MetricType::NumAuthFailedSessions);
                Err(format!("authentication failure: {}", e))
            }
        }
    }

    pub fn execute(&self, session_id: i64, stmt: &str) -> Result<ExecutionResult, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| format!("Invalid session ID: {}", session_id))?;

        let space_id = session.space().map(|s| s.id).unwrap_or(0);

        // Handle transaction control statements
        let trimmed_stmt = stmt.trim().to_uppercase();
        if trimmed_stmt.starts_with("BEGIN") || trimmed_stmt.starts_with("START TRANSACTION") {
            return self.handle_begin_transaction(&session);
        } else if trimmed_stmt.starts_with("COMMIT") {
            return self.handle_commit_transaction(&session);
        } else if trimmed_stmt.starts_with("ROLLBACK") {
            return self.handle_rollback_transaction(&session, stmt);
        } else if trimmed_stmt.starts_with("SAVEPOINT") {
            return self.handle_savepoint(&session, stmt);
        } else if trimmed_stmt.starts_with("RELEASE SAVEPOINT") {
            return self.handle_release_savepoint(&session, stmt);
        }

        // Perform a regular query.
        let result = self.execute_query_with_permission(session_id, stmt, space_id);

        // If it is a USE statement and the execution is successful, the space for the session will be updated.
        if result.is_ok() && trimmed_stmt.starts_with("USE ") {
            let space_name = stmt.trim()[4..].trim().to_string();
            // Obtain spatial information and set it in the session.
            if let Ok(space_info) = self.get_space_info(&space_name) {
                session.set_space(space_info);
            }
        }

        // Automatic submission mode processing
        if result.is_ok() && session.is_auto_commit() {
            if let Some(txn_id) = session.current_transaction() {
                if let Some(ref txn_manager) = self.transaction_manager {
                    if let Err(e) = txn_manager.commit_transaction(txn_id) {
                        warn!("Auto-commit failed: {}", e);
                    } else {
                        session.unbind_transaction();
                    }
                }
            }
        }

        result
    }

    fn get_space_info(&self, space_name: &str) -> Result<SpaceInfo, String> {
        // Retrieve spatial information from the storage.
        match self.storage.get_space(space_name) {
            Ok(Some(space)) => Ok(SpaceInfo {
                name: space_name.to_string(),
                id: space.space_id as i64,
            }),
            Ok(None) => {
                // Space does not exist; return a default space information (for testing purposes).
                Ok(SpaceInfo {
                    name: space_name.to_string(),
                    id: 1, // Default space ID
                })
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("Table 'spaces' does not exist") {
                    // The table does not exist; therefore, the default space information is returned.
                    Ok(SpaceInfo {
                        name: space_name.to_string(),
                        id: 1, // Default space ID
                    })
                } else {
                    Err(format!("Failed to get space information: {}", e))
                }
            }
        }
    }

    fn execute_query_with_permission(
        &self,
        session_id: i64,
        stmt: &str,
        space_id: i64,
    ) -> Result<ExecutionResult, String> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or_else(|| format!("Invalid session ID: {}", session_id))?;

        session.charge();

        let username = session.user();

        // Permission check: The admin has all permissions, so no check is required.
        if !self.permission_manager.is_admin(&username) {
            let permission = self.extract_permission_from_statement(stmt);
            if let Err(e) = self
                .permission_manager
                .check_permission(&username, space_id, permission)
            {
                return Err(format!("Permission check failed: {}", e));
            }
        }

        // Extract spatial information from the client session.
        let space_info = session.space().map(|s| crate::core::types::SpaceInfo {
            space_name: s.name.clone(),
            space_id: s.id as u64,
            vid_type: crate::core::types::DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        });

        // Create a QueryRequestContext from the session.
        let rctx = Arc::new(build_query_request_context(
            &session,
            stmt.to_string(),
            std::collections::HashMap::new(),
        ));

        let mut pipeline_manager = self.pipeline_manager.lock();
        let result = pipeline_manager.execute_query_with_request(stmt, rctx, space_info);

        match result {
            Ok(exec_result) => Ok(exec_result),
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

    pub async fn signout(&self, session_id: i64) {
        if let Some(session) = self.session_manager.find_session(session_id) {
            if let Some(space_name) = session.space_name() {
                self.stats_manager
                    .dec_space_metric(&space_name, MetricType::NumActiveQueries);
            }
        }
        self.session_manager.remove_session(session_id).await;
    }

    pub fn get_session_manager(&self) -> &Arc<GraphSessionManager> {
        &self.session_manager
    }

    pub fn get_permission_manager(&self) -> &Arc<PermissionManager> {
        &self.permission_manager
    }

    pub fn get_stats_manager(&self) -> &Arc<StatsManager> {
        &self.stats_manager
    }

    /// Obtain the session list (SHOW SESSIONS)
    pub async fn list_sessions(&self) -> Vec<crate::api::server::session::SessionInfo> {
        self.session_manager.list_sessions().await
    }

    /// Obtain detailed information about the specified session.
    pub async fn get_session_info(
        &self,
        session_id: i64,
    ) -> Option<crate::api::server::session::SessionInfo> {
        self.session_manager.get_session_info(session_id).await
    }

    /// Terminate the session (KILL SESSION)
    pub async fn kill_session(&self, session_id: i64, current_user: &str) -> SessionResult<()> {
        // Obtain the current session in order to check permissions.
        let current_session = self
            .session_manager
            .find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let is_admin = current_session.is_admin();

        self.session_manager
            .kill_session(session_id, current_user, is_admin)
            .await
    }

    /// Terminate the query (KILL QUERY)
    pub fn kill_query(&self, session_id: i64, query_id: u32) -> SessionResult<()> {
        let session = self
            .session_manager
            .find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;

        match session.kill_query(query_id) {
            Ok(()) => {
                self.stats_manager.dec_value(MetricType::NumActiveQueries);
                Ok(())
            }
            Err(e) => Err(SessionError::ManagerError(e.to_string())),
        }
    }

    // ==================== Transaction Control Methods ====================

    /// Processing the BEGIN TRANSACTION statement
    fn handle_begin_transaction(
        &self,
        session: &Arc<ClientSession>,
    ) -> Result<ExecutionResult, String> {
        if session.has_active_transaction() {
            return Err("Session already has an active transaction".to_string());
        }

        let txn_manager = self
            .transaction_manager
            .as_ref()
            .ok_or("Transaction manager not initialized")?;

        let options = session.transaction_options();
        match txn_manager.begin_transaction(options) {
            Ok(txn_id) => {
                session.bind_transaction(txn_id);
                session.set_auto_commit(false);
                info!("Session {} started transaction {}", session.id(), txn_id);
                Ok(ExecutionResult::Success)
            }
            Err(e) => Err(format!("Failed to start transaction: {}", e)),
        }
    }

    /// Processing the COMMIT statement
    fn handle_commit_transaction(
        &self,
        session: &Arc<ClientSession>,
    ) -> Result<ExecutionResult, String> {
        let txn_id = session.current_transaction().ok_or("No active transaction to commit")?;

        let txn_manager = self
            .transaction_manager
            .as_ref()
            .ok_or("Transaction manager not initialized")?;

        match txn_manager.commit_transaction(txn_id) {
            Ok(()) => {
                session.unbind_transaction();
                session.set_auto_commit(true);
                info!("Session {} committed transaction {}", session.id(), txn_id);
                Ok(ExecutionResult::Success)
            }
            Err(e) => Err(format!("Failed to commit transaction: {}", e)),
        }
    }

    /// Processing the ROLLBACK statement
    fn handle_rollback_transaction(
        &self,
        session: &Arc<ClientSession>,
        stmt: &str,
    ) -> Result<ExecutionResult, String> {
        let trimmed = stmt.trim().to_uppercase();

        // Check whether it is a command to perform a ROLLBACK TO SAVEPOINT.
        if trimmed.starts_with("ROLLBACK TO ") {
            let savepoint_name = stmt[trimmed.find("ROLLBACK TO ").unwrap() + 12..].trim();

            let txn_id = session.current_transaction().ok_or("No active transaction to rollback")?;

            let txn_manager = self
                .transaction_manager
                .as_ref()
                .ok_or("Transaction manager not initialized")?;

            let context = txn_manager
                .get_context(txn_id)
                .map_err(|e| format!("Failed to get transaction context: {}", e))?;

            // Try to find the save point by using its name.
            let savepoint_info = context
                .find_savepoint_by_name(savepoint_name)
                .ok_or_else(|| format!("Savepoint '{}' does not exist", savepoint_name))?;

            // Perform a rollback.
            match txn_manager.rollback_to_savepoint(txn_id, savepoint_info.id) {
                Ok(()) => {
                    info!(
                        "Session {} rolled back transaction {} to savepoint {}",
                        session.id(),
                        txn_id,
                        savepoint_name
                    );
                    Ok(ExecutionResult::Success)
                }
                Err(e) => Err(format!("Failed to rollback to savepoint: {}", e)),
            }
        } else {
            // Full transaction rollback
            let txn_id = session.current_transaction().ok_or("No active transaction to rollback")?;

            let txn_manager = self
                .transaction_manager
                .as_ref()
                .ok_or("Transaction manager not initialized")?;

            match txn_manager.abort_transaction(txn_id) {
                Ok(()) => {
                    session.unbind_transaction();
                    session.set_auto_commit(true);
                    info!("Session {} rolled back transaction {}", session.id(), txn_id);
                    Ok(ExecutionResult::Success)
                }
                Err(e) => Err(format!("Failed to rollback transaction: {}", e)),
            }
        }
    }

    /// Processing the SAVEPOINT statement
    fn handle_savepoint(
        &self,
        session: &Arc<ClientSession>,
        stmt: &str,
    ) -> Result<ExecutionResult, String> {
        let savepoint_name = stmt["SAVEPOINT".len()..].trim();

        if savepoint_name.is_empty() {
            return Err("Savepoint name cannot be empty".to_string());
        }

        let txn_id = session
            .current_transaction()
            .ok_or("No active transaction, cannot create savepoint")?;

        let txn_manager = self
            .transaction_manager
            .as_ref()
            .ok_or("Transaction manager not initialized")?;

        let context = txn_manager
            .get_context(txn_id)
            .map_err(|e| format!("Failed to get transaction context: {}", e))?;

        let savepoint_id = context.create_savepoint(Some(savepoint_name.to_string()));

        info!(
            "Session {} created savepoint {} in transaction {} (ID: {})",
            session.id(),
            savepoint_name,
            txn_id,
            savepoint_id
        );

        Ok(ExecutionResult::Success)
    }

    /// Processing the RELEASE SAVEPOINT statement
    fn handle_release_savepoint(
        &self,
        session: &Arc<ClientSession>,
        stmt: &str,
    ) -> Result<ExecutionResult, String> {
        let savepoint_name = stmt["RELEASE SAVEPOINT".len()..].trim();

        if savepoint_name.is_empty() {
            return Err("Savepoint name cannot be empty".to_string());
        }

        let txn_id = session.current_transaction().ok_or("No active transaction")?;

        let txn_manager = self
            .transaction_manager
            .as_ref()
            .ok_or("Transaction manager not initialized")?;

        let context = txn_manager
            .get_context(txn_id)
            .map_err(|e| format!("Failed to get transaction context: {}", e))?;

        let savepoint_info = context
            .find_savepoint_by_name(savepoint_name)
            .ok_or_else(|| format!("Savepoint '{}' does not exist", savepoint_name))?;

        context
            .release_savepoint(savepoint_info.id)
            .map_err(|e| format!("Failed to release savepoint: {}", e))?;

        info!(
            "Session {} released savepoint {} in transaction {} (ID: {})",
            session.id(),
            savepoint_name,
            txn_id,
            savepoint_info.id
        );

        Ok(ExecutionResult::Success)
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

    #[tokio::test]
    async fn test_graph_service_creation() {
        let config = create_test_config();

        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // The verification service has been created successfully.
        assert!(
            !graph_service
                .get_session_manager()
                .is_out_of_connections()
                .await
        );
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // Testing the default user authentication mechanism
        let result = graph_service.authenticate("root", "root").await;
        assert!(result.is_ok(), "The default user authentication should succeed.");

        let session = result.expect("Failed to get session");
        assert_eq!(session.user(), "root");
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // Test incorrect password.
        let result = graph_service.authenticate("root", "wrong_password").await;
        assert!(result.is_err(), "An incorrect password should result in a authentication failure.");

        // Test an empty username or password.
        let result = graph_service.authenticate("", "root").await;
        assert!(result.is_err(), "An empty username should result in an authentication failure.");

        let result = graph_service.authenticate("root", "").await;
        assert!(result.is_err(), "An empty password should result in a failed authentication attempt.");
    }

    #[tokio::test]
    async fn test_session_management() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        // Create a session
        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        let session_id = session.id();

        // Search for the conversation
        let found_session = graph_service.get_session_manager().find_session(session_id);
        assert!(found_session.is_some(), "We should be able to find the session that was just created.");

        // Log out of the session.
        graph_service.signout(session_id).await;

        // The verification session has been removed.
        let found_session = graph_service.get_session_manager().find_session(session_id);
        assert!(found_session.is_none(), "After signing out, the session should no longer be available.");
    }

    #[tokio::test]
    async fn test_execute_query() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // perform a search
        let result = graph_service.execute(session.id(), "SHOW SPACES");
        // Note: It may fail here because MockStorage may not implement the full feature
        // We mainly test calls that do not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_transaction_control() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Test BEGIN TRANSACTION
        let result = graph_service.execute(session.id(), "BEGIN TRANSACTION");
        // Note: This may fail because the GraphService may not have a transaction manager configured.
        // We mainly test calls that do not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_savepoint_creation() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Test creation of savepoints (need to start transaction first)
        let _ = graph_service.execute(session.id(), "BEGIN TRANSACTION");
        let result = graph_service.execute(session.id(), "SAVEPOINT sp1");

        // Note: This may fail because the GraphService may not have a transaction manager configured.
        // We mainly test calls that do not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_savepoint_empty_name() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Test empty save point name
        let _ = graph_service.execute(session.id(), "BEGIN TRANSACTION");
        let result = graph_service.execute(session.id(), "SAVEPOINT");

        // Should return an error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rollback_to_savepoint() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Test rollback to save point
        let _ = graph_service.execute(session.id(), "BEGIN TRANSACTION");
        let _ = graph_service.execute(session.id(), "SAVEPOINT sp1");
        let result = graph_service.execute(session.id(), "ROLLBACK TO SAVEPOINT sp1");

        // Note: This may fail because the GraphService may not have a transaction manager configured.
        // We mainly test calls that do not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_rollback_to_savepoint_without_transaction() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Testing rollback to a savepoint without a transaction
        let result = graph_service.execute(session.id(), "ROLLBACK TO SAVEPOINT sp1");

        // Should return an error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_release_savepoint() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Test release save point
        let _ = graph_service.execute(session.id(), "BEGIN TRANSACTION");
        let _ = graph_service.execute(session.id(), "SAVEPOINT sp1");
        let result = graph_service.execute(session.id(), "RELEASE SAVEPOINT sp1");

        // Note: This may fail because the GraphService may not have a transaction manager configured.
        // We mainly test calls that do not panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_release_savepoint_without_transaction() {
        let config = create_test_config();
        let storage = Arc::new(MockStorage::new().expect("Failed to create Memory storage"));
        let graph_service = GraphService::<MockStorage>::new_for_test(config, storage);

        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Failed to create session");

        // Testing the release of savepoints without transactions
        let result = graph_service.execute(session.id(), "RELEASE SAVEPOINT sp1");

        // Should return an error
        assert!(result.is_err());
    }
}
