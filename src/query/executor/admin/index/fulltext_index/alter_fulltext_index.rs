//! Alter Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::ast::AlterIndexAction;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Executor for altering full-text indexes
#[derive(Debug)]
pub struct AlterFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    /// Index name to alter
    index_name: String,
    /// Alteration actions
    actions: Vec<AlterIndexAction>,
    /// Fulltext coordinator
    coordinator: Arc<FulltextCoordinator>,
}

impl<S: StorageClient> AlterFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        actions: Vec<AlterIndexAction>,
        expr_context: Arc<ExpressionAnalysisContext>,
        coordinator: Arc<FulltextCoordinator>,
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
            coordinator,
        }
    }

    /// Execute alter index actions
    async fn execute_alter_actions(&self) -> DBResult<ExecutionResult> {
        let parts: Vec<&str> = self.index_name.split('_').collect();
        if parts.len() != 3 {
            return Err(crate::core::error::DBError::Internal(format!(
                "Invalid index name format: {}",
                self.index_name
            )));
        }

        let space_id: u64 = parts[0].parse().map_err(|_| {
            crate::core::error::DBError::Internal(format!(
                "Invalid space_id in index name: {}",
                self.index_name
            ))
        })?;
        let tag_name = parts[1];
        let field_name = parts[2];

        for action in &self.actions {
            match action {
                AlterIndexAction::Rebuild => {
                    self.coordinator
                        .rebuild_index(space_id, tag_name, field_name)
                        .await
                        .map_err(|e| {
                            crate::core::error::DBError::Internal(format!(
                                "Failed to rebuild index: {}",
                                e
                            ))
                        })?;
                }
                AlterIndexAction::Optimize => {
                    self.coordinator
                        .commit_all()
                        .await
                        .map_err(|e| {
                            crate::core::error::DBError::Internal(format!(
                                "Failed to optimize index: {}",
                                e
                            ))
                        })?;
                }
                AlterIndexAction::AddField(_) => {
                    return Err(crate::core::error::DBError::Internal(
                        "AddField action is not supported yet".to_string(),
                    ));
                }
                AlterIndexAction::DropField(_) => {
                    return Err(crate::core::error::DBError::Internal(
                        "DropField action is not supported yet".to_string(),
                    ));
                }
                AlterIndexAction::SetOption(_, _) => {
                    return Err(crate::core::error::DBError::Internal(
                        "SetOption action is not supported yet".to_string(),
                    ));
                }
            }
        }

        Ok(ExecutionResult::Success)
    }
}

impl<S: StorageClient> HasStorage<S> for AlterFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for AlterFulltextIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        tokio::runtime::Handle::current()
            .block_on(self.execute_alter_actions())
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
