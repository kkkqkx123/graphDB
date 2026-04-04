//! Alter Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;


use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::ast::AlterIndexAction;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Executor for altering full-text indexes
/// 
/// # Note
/// Current implementation is a placeholder. Fields are reserved for future
/// implementation of index alteration logic.
#[derive(Debug)]
pub struct AlterFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    /// Index name to alter (reserved for future implementation)
    #[allow(dead_code)]
    index_name: String,
    /// Alteration actions (reserved for future implementation)
    #[allow(dead_code)]
    actions: Vec<AlterIndexAction>,
}

impl<S: StorageClient> AlterFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        actions: Vec<AlterIndexAction>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "AlterFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            index_name,
            actions,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for AlterFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for AlterFulltextIndexExecutor<S> {
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
        "AlterFulltextIndexExecutor"
    }

    fn description(&self) -> &str {
        "Alter Fulltext Index Executor"
    }

    fn stats(&self) -> &crate::query::executor::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::ExecutorStats {
        self.base.stats_mut()
    }
}
