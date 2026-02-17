//! 事务感知存储层
//!
//! 提供与 TransactionManager 集成的存储接口，支持显式事务管理

use std::sync::Arc;

use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::{RedbStorage, RedbWriter};
use crate::storage::operations::writer::{EdgeWriter, VertexWriter};
use crate::transaction::{TransactionContext, TransactionId, TransactionManager, TransactionOptions};

/// 事务感知存储
///
/// 包装 RedbStorage，提供事务管理功能
#[derive(Clone)]
pub struct TransactionalStorage {
    inner: RedbStorage,
    txn_manager: Arc<TransactionManager>,
}

impl std::fmt::Debug for TransactionalStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionalStorage")
            .field("inner", &self.inner)
            .finish()
    }
}

impl TransactionalStorage {
    /// 创建新的事务感知存储
    ///
    /// # Arguments
    /// * `inner` - 底层的 RedbStorage 实例
    /// * `txn_manager` - 事务管理器
    pub fn new(inner: RedbStorage, txn_manager: Arc<TransactionManager>) -> Self {
        Self { inner, txn_manager }
    }

    /// 获取底层存储的引用
    pub fn inner(&self) -> &RedbStorage {
        &self.inner
    }

    /// 获取事务管理器
    pub fn transaction_manager(&self) -> &TransactionManager {
        &self.txn_manager
    }

    /// 在事务中执行多个存储操作
    ///
    /// # Arguments
    /// * `options` - 事务选项
    /// * `operations` - 闭包，接收事务客户端并执行操作
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(StorageError)` - 操作失败返回的错误
    ///
    /// # Example
    /// ```rust
    /// let result = storage.execute_in_transaction(
    ///     TransactionOptions::default(),
    ///     |client| {
    ///         let id = client.insert_vertex("space", vertex)?;
    ///         client.insert_edge("space", edge)?;
    ///         Ok(id)
    ///     },
    /// );
    /// ```
    pub fn execute_in_transaction<F, R>(
        &self,
        options: TransactionOptions,
        operations: F,
    ) -> Result<R, StorageError>
    where
        F: FnOnce(&mut TransactionalStorageClient) -> Result<R, StorageError>,
    {
        // 开始事务
        let txn_id = self
            .txn_manager
            .begin_transaction(options)
            .map_err(|e| StorageError::DbError(format!("开始事务失败: {}", e)))?;

        // 创建事务客户端
        let mut client = TransactionalStorageClient::new(&self.inner, &self.txn_manager, txn_id);

        // 执行操作
        match operations(&mut client) {
            Ok(result) => {
                // 提交事务
                self.txn_manager
                    .commit_transaction(txn_id)
                    .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
                Ok(result)
            }
            Err(e) => {
                // 中止事务
                let _ = self.txn_manager.abort_transaction(txn_id);
                Err(e)
            }
        }
    }

    /// 开始一个新事务
    ///
    /// # Arguments
    /// * `options` - 事务选项
    ///
    /// # Returns
    /// * `Ok(TransactionId)` - 事务ID
    pub fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionId, StorageError> {
        self.txn_manager
            .begin_transaction(options)
            .map_err(|e| StorageError::DbError(format!("开始事务失败: {}", e)))
    }

    /// 提交事务
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        self.txn_manager
            .commit_transaction(txn_id)
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))
    }

    /// 中止事务
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        self.txn_manager
            .abort_transaction(txn_id)
            .map_err(|e| StorageError::DbError(format!("中止事务失败: {}", e)))
    }
}

/// 事务存储客户端
///
/// 在事务上下文中执行存储操作
pub struct TransactionalStorageClient<'a> {
    storage: &'a RedbStorage,
    txn_manager: &'a TransactionManager,
    txn_id: TransactionId,
}

impl<'a> TransactionalStorageClient<'a> {
    /// 创建新的事务存储客户端
    pub fn new(
        storage: &'a RedbStorage,
        txn_manager: &'a TransactionManager,
        txn_id: TransactionId,
    ) -> Self {
        Self {
            storage,
            txn_manager,
            txn_id,
        }
    }

    /// 获取事务上下文
    fn get_context(&self) -> Result<Arc<TransactionContext>, StorageError> {
        self.txn_manager
            .get_context(self.txn_id)
            .map_err(|e| StorageError::DbError(format!("获取事务上下文失败: {}", e)))
    }

    /// 获取绑定到当前事务的 RedbWriter
    fn get_writer(&self) -> Result<RedbWriter, StorageError> {
        let ctx = self.get_context()?;
        let writer_arc = self.storage.get_writer();
        let mut writer = writer_arc.lock().clone();
        writer.bind_transaction_context(ctx);
        Ok(writer)
    }

