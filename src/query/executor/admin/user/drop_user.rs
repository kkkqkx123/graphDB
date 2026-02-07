//! DropUserExecutor - 删除用户执行器
//!
//! 负责删除数据库用户。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 删除用户执行器
///
/// 该执行器负责删除用户。
#[derive(Debug)]
pub struct DropUserExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    username: String,
    if_exists: bool,
}

impl<S: StorageClient> DropUserExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, username: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropUserExecutor".to_string(), storage),
            username,
            if_exists: false,
        }
    }

    pub fn with_if_exists(id: i64, storage: Arc<Mutex<S>>, username: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropUserExecutor".to_string(), storage),
            username,
            if_exists: true,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropUserExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage = storage.lock().map_err(|e| crate::core::error::DBError::Storage(
            crate::core::StorageError::DbError(e.to_string())
        ))?;
        let result = storage.drop_user(&self.username);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Err(crate::core::error::DBError::Storage(
                        crate::core::StorageError::DbError("User not found".to_string())
                    ))
                }
            }
            Err(e) => Err(crate::core::error::DBError::Storage(e)),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DropUserExecutor"
    }

    fn description(&self) -> &str {
        "Drops a user"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DropUserExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[tokio::test]
    async fn test_drop_user_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropUserExecutor::new(1, storage, "test_user".to_string());

        let result = executor.execute().await;
        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_drop_user_executor_if_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropUserExecutor::with_if_exists(2, storage, "test_user".to_string());

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropUserExecutor::new(3, storage, "test_user".to_string());

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let executor = DropUserExecutor::new(4, storage, "test_user".to_string());

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "DropUserExecutor");
        assert_eq!(executor.description(), "Drops a user");
        assert!(executor.stats().num_rows == 0);
    }
}
