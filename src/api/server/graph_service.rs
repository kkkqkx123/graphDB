use crate::api::core::{QueryApi, SyncApi, VectorApi};
use crate::api::server::auth::{Authenticator, AuthenticatorFactory, PasswordAuthenticator};
use crate::api::server::permission::PermissionManager;
use crate::api::server::session::{ClientSession, GraphSessionManager, SpaceInfo};
use crate::config::Config;
use crate::core::error::{SessionError, SessionResult};
use crate::core::stats::StatsManager;
use crate::core::{MetricType, Permission};
use crate::query::executor::ExecutionResult;
use crate::query::DataSet;
use crate::storage::engine::redb_storage::RedbStorage;
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use log::{info, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use vector_client::VectorManager;

pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_api: Arc<Mutex<QueryApi<S>>>,
    authenticator: PasswordAuthenticator,
    permission_manager: Arc<PermissionManager>,
    pub stats_manager: Arc<StatsManager>,
    storage: Arc<S>,
    vector_api: Option<Arc<VectorApi>>,
    sync_api: Option<Arc<SyncApi>>,

    // Transaction management-related
    transaction_manager: Option<Arc<TransactionManager>>,
}

impl<S: StorageClient + Clone + 'static> GraphService<S> {
    /// Create a new GraphService (without a transaction manager, for use in a production environment).
    pub async fn new(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, true).await
    }

    /// Create a new GraphService (without a transaction manager and without starting any background tasks, for testing purposes).
    pub async fn new_for_test(config: Config, storage: Arc<S>) -> Arc<Self> {
        Self::create_service(config, storage, None, false).await
    }

    /// Use the transaction manager to create a GraphService.
    pub async fn new_with_transaction_manager(
        config: Config,
        storage: Arc<S>,
        transaction_manager: Arc<TransactionManager>,
    ) -> Arc<Self> {
        Self::create_service(config, storage, Some(transaction_manager), true).await
    }

    /// Internal constructor: Extracts the common logic
    ///
    /// # Parameters
    /// `start_cleanup_task` – Whether to initiate the background task for session cleanup
    async fn create_service(
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
            session_manager.start_cleanup_task().await;
        }

        // Use core layer QueryApi instead of directly using QueryPipelineManager
        // Support vector search with metadata provider if enabled
        // Try to get schema_manager from storage if it's RedbStorage
        let schema_manager = storage
            .as_any()
            .downcast_ref::<RedbStorage>()
            .map(|redb_storage| redb_storage.state().schema_manager.clone());

        let (query_api, vector_api) = if config.vector.enabled {
            match QueryApi::with_vector_search(
                Arc::new(Mutex::new((*storage).clone())),
                config.vector.clone(),
                schema_manager.clone(),
            )
            .await
            {
                Ok(api) => {
                    // Create vector manager and vector API
                    let vector_manager = Arc::new(
                        VectorManager::new(config.vector.clone())
                            .await
                            .unwrap_or_else(|_| panic!("Failed to create vector manager")),
                    );
                    let vector_api = Arc::new(VectorApi::new(vector_manager.clone()));
                    (Arc::new(Mutex::new(api)), Some(vector_api))
                }
                Err(e) => {
                    warn!(
                        "Failed to initialize vector search, falling back to basic QueryApi: {}",
                        e
                    );
                    let api = if let Some(sm) = schema_manager.clone() {
                        QueryApi::with_schema_manager(Arc::new(Mutex::new((*storage).clone())), sm)
                    } else {
                        QueryApi::new(Arc::new(Mutex::new((*storage).clone())))
                    };
                    (Arc::new(Mutex::new(api)), None)
                }
            }
        } else {
            let api = if let Some(sm) = schema_manager {
                QueryApi::with_schema_manager(Arc::new(Mutex::new((*storage).clone())), sm)
            } else {
                QueryApi::new(Arc::new(Mutex::new((*storage).clone())))
            };
            (Arc::new(Mutex::new(api)), None)
        };

        let authenticator = AuthenticatorFactory::create_default(&config.server.auth);
        let permission_manager = Arc::new(PermissionManager::new());

        // Create StatsManager with slow query logger
        let slow_query_config = config.to_slow_query_config();

        let server_stats_manager = Arc::new(
            StatsManager::with_slow_query_logger(config.monitoring.clone(), slow_query_config)
                .expect("Failed to create StatsManager with slow query logger"),
        );

        // Create sync API if storage supports it
        let sync_api = storage
            .get_sync_manager()
            .map(|sync_manager| Arc::new(SyncApi::new(sync_manager)));

        Arc::new(Self {
            session_manager,
            query_api,
            authenticator,
            permission_manager,
            stats_manager: server_stats_manager,
            storage,
            vector_api,
            sync_api,
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

        // Perform a regular query using core layer QueryApi
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
                    // Use block_on to execute async commit_transaction in sync context
                    let rt = tokio::runtime::Handle::current();
                    if let Err(e) = rt.block_on(txn_manager.commit_transaction(txn_id)) {
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

        // Use core layer QueryApi to execute query
        let query_request = crate::api::core::QueryRequest {
            space_id: session.space().map(|s| s.id as u64),
            space_name: session.space().map(|s| s.name),
            auto_commit: session.is_auto_commit(),
            transaction_id: session.current_transaction(),
            parameters: None,
        };

        let mut query_api = self.query_api.lock();
        let result = query_api.execute(stmt, query_request);

        match result {
            Ok(query_result) => Ok(Self::convert_to_execution_result(query_result)),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Convert core QueryResult to query ExecutionResult
    fn convert_to_execution_result(result: crate::api::core::QueryResult) -> ExecutionResult {
        if result.rows.is_empty() {
            return ExecutionResult::Empty;
        }

        // General case: return DataSet
        let rows: Vec<Vec<crate::core::Value>> = result
            .rows
            .into_iter()
            .map(|row| {
                result
                    .columns
                    .iter()
                    .filter_map(|col| row.get(col).cloned())
                    .collect()
            })
            .collect();

        ExecutionResult::DataSet(DataSet {
            col_names: result.columns,
            rows,
        })
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

    pub fn vector_api(&self) -> Option<&Arc<VectorApi>> {
        self.vector_api.as_ref()
    }

    pub fn sync_api(&self) -> Option<&Arc<SyncApi>> {
        self.sync_api.as_ref()
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
        let txn_id = session
            .current_transaction()
            .ok_or("No active transaction to commit")?;

        let txn_manager = self
            .transaction_manager
            .as_ref()
            .ok_or("Transaction manager not initialized")?;

        // Use block_on to execute async commit_transaction in sync context
        let rt = tokio::runtime::Handle::current();
        match rt.block_on(txn_manager.commit_transaction(txn_id)) {
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
            let savepoint_name = trimmed
                .strip_prefix("ROLLBACK TO ")
                .map(|s| s.trim())
                .ok_or("Invalid ROLLBACK TO syntax")?;

            let txn_id = session
                .current_transaction()
                .ok_or("No active transaction to rollback")?;

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
            let txn_id = session
                .current_transaction()
                .ok_or("No active transaction to rollback")?;

            let txn_manager = self
                .transaction_manager
                .as_ref()
                .ok_or("Transaction manager not initialized")?;

            match txn_manager.rollback_transaction(txn_id) {
                Ok(()) => {
                    session.unbind_transaction();
                    session.set_auto_commit(true);
                    info!(
                        "Session {} rolled back transaction {}",
                        session.id(),
                        txn_id
                    );
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

        let txn_id = session
            .current_transaction()
            .ok_or("No active transaction, cannot release savepoint")?;

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

        // Release the savepoint.
        if let Err(e) = context.release_savepoint(savepoint_info.id) {
            return Err(format!("Failed to release savepoint: {}", e));
        }

        info!(
            "Session {} released savepoint {} in transaction {}",
            session.id(),
            savepoint_name,
            txn_id
        );

        Ok(ExecutionResult::Success)
    }
}
