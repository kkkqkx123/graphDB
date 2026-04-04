//! Match Fulltext Executor

use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::{DataSet, Value};
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::ast::fulltext::{FulltextMatchCondition, YieldClause};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

#[derive(Debug)]
pub struct MatchFulltextExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    pattern: String,
    fulltext_condition: FulltextMatchCondition,
    yield_clause: Option<YieldClause>,
}

impl<S: StorageClient> MatchFulltextExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        pattern: String,
        fulltext_condition: FulltextMatchCondition,
        yield_clause: Option<YieldClause>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "MatchFulltextExecutor".to_string(),
                storage,
                expr_context,
            ),
            pattern,
            fulltext_condition,
            yield_clause,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for MatchFulltextExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for MatchFulltextExecutor<S> {
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
        "MatchFulltextExecutor"
    }

    fn description(&self) -> &str {
        "Match Fulltext Executor"
    }

    fn stats(&self) -> &crate::query::executor::ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::ExecutorStats {
        self.base.stats_mut()
    }
}
