use crate::core::StorageError;
use redb::Database;
use std::sync::Arc;
use std::time::Duration;

const WRITE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WriteTxnExecutor<'a> {
    db: &'a Arc<Database>,
}

impl<'a> WriteTxnExecutor<'a> {
    pub fn new(db: &'a Arc<Database>) -> Self {
        Self { db }
    }

    pub fn execute<F, R>(&self, operation: F) -> Result<R, StorageError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        let txn = Self::begin_write_with_timeout(self.db, WRITE_LOCK_TIMEOUT)?;
        let result = operation(&txn)?;
        txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(result)
    }

    fn begin_write_with_timeout(
        db: &Arc<Database>,
        timeout: Duration,
    ) -> Result<redb::WriteTransaction, StorageError> {
        let db = Arc::clone(db);
        let (tx, rx) = std::sync::mpsc::channel();

        let _handle = std::thread::spawn(move || {
            let result = db.begin_write();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result.map_err(|e| StorageError::DbError(e.to_string())),
            Err(_) => Err(StorageError::DbError(format!(
                "Timed out acquiring write lock after {:?}. \
                 Another write transaction may be blocking.",
                timeout
            ))),
        }
    }
}
