//! Drop Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;


use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

#[derive(Debug)]
pub struct DropFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        if_exists: bool,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DropFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            index_name,
            if_exists,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for DropFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for DropFulltextIndexExecutor<S> {
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
        "DropFulltextIndexExecutor"
    }

    fn description(&self) -> &str {
        "Drop Fulltext Index Executor"
    }

    fn stats(&self) -> &crate::query::executor::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::ExecutorStats {
        self.base.stats_mut()
    }
}
