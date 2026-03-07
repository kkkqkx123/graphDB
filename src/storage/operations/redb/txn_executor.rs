use crate::core::StorageError;
use crate::transaction::TransactionContext;
use redb::Database;
use std::sync::Arc;

/// 写事务执行器
///
/// 封装写事务的执行逻辑，支持绑定事务和独立事务两种模式
pub struct WriteTxnExecutor<'a> {
    /// 绑定的事务上下文（可选）
    bound_context: Option<Arc<TransactionContext>>,
    /// 独立的数据库连接（用于独立事务）
    db: Option<&'a Arc<Database>>,
}

impl<'a> WriteTxnExecutor<'a> {
    /// 创建绑定到事务上下文的执行器
    pub fn bound(context: Arc<TransactionContext>) -> Self {
        Self {
            bound_context: Some(context),
            db: None,
        }
    }

    /// 创建独立事务执行器
    pub fn independent(db: &'a Arc<Database>) -> Self {
        Self {
            bound_context: None,
            db: Some(db),
        }
    }

    /// 执行写操作
    ///
    /// 如果绑定了事务上下文，则在绑定的事务中执行
    /// 否则创建新的独立事务并提交
    pub fn execute<F, R>(&self, operation: F) -> Result<R, StorageError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        match &self.bound_context {
            Some(ctx) => {
                // 检查事务状态
                ctx.can_execute()
                    .map_err(|e| StorageError::DbError(format!("事务状态不允许执行操作: {}", e)))?;

                // 在绑定的事务上下文中执行
                ctx.with_write_txn(operation)
                    .map_err(|e| StorageError::DbError(e.to_string()))
            }
            None => {
                // 创建新的独立事务
                let db = self.db.expect("独立事务需要数据库连接");
                let txn = db
                    .begin_write()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                let result = operation(&txn)?;
                txn.commit()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                Ok(result)
            }
        }
    }
}
