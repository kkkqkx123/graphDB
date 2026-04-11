//! Session Management Module
//!
//! Provide the concept of a "session" as the context in which queries are executed.

use crate::api::core::{CoreError, CoreResult, QueryApi, QueryRequest, SchemaApi};
use crate::api::embedded::batch::BatchInserter;
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::transaction::{Transaction, TransactionConfig};
use crate::coordinator::FulltextCoordinator;
use crate::core::SessionStatistics;
use crate::core::Value;
use crate::query::executor::expression::functions::{CustomFunction, FunctionRegistry};
use crate::search::FulltextIndexManager;
use crate::storage::StorageClient;
use crate::sync::SyncManager;
use crate::transaction::TransactionManager;
use crate::transaction::TransactionOptions;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Session – Execution Context
///
/// A session is the basic unit for the execution of queries, and it maintains contextual information such as the current graph space and the transaction status.
///
/// # Examples
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
/// let mut session = db.session()?;
///
// Switch to the image space
/// session.use_space("test_space")?;
///
// Execute the query
/// let result = session.execute("MATCH (n) RETURN n")?;
///
// Using a transaction
/// let txn = session.begin_transaction()?;
/// txn.execute("CREATE TAG user(name string)")?;
/// txn.commit()?;
/// # Ok(())
/// # }
/// ```
pub struct Session<S: StorageClient + Clone + 'static> {
    db: Arc<GraphDatabaseInner<S>>,
    space_id: Option<u64>,
    space_name: Option<String>,
    auto_commit: bool,
    /// Session-level change statistics
    statistics: SessionStatistics,
    /// Session-level function registry
    function_registry: Arc<Mutex<FunctionRegistry>>,
}

/// Internal structure of the database, used for sharing data between Session and GraphDatabase
#[repr(C)]
pub(crate) struct GraphDatabaseInner<S: StorageClient + Clone + 'static> {
    pub(crate) query_api: Arc<Mutex<QueryApi<S>>>,
    pub(crate) schema_api: SchemaApi<S>,
    pub(crate) txn_manager: Arc<TransactionManager>,
    pub(crate) storage: Arc<Mutex<S>>,
    pub(crate) fulltext_manager: Option<Arc<FulltextIndexManager>>,
    pub(crate) fulltext_coordinator: Option<Arc<FulltextCoordinator>>,
    pub(crate) sync_manager: Option<Arc<SyncManager>>,
}

