//! AlterUserExecutor - 修改用户执行器
//!
//! 负责修改用户属性（如角色、锁定状态等）。

use std::sync::{Arc, Mutex};

use crate::core::types::metadata::UserAlterInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 修改用户执行器
///
/// 该执行器负责修改用户属性。
#[derive(Debug)]
pub struct AlterUserExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    alter_info: UserAlterInfo,
}

impl<S: StorageClient> AlterUserExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, alter_info: UserAlterInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "AlterUserExecutor".to_string(), storage),
            alter_info,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AlterUserExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage = storage.lock().map_err(|e| crate::core::error::DBError::Storage(
            crate::core::StorageError::DbError(e.to_string())
        ))?;
        let result = storage.alter_user(&self.alter_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Err(crate::core::error::DBError::Storage(
                crate::core::StorageError::DbError("Failed to alter user".to_string())
            )),
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
        "AlterUserExecutor"
    }

    fn description(&self) -> &str {
        "Alters a user"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for AlterUserExecutor<S> {
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
    async fn test_alter_user_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let alter_info = UserAlterInfo::new("test_user".to_string());
        let mut executor = AlterUserExecutor::new(1, storage, alter_info);

        let result = executor.execute().await;
        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let alter_info = UserAlterInfo::new("test_user".to_string());
        let mut executor = AlterUserExecutor::new(2, storage, alter_info);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let alter_info = UserAlterInfo::new("test_user".to_string());
        let executor = AlterUserExecutor::new(3, storage, alter_info);

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "AlterUserExecutor");
        assert_eq!(executor.description(), "Alters a user");
        assert!(executor.stats().num_rows == 0);
    }
}
