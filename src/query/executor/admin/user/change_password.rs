//! ChangePasswordExecutor - 修改密码执行器
//!
//! 负责修改用户密码。

use std::sync::{Arc, Mutex};

use crate::core::types::metadata::PasswordInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 修改密码执行器
///
/// 该执行器负责修改用户密码。
#[derive(Debug)]
pub struct ChangePasswordExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    username: String,
    old_password: String,
    new_password: String,
}

impl<S: StorageClient> ChangePasswordExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, username: String, old_password: String, new_password: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ChangePasswordExecutor".to_string(), storage),
            username,
            old_password,
            new_password,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ChangePasswordExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage = storage.lock().map_err(|e| crate::core::error::DBError::Storage(
            crate::core::StorageError::DbError(e.to_string())
        ))?;
        let password_info = PasswordInfo {
            username: self.username.clone(),
            old_password: self.old_password.clone(),
            new_password: self.new_password.clone(),
        };
        let result = storage.change_password(&password_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Err(crate::core::error::DBError::Storage(
                crate::core::StorageError::DbError("Failed to change password".to_string())
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
        "ChangePasswordExecutor"
    }

    fn description(&self) -> &str {
        "Changes a user's password"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for ChangePasswordExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[test]
    fn test_change_password_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ChangePasswordExecutor::new(
            1,
            storage,
            "test_user".to_string(),
            "old_password".to_string(),
            "new_password".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ChangePasswordExecutor::new(
            2,
            storage,
            "test_user".to_string(),
            "old_password".to_string(),
            "new_password".to_string(),
        );

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let executor = ChangePasswordExecutor::new(
            3,
            storage,
            "test_user".to_string(),
            "old_password".to_string(),
            "new_password".to_string(),
        );

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "ChangePasswordExecutor");
        assert_eq!(executor.description(), "Changes a user's password");
        assert!(executor.stats().num_rows == 0);
    }
}
