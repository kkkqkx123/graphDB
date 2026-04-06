//! Drop Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::core::error::{CoordinatorError, DBError, FulltextError, QueryError};
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Executor for dropping full-text indexes
#[derive(Debug)]
pub struct DropFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    if_exists: bool,
    space_id: u64,
    coordinator: Arc<FulltextCoordinator>,
}

impl<S: StorageClient> DropFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        if_exists: bool,
        space_id: u64,
        expr_context: Arc<ExpressionAnalysisContext>,
        coordinator: Arc<FulltextCoordinator>,
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
            space_id,
            coordinator,
        }
    }

    fn parse_index_name(&self) -> Option<(String, String)> {
        let parts: Vec<&str> = self.index_name.split('_').collect();
        if parts.len() >= 3 {
            let tag_name = parts[1].to_string();
            let field_name = parts[2..].join("_");
            Some((tag_name, field_name))
        } else {
            None
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
        let parsed = self.parse_index_name();

        match parsed {
            Some((tag_name, field_name)) => {
                let result = futures::executor::block_on(
                    self.coordinator
                        .drop_index(self.space_id, &tag_name, &field_name),
                );

                match result {
                    Ok(()) => {
                        log::info!(
                            "Dropped fulltext index '{}' on {}.{}",
                            self.index_name,
                            tag_name,
                            field_name
                        );
                    }
                    Err(CoordinatorError::Fulltext(FulltextError::IndexNotFound(_))) => {
                        if self.if_exists {
                            log::warn!(
                                "Fulltext index '{}' does not exist, skipping",
                                self.index_name
                            );
                        } else {
                            return Err(DBError::from(CoordinatorError::Fulltext(
                                FulltextError::IndexNotFound(self.index_name.clone()),
                            )));
                        }
                    }
                    Err(e) => {
                        return Err(DBError::from(e));
                    }
                }
            }
            None => {
                if !self.if_exists {
                    return Err(DBError::Query(QueryError::ExecutionError(format!(
                        "Invalid fulltext index name format: '{}'. Expected format: <prefix>_<tag>_<field>",
                        self.index_name
                    ))));
                }
                log::warn!(
                    "Invalid fulltext index name format '{}', skipping due to IF EXISTS",
                    self.index_name
                );
            }
        }

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
