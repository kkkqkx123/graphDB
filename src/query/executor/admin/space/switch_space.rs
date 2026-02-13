//! SwitchSpaceExecutor - 切换空间执行器
//!
//! 负责切换当前会话的空间。

use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 切换空间执行器
///
/// 该执行器负责切换当前会话的空间。
#[derive(Debug)]
pub struct SwitchSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> SwitchSpaceExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "SwitchSpaceExecutor".to_string(), storage),
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for SwitchSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let space_exists = storage_guard.space_exists(&self.space_name);

        if space_exists {
            Ok(ExecutionResult::Success)
        } else {
            Ok(ExecutionResult::Error(format!(
                "Space '{}' does not exist",
                self.space_name
            )))
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
        "SwitchSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Switches to a different space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for SwitchSpaceExecutor<S> {
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
    fn test_switch_space_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = SwitchSpaceExecutor::new(1, storage, "test_space".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = SwitchSpaceExecutor::new(2, storage, "test_space".to_string());

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let executor = SwitchSpaceExecutor::new(3, storage, "test_space".to_string());

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "SwitchSpaceExecutor");
        assert_eq!(executor.description(), "Switches to a different space");
        assert!(executor.stats().num_rows == 0);
    }
}
