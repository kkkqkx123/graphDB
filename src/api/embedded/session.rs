//! 会话管理模块
//!
//! 提供会话（Session）概念，作为查询执行的上下文

use crate::api::core::{CoreError, CoreResult, QueryApi, QueryContext, SchemaApi, TransactionApi};
use crate::api::embedded::batch::BatchInserter;
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::statement::PreparedStatement;
use crate::api::embedded::transaction::{Transaction, TransactionConfig};
use crate::core::Value;
use crate::storage::StorageClient;
use crate::transaction::{TransactionManager, TransactionOptions, SavepointManager};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// 会话 - 执行上下文
///
/// 会话是查询执行的基本单元，维护当前图空间、事务状态等上下文信息
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
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
/// # Ok(())
/// # }
/// ```
pub struct Session<S: StorageClient + Clone + 'static> {
    db: Arc<GraphDatabaseInner<S>>,
    space_id: Option<u64>,
    space_name: Option<String>,
    auto_commit: bool,
}

/// 数据库内部结构，用于在 Session 和 GraphDatabase 之间共享
pub(crate) struct GraphDatabaseInner<S: StorageClient + 'static> {
    pub(crate) query_api: Arc<Mutex<QueryApi<S>>>,
    pub(crate) txn_api: TransactionApi,
    pub(crate) schema_api: SchemaApi<S>,
    pub(crate) txn_manager: Arc<TransactionManager>,
    pub(crate) savepoint_manager: Arc<SavepointManager>,
    pub(crate) storage: Arc<Mutex<S>>,
}

