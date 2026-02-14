//! CreateUserExecutor - 创建用户执行器
//!
//! 负责创建新的数据库用户。

use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::types::metadata::UserInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 创建用户执行器
///
/// 该执行器负责在存储层创建新用户。
#[derive(Debug)]
pub struct CreateUserExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    user_info: UserInfo,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateUserExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, user_info: UserInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateUserExecutor".to_string(), storage),
            user_info,
            if_not_exists: false,
        }
    }

    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, user_info: UserInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateUserExecutor".to_string(), storage),
            user_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateUserExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage = storage.lock();
        let result = storage.create_user(&self.user_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Err(crate::core::error::DBError::Storage(
                        crate::core::StorageError::DbError("User already exists".to_string())
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
        "CreateUserExecutor"
    }

    fn description(&self) -> &str {
        "Creates a new user"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for CreateUserExecutor<S> {
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
    fn test_create_user_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let user_info = UserInfo::new("test_user".to_string(), "password123".to_string());
        let mut executor = CreateUserExecutor::new(1, storage, user_info);

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Expected result to exist") {
            ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_user_executor_if_not_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let user_info = UserInfo::new("test_user".to_string(), "password123".to_string());
        let mut executor = CreateUserExecutor::with_if_not_exists(2, storage, user_info);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let user_info = UserInfo::new("test_user".to_string(), "password123".to_string());
        let mut executor = CreateUserExecutor::new(3, storage, user_info);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let user_info = UserInfo::new("test_user".to_string(), "password123".to_string());
        let executor = CreateUserExecutor::new(4, storage, user_info);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "CreateUserExecutor");
        assert_eq!(executor.description(), "Creates a new user");
        assert!(executor.stats().num_rows == 0);
    }
}
