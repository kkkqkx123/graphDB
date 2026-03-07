use crate::core::StorageError;
use crate::transaction::TransactionContext;
use redb::Database;
use std::sync::Arc;

pub struct WriteTxnExecutor<'a> {
    bound_context: Option<Arc<TransactionContext>>,
    db: Option<&'a Arc<Database>>,
}

impl<'a> WriteTxnExecutor<'a> {
    pub fn bound(context: Arc<TransactionContext>) -> Self {
        Self {
            bound_context: Some(context),
            db: None,
        }
    }

    pub fn independent(db: &'a Arc<Database>) -> Self {
        Self {
            bound_context: None,
            db: Some(db),
        }
    }

    pub fn execute<F, R>(&self, operation: F) -> Result<R, StorageError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        match &self.bound_context {
            Some(ctx) => {
                ctx.can_execute()
                    .map_err(|e| StorageError::DbError(format!("事务状态不允许执行操作: {}", e)))?;

                ctx.with_write_txn(operation)
                    .map_err(|e| StorageError::DbError(e.to_string()))
            }
            None => {
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
