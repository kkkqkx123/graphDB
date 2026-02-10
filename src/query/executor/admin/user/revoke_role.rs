//! RevokeRoleExecutor - 撤销角色执行器
//!
//! 负责撤销用户在指定空间的角色权限。

use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 撤销角色执行器
///
/// 该执行器负责撤销用户在指定空间的角色权限。
#[derive(Debug)]
pub struct RevokeRoleExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    username: String,
    space_name: String,
}

impl<S: StorageClient> RevokeRoleExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, username: String, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "RevokeRoleExecutor".to_string(), storage),
            username,
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for RevokeRoleExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let space_id = storage_guard.get_space_id(&self.space_name).map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Failed to get space ID: {}", e))
            )
        })?;

        let result = storage_guard.revoke_role(&self.username, space_id);

        match result {
            Ok(_) => Ok(ExecutionResult::Success),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to revoke role: {}", e))),
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
        "RevokeRoleExecutor"
    }

    fn description(&self) -> &str {
        "Revokes a role from a user in a space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for RevokeRoleExecutor<S> {
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
    async fn test_revoke_role_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = RevokeRoleExecutor::new(
            1,
            storage,
            "test_user".to_string(),
            "test_space".to_string(),
        );

        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = RevokeRoleExecutor::new(
            2,
            storage,
            "test_user".to_string(),
            "test_space".to_string(),
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
        let executor = RevokeRoleExecutor::new(
            3,
            storage,
            "test_user".to_string(),
            "test_space".to_string(),
        );

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "RevokeRoleExecutor");
        assert_eq!(executor.description(), "Revokes a role from a user in a space");
        assert!(executor.stats().num_rows == 0);
    }
}
