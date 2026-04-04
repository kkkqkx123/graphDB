//! Describe Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;


use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Executor for describing full-text index metadata
/// 
/// # Note
/// Current implementation is a placeholder. The `index_name` field is reserved
/// for future implementation of index metadata retrieval logic.
#[derive(Debug)]
pub struct DescribeFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    /// Index name to describe (reserved for future implementation)
    #[allow(dead_code)]
    index_name: String,
}

impl<S: StorageClient> DescribeFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DescribeFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            index_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for DescribeFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for DescribeFulltextIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Empty)
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        "DescribeFulltextIndexExecutor"
    }

    fn description(&self) -> &str {
        "Describe Fulltext Index Executor"
    }

    fn stats(&self) -> &crate::query::executor::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::ExecutorStats {
        self.base.stats_mut()
    }
}
