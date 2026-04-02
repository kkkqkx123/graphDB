//! Label Operated Actuators
//!
//! Responsible for removing labels from vertices

use std::sync::Arc;
use std::time::Instant;

use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// Delete Label Enforcer
///
/// Responsible for removing labels from vertices
pub struct DeleteTagExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    tag_names: Vec<String>,
    vertex_ids: Vec<Value>,
    space_name: String,
    delete_all_tags: bool,
}

impl<S: StorageClient> DeleteTagExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        tag_names: Vec<String>,
        vertex_ids: Vec<Value>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteTagExecutor".to_string(), storage, expr_context),
            tag_names,
            vertex_ids,
            space_name: "default".to_string(),
            delete_all_tags: false,
        }
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }

    /// Setting the Delete All Tabs Mode
    pub fn delete_all_tags(mut self) -> Self {
        self.delete_all_tags = true;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteTagExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => Ok(ExecutionResult::Count(count)),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DeleteTagExecutor"
    }

    fn description(&self) -> &str {
        "Delete tag executor - removes tags from vertices"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DeleteTagExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;
        let mut storage = self.get_storage().lock();

        for vertex_id in &self.vertex_ids {
            // If you are in delete all labels mode, first get all label names of the vertices
            let tag_names_to_delete = if self.delete_all_tags {
                match storage.get_vertex(&self.space_name, vertex_id) {
                    Ok(Some(vertex)) => vertex
                        .tags
                        .iter()
                        .map(|tag| tag.name.clone())
                        .collect::<Vec<_>>(),
                    Ok(None) => {
                        // Vertex does not exist, skip
                        continue;
                    }
                    Err(_) => {
                        continue;
                    }
                }
            } else {
                self.tag_names.clone()
            };

            match storage.delete_tags(&self.space_name, vertex_id, &tag_names_to_delete) {
                Ok(deleted_count) => {
                    total_deleted += deleted_count;
                }
                Err(_) => {
                    // Logging errors but continuing to process other vertices
                }
            }
        }

        Ok(total_deleted)
    }
}
