//! 数据库主模块
//!
//! 提供 GraphDatabase 结构体，作为嵌入式 API 的主要入口点

use crate::api::core::{CoreError, CoreResult, QueryApi, SchemaApi, SpaceConfig, TransactionApi};
use crate::api::embedded::config::DatabaseConfig;
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::session::{GraphDatabaseInner, Session};
use crate::core::Value;
use crate::storage::{RedbStorage, StorageClient};
use crate::transaction::{TransactionManager, TransactionManagerConfig, SavepointManager};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// 嵌入式 GraphDB 数据库
///
/// 这是嵌入式 API 的主要入口点，提供类似 SQLite 的简单使用方式。
/// 对应 SQLite 的 sqlite3 结构体。
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // 打开数据库
/// let db = GraphDatabase::open("my_database")?;
///
/// // 创建会话
/// let mut session = db.session()?;
///
/// // 切换图空间
/// session.use_space("test_space")?;
///
/// // 执行查询
/// let result = session.execute("MATCH (n) RETURN n")?;
///
/// // 使用事务
/// let txn = session.begin_transaction()?;
/// txn.execute("CREATE TAG user(name string)")?;
/// txn.commit()?;
///
/// // 关闭数据库
/// db.close()?;
/// # Ok(())
/// # }
/// ```
pub struct GraphDatabase<S: StorageClient + Clone + 'static> {
    inner: Arc<GraphDatabaseInner<S>>,
    config: DatabaseConfig,
    storage: Arc<Mutex<S>>,
}

impl GraphDatabase<RedbStorage> {
    /// 打开或创建数据库（文件模式）
    ///
    /// # 参数
    /// - `path` - 数据库文件路径
    ///
    /// # 返回
    /// - 成功时返回 GraphDatabase 实例
    /// - 失败时返回错误
    pub fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let config = DatabaseConfig::file(path);
        Self::open_with_config(config)
    }

    /// 创建内存数据库
    ///
    /// # 返回
    /// - 成功时返回 GraphDatabase 实例
    /// - 失败时返回错误
    pub fn open_in_memory() -> CoreResult<Self> {
        let config = DatabaseConfig::memory();
        Self::open_with_config(config)
    }

    /// 使用配置打开数据库
    ///
    /// # 参数
    /// - `config` - 数据库配置
    ///
    /// # 返回
    /// - 成功时返回 GraphDatabase 实例
    /// - 失败时返回错误
    pub fn open_with_config(config: DatabaseConfig) -> CoreResult<Self> {
        let storage = if config.is_memory() {
            RedbStorage::new().map_err(|e| {
                CoreError::StorageError(format!("初始化内存存储失败: {}", e))
            })?
        } else {
            let path = config.path().ok_or_else(|| {
                CoreError::StorageError("数据库路径为空".to_string())
            })?;
            RedbStorage::new_with_path(path.to_path_buf()).map_err(|e| {
                CoreError::StorageError(format!("初始化存储失败: {}", e))
            })?
        };

        let storage = Arc::new(Mutex::new(storage));
        let db = storage.lock().get_db().clone();

        let txn_manager_config = TransactionManagerConfig::default();
        let txn_manager = Arc::new(TransactionManager::new(db, txn_manager_config));
        let savepoint_manager = Arc::new(SavepointManager::new());

        let query_api = Arc::new(Mutex::new(QueryApi::new(storage.clone())));
        let txn_api = TransactionApi::new(txn_manager.clone());
        let schema_api = SchemaApi::new(storage.clone());

        let inner = Arc::new(GraphDatabaseInner {
            query_api,
            txn_api,
            schema_api,
            txn_manager,
            savepoint_manager,
            storage: storage.clone(),
        });

        Ok(Self {
            inner,
            config,
            storage,
        })
    }
}

impl<S: StorageClient + Clone + 'static> GraphDatabase<S> {
    /// 创建新会话
    ///
    /// # 返回
    /// - 成功时返回 Session 实例
    /// - 失败时返回错误
    pub fn session(&self) -> CoreResult<Session<S>> {
        Ok(Session::new(self.inner.clone()))
    }

    /// 执行简单查询（便捷方法）
    ///
    /// 此方法创建一个临时会话执行查询，适合简单的单次查询场景。
    /// 对于复杂场景，建议使用 session() 创建会话。
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
        let session = self.session()?;
        session.execute(query)
    }

    /// 执行参数化查询（便捷方法）
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    /// - `params` - 查询参数
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

    /// 创建图空间（便捷方法）
    ///
    /// # 参数
    /// - `name` - 空间名称
    /// - `config` - 空间配置
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn create_space(&self, name: &str, space_config: SpaceConfig) -> CoreResult<()> {
        let session = self.session()?;
        session.create_space(name, space_config)
    }

    /// 删除图空间（便捷方法）
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

    /// 列出所有图空间（便捷方法）
    pub fn list_spaces(&self) -> CoreResult<Vec<String>> {
        let session = self.session()?;
        session.list_spaces()
    }

    /// 关闭数据库
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn close(self) -> CoreResult<()> {
        // 释放资源
        drop(self.inner);
        drop(self.storage);
        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// 检查是否为内存数据库
    pub fn is_memory(&self) -> bool {
        self.config.is_memory()
    }
}

// 为了支持 Send + Sync
unsafe impl<S: StorageClient + Clone + 'static> Send for GraphDatabase<S> {}
unsafe impl<S: StorageClient + Clone + 'static> Sync for GraphDatabase<S> {}

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
