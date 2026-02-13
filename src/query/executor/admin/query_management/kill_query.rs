//! KillQueryExecutor - 终止查询执行器
//!
//! 负责终止正在运行的查询。

use std::sync::{Arc, Mutex};

use crate::api::session::GLOBAL_QUERY_MANAGER;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 终止查询执行器
///
/// 该执行器负责终止正在运行的查询。
#[derive(Debug)]
pub struct KillQueryExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    query_id: i64,
}

impl<S: StorageClient> KillQueryExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, query_id: i64) -> Self {
        Self {
            base: BaseExecutor::new(id, "KillQueryExecutor".to_string(), storage),
            query_id,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for KillQueryExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let killed = GLOBAL_QUERY_MANAGER.get().map(|qm| qm.kill_query(self.query_id)).unwrap_or(false);

        if killed {
            Ok(ExecutionResult::Success)
        } else {
            Ok(ExecutionResult::Error(format!(
                "Failed to kill query {}: query not found or not running",
                self.query_id
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
        "KillQueryExecutor"
    }

    fn description(&self) -> &str {
        "Kills a running query"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for KillQueryExecutor<S> {
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
    fn test_kill_query_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = KillQueryExecutor::new(1, storage, 123);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = KillQueryExecutor::new(2, storage, 123);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let executor = KillQueryExecutor::new(3, storage, 123);

        assert_eq!(executor.id(), 3);
        assert_eq!(executor.name(), "KillQueryExecutor");
        assert_eq!(executor.description(), "Kills a running query");
        assert!(executor.stats().num_rows == 0);
    }
}
