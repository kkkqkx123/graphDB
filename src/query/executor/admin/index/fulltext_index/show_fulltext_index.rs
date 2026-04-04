//! Show Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;


use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

#[derive(Debug)]
pub struct ShowFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
}

impl<S: StorageClient> ShowFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ShowFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
        }
    }
}

impl<S: StorageClient> HasStorage<S> for ShowFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for ShowFulltextIndexExecutor<S> {
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
        "ShowFulltextIndexExecutor"
    }

    fn description(&self) -> &str {
        "Show Fulltext Index Executor"
    }

    fn stats(&self) -> &crate::query::executor::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::ExecutorStats {
        self.base.stats_mut()
    }
}
