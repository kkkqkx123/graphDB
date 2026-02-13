//! AlterSpaceExecutor - 修改空间执行器
//!
//! 负责修改图空间的配置。

use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 空间修改选项
#[derive(Debug, Clone)]
pub enum SpaceAlterOption {
    PartitionNum(usize),
    ReplicaFactor(usize),
    Comment(String),
}

/// 修改空间执行器
///
/// 该执行器负责修改图空间的配置。
#[derive(Debug)]
pub struct AlterSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    options: Vec<SpaceAlterOption>,
}

impl<S: StorageClient> AlterSpaceExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, options: Vec<SpaceAlterOption>) -> Self {
        Self {
            base: BaseExecutor::new(id, "AlterSpaceExecutor".to_string(), storage),
            space_name,
            options,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AlterSpaceExecutor<S> {
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

        for option in &self.options {
            match option {
                SpaceAlterOption::PartitionNum(num) => {
                    if let Err(e) = storage_guard.alter_space_partition_num(space_id, *num) {
                        return Ok(ExecutionResult::Error(format!("Failed to alter partition num: {}", e)));
                    }
                }
                SpaceAlterOption::ReplicaFactor(factor) => {
                    if let Err(e) = storage_guard.alter_space_replica_factor(space_id, *factor) {
                        return Ok(ExecutionResult::Error(format!("Failed to alter replica factor: {}", e)));
                    }
                }
                SpaceAlterOption::Comment(comment) => {
                    if let Err(e) = storage_guard.alter_space_comment(space_id, comment.clone()) {
                        return Ok(ExecutionResult::Error(format!("Failed to alter comment: {}", e)));
                    }
                }
            }
        }

        Ok(ExecutionResult::Success)
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
        "AlterSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Alters a space's configuration"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for AlterSpaceExecutor<S> {
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
    fn test_alter_space_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let options = vec![
            SpaceAlterOption::PartitionNum(2),
            SpaceAlterOption::ReplicaFactor(1),
        ];
        let mut executor = AlterSpaceExecutor::new(
            1,
            storage,
            "test_space".to_string(),
            options,
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let options = vec![SpaceAlterOption::Comment("test".to_string())];
        let mut executor = AlterSpaceExecutor::new(
            2,
            storage,
            "test_space".to_string(),
            options,
        );

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let options = vec![SpaceAlterOption::PartitionNum(2)];
        let executor = AlterSpaceExecutor::new(
            3,
            storage,
            "test_space".to_string(),
            options,
        );

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "AlterSpaceExecutor");
        assert_eq!(executor.description(), "Alters a space's configuration");
        assert!(executor.stats().num_rows == 0);
    }
}
