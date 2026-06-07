//! ShowEdgeIndexStatusExecutor – Executor for displaying the status of the edge index
//!
//! Responsible for displaying the status information of the side index.

use parking_lot::RwLock;
use std::sync::Arc;

use crate::core::error::DBError;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Display the status of the edge index executor.
///
/// This actuator is responsible for displaying the status information of the edge index.
#[derive(Debug)]
pub struct ShowEdgeIndexStatusExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: Option<String>,
}

impl<S: StorageClient> ShowEdgeIndexStatusExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ShowEdgeIndexStatusExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_name: None,
        }
    }

    pub fn with_index_name(
        id: i64,
        storage: Arc<RwLock<S>>,
        space_name: String,
        index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ShowEdgeIndexStatusExecutor".to_string(),
                storage,
                expr_context,
            ),
            space_name,
            index_name: Some(index_name),
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowEdgeIndexStatusExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        Err(DBError::storage("edge indexes are not supported"))
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
        "ShowEdgeIndexStatusExecutor"
    }

    fn description(&self) -> &str {
        "Shows edge index status"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for ShowEdgeIndexStatusExecutor<S> {
    fn get_storage(&self) -> &Arc<RwLock<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::executor::Executor;
    use crate::storage::MockStorage;
    use ExpressionAnalysisContext;

    #[test]
    fn test_show_edge_index_status_executor() {
        let storage = Arc::new(RwLock::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor =
            ShowEdgeIndexStatusExecutor::new(1, storage, "test_space".to_string(), expr_context);

        let result = executor.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_show_edge_index_status_executor_with_name() {
        let storage = Arc::new(RwLock::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor = ShowEdgeIndexStatusExecutor::with_index_name(
            2,
            storage,
            "test_space".to_string(),
            "test_index".to_string(),
            expr_context,
        );

        let result = executor.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(RwLock::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let mut executor =
            ShowEdgeIndexStatusExecutor::new(3, storage, "test_space".to_string(), expr_context);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(RwLock::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let executor =
            ShowEdgeIndexStatusExecutor::new(4, storage, "test_space".to_string(), expr_context);

        assert_eq!(executor.id(), 4);
        assert_eq!(executor.name(), "ShowEdgeIndexStatusExecutor");
        assert_eq!(executor.description(), "Shows edge index status");
        assert!(executor.stats().num_rows == 0);
    }
}