    /// 插入顶点
    pub fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let mut writer = self.get_writer()?;
        writer.insert_vertex(space, vertex)
    }

    /// 更新顶点
    pub fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let mut writer = self.get_writer()?;
        writer.update_vertex(space, vertex)
    }

    /// 删除顶点
    pub fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let mut writer = self.get_writer()?;
        writer.delete_vertex(space, id)
    }

    /// 批量插入顶点
    pub fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut writer = self.get_writer()?;
        writer.batch_insert_vertices(space, vertices)
    }

    /// 插入边
    pub fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let mut writer = self.get_writer()?;
        writer.insert_edge(space, edge)
    }

    /// 删除边
    pub fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let mut writer = self.get_writer()?;
        writer.delete_edge(space, src, dst, edge_type)
    }

    /// 批量插入边
    pub fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut writer = self.get_writer()?;
        writer.batch_insert_edges(space, edges)
    }

    /// 删除标签
    pub fn delete_tags(&mut self, space: &str, vertex_id: &Value, tag_names: &[String]) -> Result<usize, StorageError> {
        let mut writer = self.get_writer()?;
        writer.delete_tags(space, vertex_id, tag_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use crate::core::vertex_edge_path::Tag;

    fn create_test_storage() -> (RedbStorage, Arc<TransactionManager>, TempDir) {
        use std::sync::atomic::{AtomicU64, Ordering};
        
        // 使用静态计数器确保每次调用使用不同的文件名
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        
        let temp_dir = TempDir::new().expect("创建临时目录失败");
        // 使用计数器生成唯一的数据库文件名
        let db_path = temp_dir.path().join(format!("test_{}.db", counter));
        
        // 先创建 RedbStorage，它会创建数据库
        let storage = RedbStorage::new_with_path(db_path.clone())
            .expect("创建存储失败");
        
        // 从 RedbStorage 获取数据库实例
        let db = Arc::clone(storage.get_db());
        
        let txn_manager = Arc::new(TransactionManager::new(db, Default::default()));
        (storage, txn_manager, temp_dir)
    }

    #[test]
    fn test_transactional_storage_creation() {
        let (storage, txn_manager, _temp) = create_test_storage();
        let transactional = TransactionalStorage::new(storage, txn_manager);
        assert!(transactional.transaction_manager().stats().total_transactions.load(std::sync::atomic::Ordering::Relaxed) == 0);
    }

    #[test]
    fn test_begin_and_commit_transaction() {
        let (storage, txn_manager, _temp) = create_test_storage();
        let transactional = TransactionalStorage::new(storage, txn_manager);

        let txn_id = transactional
            .begin_transaction(TransactionOptions::default())
            .expect("开始事务失败");

        transactional
            .commit_transaction(txn_id)
            .expect("提交事务失败");
    }

    #[test]
    fn test_begin_and_abort_transaction() {
        let (storage, txn_manager, _temp) = create_test_storage();
        let transactional = TransactionalStorage::new(storage, txn_manager);

        let txn_id = transactional
            .begin_transaction(TransactionOptions::default())
            .expect("开始事务失败");

        transactional
            .abort_transaction(txn_id)
            .expect("中止事务失败");
    }

    #[test]
    fn test_execute_in_transaction() {
        let (storage, txn_manager, _temp) = create_test_storage();
        let transactional = TransactionalStorage::new(storage, txn_manager);

        // 创建测试顶点
        use std::collections::HashMap;
        let vertex = Vertex::new(
            Value::Null(Default::default()),
            vec![Tag {
                name: "Test".to_string(),
                properties: HashMap::new(),
            }],
        );

        // 在事务中插入顶点
        let result = transactional.execute_in_transaction(
            TransactionOptions::default(),
            |client| {
                client.insert_vertex("test_space", vertex)
            },
        );

        assert!(result.is_ok(), "事务执行失败: {:?}", result.err());
    }

    #[test]
    fn test_transaction_rollback_on_error() {
        let (storage, txn_manager, _temp) = create_test_storage();
        let transactional = TransactionalStorage::new(storage, txn_manager);

        // 执行一个会失败的事务
        let result = transactional.execute_in_transaction(
            TransactionOptions::default(),
            |_client| -> Result<Value, StorageError> {
                Err(StorageError::DbError("故意失败".to_string()))
            },
        );

        assert!(result.is_err());
        // 验证事务已中止
        assert_eq!(
            transactional.transaction_manager().stats().aborted_transactions.load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }
}
