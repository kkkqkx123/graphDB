//! DropTagExecutor - Drop Tag Executor
//!
//! Responsible for deleting the specified tag and all its data.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Delete Label Enforcer
///
/// This actuator is responsible for deleting the specified tag and all its data.
#[derive(Debug)]
pub struct DropTagExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    tag_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropTagExecutor<S> {
    /// Create a new DropTagExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_name: String,
        tag_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropTagExecutor".to_string(), storage, expr_context),
            space_name,
            tag_name,
            if_exists: false,
        }
    }

    /// Creating a DropTagExecutor with the IF EXISTS option
    pub fn with_if_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_name: String,
        tag_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropTagExecutor".to_string(), storage, expr_context),
            space_name,
            tag_name,
            if_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropTagExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let vertices_with_tag = storage_guard
            .scan_vertices_by_tag(&self.space_name, &self.tag_name)
            .unwrap_or_default();

        if !vertices_with_tag.is_empty() {
            return Ok(ExecutionResult::Error(format!(
                "Cannot drop tag '{}': {} vertices are using this tag",
                self.tag_name,
                vertices_with_tag.len()
            )));
        }

        let result = storage_guard.drop_tag(&self.space_name, &self.tag_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!(
                        "Tag '{}' not found in space '{}'",
                        self.tag_name, self.space_name
                    )))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to drop tag: {}", e))),
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
        "DropTagExecutor"
    }

    fn description(&self) -> &str {
        "Drops a tag"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DropTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