impl<S: StorageClient + Clone + 'static> Session<S> {
    /// Create a new session.
    pub(crate) fn new(db: Arc<GraphDatabaseInner<S>>) -> Self {
        Self {
            db,
            space_id: None,
            space_name: None,
            auto_commit: true,
            statistics: SessionStatistics::new(),
            function_registry: Arc::new(Mutex::new(FunctionRegistry::new())),
        }
    }

    /// Register a custom function
    pub fn register_custom_function(&self, function: CustomFunction) -> CoreResult<()> {
        let mut registry = self.function_registry.lock();
        registry.register_custom_full(function);
        Ok(())
    }

    /// Obtain a reference to the function registry.
    pub fn function_registry(&self) -> Arc<Mutex<FunctionRegistry>> {
        Arc::clone(&self.function_registry)
    }

    /// Get the number of rows affected by the last operation.
    pub fn changes(&self) -> u64 {
        self.statistics.last_changes()
    }

    /// Obtain the total number of session changes
    pub fn total_changes(&self) -> u64 {
        self.statistics.total_changes()
    }

    /// Obtain the ID of the last vertex that was inserted.
    pub fn last_insert_vertex_id(&self) -> Option<i64> {
        self.statistics.last_insert_vertex_id()
    }

    /// Obtain the ID of the last inserted edge.
    pub fn last_insert_edge_id(&self) -> Option<i64> {
        self.statistics.last_insert_edge_id()
    }

    /// Obtain statistical information references
    pub fn statistics(&self) -> &SessionStatistics {
        &self.statistics
    }

    /// Switch to the image space
    ///
    /// # Parameters
    /// `space_name` – Name of the graph space
    ///
    /// # Back
    /// - Returns on success ()
    /// - Return an error when something goes wrong (for example, if the required space does not exist).
    pub fn use_space(&mut self, space_name: &str) -> CoreResult<()> {
        let space_id = self.db.schema_api.use_space(space_name)?;
        self.space_id = Some(space_id);
        self.space_name = Some(space_name.to_string());
        Ok(())
    }

    /// Obtain the name of the current image space.
    pub fn current_space(&self) -> Option<&str> {
        self.space_name.as_deref()
    }

    /// Obtain the current image space ID.
    pub fn current_space_id(&self) -> Option<u64> {
        self.space_id
    }

    /// Enable the automatic submission mode.
    ///
    /// When `auto_commit` is set to `true`, each query is automatically committed.
    /// When `auto_commit` is set to `false`, transactions must be explicitly used.
    pub fn set_auto_commit(&mut self, auto_commit: bool) {
        self.auto_commit = auto_commit;
    }

    /// Enable the automatic submission mode.
    pub fn auto_commit(&self) -> bool {
        self.auto_commit
    }

    /// Execute the query statement.
    ///
    /// # Parameters
    /// `query` – A string representing the query statement.
    ///
    /// # Back
    /// Return the query results when successful.
    /// - Return error on failure
    pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
        // Reset the previous change history
        self.statistics.reset_last();

        let ctx = QueryRequest {
            space_id: self.space_id,
            auto_commit: self.auto_commit,
            transaction_id: None,
            parameters: None,
        };

        let mut query_api = self.db.query_api.lock();
        let result = query_api.execute(query, ctx)?;

        // Update statistical information
        self.statistics
            .record_changes(result.metadata.rows_returned);

        Ok(QueryResult::from_core(result))
    }

    /// Execute a parameterized query
    ///
    /// # Parameters
    /// - `query` - query statement string
    /// - `params` – Query parameters
    ///
    /// # Return
    /// - Returns query results on success
    /// - Return error on failure
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>,
    ) -> CoreResult<QueryResult> {
        let ctx = QueryRequest {
            space_id: self.space_id,
            auto_commit: self.auto_commit,
            transaction_id: None,
            parameters: Some(params),
        };

        let mut query_api = self.db.query_api.lock();
        let result = query_api.execute(query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// Start a transaction
    ///
    /// # Return
    /// - Returns the transaction handle on success
    /// - Return error on failure
    pub fn begin_transaction(&self) -> CoreResult<Transaction<'_, S>> {
        let options = TransactionOptions::default();
        let txn_id = self
            .db
            .txn_manager
            .begin_transaction(options)
            .map_err(|e| crate::api::core::CoreError::TransactionFailed(e.to_string()))?;
        let txn_handle = crate::api::core::TransactionHandle(txn_id);

        Ok(Transaction::new(self, txn_handle))
    }

    /// Starting a Transaction with Configuration
    ///
    /// # Parameters
    /// - `config` - transaction configuration options
    ///
    /// # Return
    /// - Returns the transaction handle on success
    /// - Return error on failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graphdb::api::embedded::{GraphDatabase, TransactionConfig};
    /// use std::time::Duration;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let session = db.session()?;
    ///
    // Create read-only transactions
    /// let config = TransactionConfig::new()
    ///     .read_only()
    ///     .with_timeout(Duration::from_secs(60));
    ///
    /// let txn = session.begin_transaction_with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_transaction_with_config(
        &self,
        config: TransactionConfig,
    ) -> CoreResult<Transaction<'_, S>> {
        let options = config.into_options();
        let txn_id = self
            .db
            .txn_manager
            .begin_transaction(options)
            .map_err(|e| crate::api::core::CoreError::TransactionFailed(e.to_string()))?;
        let txn_handle = crate::api::core::TransactionHandle(txn_id);

        Ok(Transaction::new(self, txn_handle))
    }

    /// Performing operations in a transaction (autocommit/rollback)
    ///
    /// # Parameters
    /// - `f` - closure executed in a transaction
    ///
    /// # Return
    /// - Returns the closure's return value on success
    /// - Return error on failure
    pub async fn with_transaction<F, T>(&self, f: F) -> CoreResult<T>
    where
        F: FnOnce(&Transaction<'_, S>) -> CoreResult<T>,
    {
        let txn = self.begin_transaction()?;

        match f(&txn) {
            Ok(result) => {
                txn.commit().await?;
                Ok(result)
            }
            Err(e) => {
                let _ = txn.rollback().await;
                Err(e)
            }
        }
    }

    /// Creating a graph space
    ///
    /// # Parameters
    /// - `name' - space name
    /// - `config' - space configuration
    ///
    /// # Return
    /// - Returns on success ()
    /// - Return error on failure
    pub fn create_space(
        &self,
        name: &str,
        config: crate::api::core::SpaceConfig,
    ) -> CoreResult<()> {
        self.db.schema_api.create_space(name, config)
    }

    /// Deletion of map space
    ///
    /// # Parameters
    /// - `name' - space name
    ///
    /// # Return
    /// - Returns on success ()
    /// - Return error on failure
    pub fn drop_space(&self, name: &str) -> CoreResult<()> {
        self.db.schema_api.drop_space(name)
    }

    /// List all graph spaces
    pub fn list_spaces(&self) -> CoreResult<Vec<String>> {
        // Getting all the space through the storage layer
        let storage = self.db.storage.lock();
        let spaces = storage
            .list_spaces()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        Ok(spaces.into_iter().map(|s| s.space_name).collect())
    }

    /// Getting a lock on the query API (internal use)
    pub(crate) fn query_api(&self) -> parking_lot::MutexGuard<'_, QueryApi<S>> {
        self.db.query_api.as_ref().lock()
    }

    /// Get space ID (internal use)
    pub(crate) fn space_id(&self) -> Option<u64> {
        self.space_id
    }

    /// Getting the transaction manager (internal use)
    pub(crate) fn txn_manager(&self) -> Arc<TransactionManager> {
        self.db.txn_manager.clone()
    }

    /// Acquiring stored locks (for internal use)
    pub(crate) fn storage(&self) -> parking_lot::MutexGuard<'_, S> {
        self.db.storage.lock()
    }

    /// Get current space name (for internal use)
    pub(crate) fn space_name(&self) -> Option<&str> {
        self.space_name.as_deref()
    }

    /// Creating a Batch Inserter
    ///
    /// # Parameters
    /// - `batch_size` - batch size, automatically refreshes when this amount is reached
    ///
    /// # Return
    /// - Returns an instance of BatchInserter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    /// use graphdb::core::{Vertex, Value};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let session = db.session()?;
    ///
    // Create a batch inserter that automatically refreshes every 100 entries
    /// let mut inserter = session.batch_inserter(100);
    ///
    // Add vertices
    /// for i in 0..1000 {
    ///     let vertex = Vertex::with_vid(Value::Int(i));
    ///     inserter.add_vertex(vertex);
    /// }
    ///
    // Perform batch insertion
    /// let result = inserter.execute()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter<'_, S> {
        BatchInserter::new(self, batch_size)
    }

    /// Batch insert vertices
    ///
    /// # Parameters
    /// - `vertices` - list of vertices to insert
    ///
    /// # Return
    /// - Returns the number of vertices inserted on success
    /// - Return error on failure
    pub fn batch_insert_vertices(&self, vertices: Vec<crate::core::Vertex>) -> CoreResult<usize> {
        let space_name = self
            .space_name()
            .ok_or_else(|| CoreError::InvalidParameter("No graph space selected".to_string()))?;

        let count = vertices.len();
        let mut storage = self.storage();
        storage
            .batch_insert_vertices(space_name, vertices)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        Ok(count)
    }

    /// Batch insert edges
    ///
    /// # Parameters
    /// - `edges` - list of edges to insert
    ///
    /// # Return
    /// - Returns the number of edges inserted on success
    /// - Return error on failure
    pub fn batch_insert_edges(&self, edges: Vec<crate::core::Edge>) -> CoreResult<usize> {
        let space_name = self
            .space_name()
            .ok_or_else(|| CoreError::InvalidParameter("No graph space selected".to_string()))?;

        let count = edges.len();
        let mut storage = self.storage();
        storage
            .batch_insert_edges(space_name, edges)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        Ok(count)
    }

    /// Commit a transaction by handle (for C API use)
    ///
    /// # Parameters
    /// - `txn_handle` - transaction handle
    ///
    /// # Return
    /// - Returns () on success
    /// - Return error on failure
    pub async fn commit_transaction(
        &self,
        txn_handle: crate::api::core::TransactionHandle,
    ) -> CoreResult<()> {
        self.txn_manager()
            .commit_transaction(txn_handle.0)
            .await
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// Rollback a transaction by handle (for C API use)
    ///
    /// # Parameters
    /// - `txn_handle` - transaction handle
    ///
    /// # Return
    /// - Returns () on success
    /// - Return error on failure
    pub fn rollback_transaction(
        &self,
        txn_handle: crate::api::core::TransactionHandle,
    ) -> CoreResult<()> {
        self.txn_manager()
            .abort_transaction(txn_handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// Create a savepoint for a transaction (for C API use)
    ///
    /// # Parameters
    /// - `txn_handle` - transaction handle
    /// - `name` - savepoint name
    ///
    /// # Return
    /// - Returns savepoint ID on success
    /// - Return error on failure
    pub fn create_savepoint(
        &self,
        txn_handle: &crate::api::core::TransactionHandle,
        name: &str,
    ) -> CoreResult<crate::api::core::SavepointId> {
        self.txn_manager()
            .create_savepoint(txn_handle.0, Some(name.to_string()))
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
            .map(crate::api::core::SavepointId)
    }

    /// Release a savepoint (for C API use)
    ///
    /// # Parameters
    /// - `txn_handle` - transaction handle
    /// - `savepoint` - savepoint ID
    ///
    /// # Return
    /// - Returns () on success
    /// - Return error on failure
    pub fn release_savepoint(
        &self,
        txn_handle: &crate::api::core::TransactionHandle,
        savepoint: crate::api::core::SavepointId,
    ) -> CoreResult<()> {
        self.txn_manager()
            .release_savepoint(txn_handle.0, savepoint.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// Rollback to a savepoint (for C API use)
    ///
    /// # Parameters
    /// - `txn_handle` - transaction handle
    /// - `savepoint` - savepoint ID
    ///
    /// # Return
    /// - Returns () on success
    /// - Return error on failure
    pub fn rollback_to_savepoint(
        &self,
        txn_handle: &crate::api::core::TransactionHandle,
        savepoint: crate::api::core::SavepointId,
    ) -> CoreResult<()> {
        self.txn_manager()
            .rollback_to_savepoint(txn_handle.0, savepoint.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
}

impl<S: StorageClient + Clone + 'static> Drop for Session<S> {
    fn drop(&mut self) {
        // No special cleanup is required when the session is discarded.
        // Because all transactions are managed through the Transaction object, and Transactions have their own Drop implementation
        // Just logging here for debugging purposes
        log::debug!(
            "Session released, current graph space: {:?}",
            self.space_name
        );
    }
}

// In order to support Send + Sync, we need to ensure that S satisfies these constraints
// Safety Notes:
// 1. Session uses Arc<GraphDatabaseInner<S>> to share data internally, Arc itself is Send + Sync.
// 2. QueryApi in GraphDatabaseInner is Mutex-protected for thread-safety.
// 3. The StorageClient class must implement the Clone method and be marked as ‘static’. This is to ensure that objects can be safely passed between different threads.
// 4. All internal states (space_id, space_name, auto_commit) are of simple, replicable types.
// Therefore, the Session can securely implement both the Send and Sync functions.
unsafe impl<S: StorageClient + Clone + 'static> Send for Session<S> {}
unsafe impl<S: StorageClient + Clone + 'static> Sync for Session<S> {}