impl<S: StorageClient + Clone + 'static> Session<S> {
    /// 创建新会话
    pub(crate) fn new(
        db: Arc<GraphDatabaseInner<S>>,
    ) -> Self {
        Self {
            db,
            space_id: None,
            space_name: None,
            auto_commit: true,
        }
    }

    /// 切换图空间
    ///
    /// # 参数
    /// - `space_name` - 图空间名称
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误（如空间不存在）
    pub fn use_space(&mut self, space_name: &str) -> CoreResult<()> {
        let space_id = self.db.schema_api.use_space(space_name)?;
        self.space_id = Some(space_id);
        self.space_name = Some(space_name.to_string());
        Ok(())
    }

    /// 获取当前图空间名称
    pub fn current_space(&self) -> Option<&str> {
        self.space_name.as_deref()
    }

    /// 获取当前图空间 ID
    pub fn current_space_id(&self) -> Option<u64> {
        self.space_id
    }

    /// 设置自动提交模式
    ///
    /// 当 auto_commit 为 true 时，每个查询都会自动提交
    /// 当 auto_commit 为 false 时，需要显式使用事务
    pub fn set_auto_commit(&mut self, auto_commit: bool) {
        self.auto_commit = auto_commit;
    }

    /// 获取自动提交模式
    pub fn auto_commit(&self) -> bool {
        self.auto_commit
    }

    /// 执行查询语句
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
        let ctx = QueryContext {
            space_id: self.space_id,
            auto_commit: self.auto_commit,
            transaction_id: None,
            parameters: None,
        };

        let mut query_api = self.db.query_api.lock();
        let result = query_api.execute(query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// 执行参数化查询
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
        let ctx = QueryContext {
            space_id: self.space_id,
            auto_commit: self.auto_commit,
            transaction_id: None,
            parameters: Some(params),
        };

        let mut query_api = self.db.query_api.lock();
        let result = query_api.execute(query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// 开始事务
    ///
    /// # 返回
    /// - 成功时返回事务句柄
    /// - 失败时返回错误
    pub fn begin_transaction(&self) -> CoreResult<Transaction<S>> {
        let options = TransactionOptions::default();
        let txn_handle = self.db.txn_api.begin(options)?;

        Ok(Transaction::new(self, txn_handle))
    }

    /// 使用配置开始事务
    ///
    /// # 参数
    /// - `config` - 事务配置选项
    ///
    /// # 返回
    /// - 成功时返回事务句柄
    /// - 失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::{GraphDatabase, TransactionConfig};
    /// use std::time::Duration;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let session = db.session()?;
    ///
    /// // 创建只读事务
    /// let config = TransactionConfig::new()
    ///     .read_only()
    ///     .with_timeout(Duration::from_secs(60));
    ///
    /// let txn = session.begin_transaction_with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_transaction_with_config(&self, config: TransactionConfig) -> CoreResult<Transaction<S>> {
        let options = config.into_options();
        let txn_handle = self.db.txn_api.begin(options)?;

        Ok(Transaction::new(self, txn_handle))
    }

    /// 在事务中执行操作（自动提交/回滚）
    ///
    /// # 参数
    /// - `f` - 在事务中执行的闭包
    ///
    /// # 返回
    /// - 成功时返回闭包的返回值
    /// - 失败时返回错误
    pub fn with_transaction<F, T>(&self, f: F) -> CoreResult<T>
    where
        F: FnOnce(&Transaction<S>) -> CoreResult<T>,
    {
        let txn = self.begin_transaction()?;

        match f(&txn) {
            Ok(result) => {
                txn.commit()?;
                Ok(result)
            }
            Err(e) => {
                let _ = txn.rollback();
                Err(e)
            }
        }
    }

    /// 创建图空间
    ///
    /// # 参数
    /// - `name` - 空间名称
    /// - `config` - 空间配置
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn create_space(
        &self,
        name: &str,
        config: crate::api::core::SpaceConfig,
    ) -> CoreResult<()> {
        self.db.schema_api.create_space(name, config)
    }

    /// 删除图空间
    ///
    /// # 参数
    /// - `name` - 空间名称
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn drop_space(&self, name: &str) -> CoreResult<()> {
        self.db.schema_api.drop_space(name)
    }

    /// 列出所有图空间
    pub fn list_spaces(&self) -> CoreResult<Vec<String>> {
        // 通过存储层获取所有空间
        let storage = self.db.storage.lock();
        let spaces = storage.list_spaces()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        Ok(spaces.into_iter().map(|s| s.space_name).collect())
    }

    /// 获取查询 API 的锁（内部使用）
    pub(crate) fn query_api(&self) -> parking_lot::MutexGuard<'_, QueryApi<S>> {
        self.db.query_api.as_ref().lock()
    }

    /// 获取事务 API 的引用（内部使用）
    pub(crate) fn txn_api(&self) -> &TransactionApi {
        &self.db.txn_api
    }

    /// 获取空间 ID（内部使用）
    pub(crate) fn space_id(&self) -> Option<u64> {
        self.space_id
    }

    /// 获取事务管理器（内部使用）
    pub(crate) fn txn_manager(&self) -> Arc<TransactionManager> {
        self.db.txn_manager.clone()
    }

    /// 获取保存点管理器（内部使用）
    pub(crate) fn savepoint_manager(&self) -> Arc<SavepointManager> {
        self.db.savepoint_manager.clone()
    }

    /// 获取存储的锁（内部使用）
    pub(crate) fn storage(&self) -> parking_lot::MutexGuard<'_, S> {
        self.db.storage.lock()
    }

    /// 获取当前空间名称（内部使用）
    pub(crate) fn space_name(&self) -> Option<&str> {
        self.space_name.as_deref()
    }

    /// 创建批量插入器
    ///
    /// # 参数
    /// - `batch_size` - 批次大小，达到此数量时自动刷新
    ///
    /// # 返回
    /// - 返回 BatchInserter 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    /// use graphdb::core::{Vertex, Value};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open_in_memory()?;
    /// let session = db.session()?;
    ///
    /// // 创建批量插入器，每100条自动刷新
    /// let mut inserter = session.batch_inserter(100);
    ///
    /// // 添加顶点
    /// for i in 0..1000 {
    ///     let vertex = Vertex::with_vid(Value::Int(i));
    ///     inserter.add_vertex(vertex);
    /// }
    ///
    /// // 执行批量插入
    /// let result = inserter.execute()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter<'_, S> {
        BatchInserter::new(self, batch_size)
    }

    /// 预编译查询语句
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    ///
    /// # 返回
    /// - 成功时返回 PreparedStatement 实例
    /// - 失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    /// use graphdb::core::Value;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open_in_memory()?;
    /// let session = db.session()?;
    ///
    /// // 预编译查询
    /// let mut stmt = session.prepare("MATCH (n:User {id: $id}) RETURN n")?;
    ///
    /// // 绑定参数并执行
    /// stmt.bind("id", Value::Int(1))?;
    /// let result = stmt.execute()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn prepare(&self, query: &str) -> CoreResult<PreparedStatement<S>> {
        Ok(PreparedStatement::new(
            self.db.query_api.clone(),
            query.to_string(),
            self.space_id,
        ))
    }
}

// 为了支持 Send + Sync，我们需要确保 S 满足这些约束
unsafe impl<S: StorageClient + Clone + 'static> Send for Session<S> {}
unsafe impl<S: StorageClient + Clone + 'static> Sync for Session<S> {}
