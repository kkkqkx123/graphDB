//! Database main module
//!
//! Provide the GraphDatabase structure as the main entry point for the embedded API.

use crate::api::core::{CoreError, CoreResult, QueryApi, SchemaApi, SpaceConfig};
use crate::api::embedded::config::DatabaseConfig;
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::session::{GraphDatabaseInner, Session};
use crate::core::Value;
use crate::storage::{RedbStorage, StorageClient};
use crate::transaction::{TransactionManager, TransactionManagerConfig};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
use crate::storage::test_mock::MockStorage;

/// Embedded GraphDB database
///
/// This is the main entry point for the embedded API, offering a simple way of use similar to that of SQLite.
/// The sqlite3 structure corresponding to SQLite.
///
/// # Example
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
// Open the database
/// let db = GraphDatabase::open("my_database")?;
///
// Create a session
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
///
/// The database is automatically closed when the `db` variable goes out of scope.
/// ```
pub struct GraphDatabase<S: StorageClient + Clone + 'static> {
    inner: Arc<GraphDatabaseInner<S>>,
    config: DatabaseConfig,
}

impl GraphDatabase<RedbStorage> {
    /// Open or create a database (in file mode).
    ///
    /// # Parameters
    /// `path` – The path to the database file
    ///
    /// # Back
    /// Return the GraphDatabase instance upon successful completion.
    /// - Return error on failure
    pub fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let config = DatabaseConfig::file(path);
        Self::open_with_config(config)
    }

    /// Create a memory database
    ///
    /// # 返回
    /// - 成功时返回 GraphDatabase 实例
    /// - 失败时返回错误
    pub fn open_in_memory() -> CoreResult<Self> {
        let config = DatabaseConfig::memory();
        Self::open_with_config(config)
    }

    /// Open the database using the configuration settings.
    ///
    /// # 参数
    /// `config` – Database configuration
    ///
    /// # 返回
    /// - 成功时返回 GraphDatabase 实例
    /// - 失败时返回错误
    pub fn open_with_config(config: DatabaseConfig) -> CoreResult<Self> {
        let storage = if config.is_memory() {
            RedbStorage::new()
                .map_err(|e| CoreError::StorageError(format!("初始化内存存储失败: {}", e)))?
        } else {
            let path = config
                .path()
                .ok_or_else(|| CoreError::StorageError("数据库路径为空".to_string()))?;
            RedbStorage::new_with_path(path.to_path_buf())
                .map_err(|e| CoreError::StorageError(format!("初始化存储失败: {}", e)))?
        };

        let storage = Arc::new(Mutex::new(storage));
        let db = storage.lock().get_db().clone();

        let txn_manager_config = TransactionManagerConfig::default();
        let txn_manager = Arc::new(TransactionManager::new(db, txn_manager_config));

        let query_api = Arc::new(Mutex::new(QueryApi::new(storage.clone())));
        let schema_api = SchemaApi::new(storage.clone());

        let inner = Arc::new(GraphDatabaseInner {
            query_api,
            schema_api,
            txn_manager,
            storage,
        });

        Ok(Self { inner, config })
    }
}

impl<S: StorageClient + Clone + 'static> GraphDatabase<S> {
    /// Create a new session.
    ///
    /// # 返回
    /// Return the Session instance upon successful completion.
    /// - 失败时返回错误
    pub fn session(&self) -> CoreResult<Session<S>> {
        Ok(Session::new(self.inner.clone()))
    }

    /// Perform simple queries (a convenient method)
    ///
    /// This method creates a temporary session to execute the query, which is suitable for simple, one-time query scenarios.
    /// 对于复杂场景，建议使用 session() 创建会话。
    ///
    /// # 参数
    /// `query` – A string representing the query statement.
    ///
    /// # 返回
    /// Return the query results when successful.
    /// - 失败时返回错误
    pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
        let session = self.session()?;
        session.execute(query)
    }

    /// Executing parameterized queries (a convenient method)
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    /// - `params` – Query parameters
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>,
    ) -> CoreResult<QueryResult> {
        let session = self.session()?;
        session.execute_with_params(query, params)
    }

    /// Creating a graphical space (an easy method)
    ///
    /// # 参数
    /// - `name' - space name
    /// `config` – Space configuration
    ///
    /// # 返回
    /// - Returns on success ()
    /// - 失败时返回错误
    pub fn create_space(&self, name: &str, space_config: SpaceConfig) -> CoreResult<()> {
        let session = self.session()?;
        session.create_space(name, space_config)
    }

    /// Deletion of map space (convenient method)
    ///
    /// # 参数
    /// - `name` - 空间名称
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn drop_space(&self, name: &str) -> CoreResult<()> {
        let session = self.session()?;
        session.drop_space(name)
    }

    /// List all graph spaces (convenience method)
    pub fn list_spaces(&self) -> CoreResult<Vec<String>> {
        let session = self.session()?;
        session.list_spaces()
    }

    /// Get Configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Checking for in-memory databases
    pub fn is_memory(&self) -> bool {
        self.config.is_memory()
    }

    /// Getting a reference to the storage client
    ///
    /// # 返回
    /// - MutexGuard for Storage Clients
    pub fn storage(&self) -> parking_lot::MutexGuard<'_, S> {
        self.inner.storage.lock()
    }
}

// To support Send + Sync
// Safety Notes:
// 1. GraphDatabase uses Arc<GraphDatabaseInner<S>> to share data internally, Arc itself is Send + Sync.
// 2. QueryApi in GraphDatabaseInner is Mutex-protected for thread-safety.
// 3. StorageClient is required to implement Clone + 'static to ensure safe cross-thread delivery.
// 4. TransactionManager uses Arc wrappers, which can be safely shared across threads.
// 5. config is a standalone DatabaseConfig, safe to pass across threads.
// GraphDatabase can therefore securely implement Send and Sync.
unsafe impl<S: StorageClient + Clone + 'static> Send for GraphDatabase<S> {}
unsafe impl<S: StorageClient + Clone + 'static> Sync for GraphDatabase<S> {}

#[cfg(test)]
impl GraphDatabase<MockStorage> {
    /// Create database for testing (using Mock storage)
    ///
    /// 注意：此方法仅用于测试，实际使用时应使用 `GraphDatabase::open()`
    #[cfg(test)]
    pub fn open_test() -> CoreResult<Self> {
        let storage = MockStorage::new()
            .map_err(|e| CoreError::StorageError(format!("初始化Mock存储失败: {}", e)))?;

        let storage = Arc::new(Mutex::new(storage));
        let db = storage.lock().get_db().clone();

        let txn_manager_config = TransactionManagerConfig::default();
        let txn_manager = Arc::new(TransactionManager::new(db, txn_manager_config));

        let query_api = Arc::new(Mutex::new(QueryApi::new(storage.clone())));
        let schema_api = SchemaApi::new(storage.clone());

        let inner = Arc::new(GraphDatabaseInner {
            query_api,
            schema_api,
            txn_manager,
            storage,
        });

        Ok(Self {
            inner,
            config: DatabaseConfig::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::memory();
        assert!(config.is_memory());

        let config = DatabaseConfig::file("/tmp/test.db");
        assert!(!config.is_memory());
    }
}
