//! Show Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::core::DataSet;
use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

#[derive(Debug)]
pub struct ShowFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    coordinator: Arc<FulltextCoordinator>,
}

impl<S: StorageClient> ShowFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
        coordinator: Arc<FulltextCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ShowFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            coordinator,
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
        let indexes = self.coordinator.list_indexes();

        let col_names = vec![
            "Index Name".to_string(),
            "Space ID".to_string(),
            "Tag Name".to_string(),
            "Field Name".to_string(),
            "Engine Type".to_string(),
            "Doc Count".to_string(),
            "Status".to_string(),
            "Created At".to_string(),
        ];

        let rows: Vec<Vec<Value>> = indexes
            .into_iter()
            .map(|meta| {
                vec![
                    Value::String(meta.index_name),
                    Value::Int(meta.space_id as i64),
                    Value::String(meta.tag_name),
                    Value::String(meta.field_name),
                    Value::String(meta.engine_type.to_string()),
                    Value::Int(meta.doc_count as i64),
                    Value::String(format!("{:?}", meta.status)),
                    Value::String(meta.created_at.to_rfc3339()),
                ]
            })
            .collect();

        let dataset = DataSet { col_names, rows };
        Ok(ExecutionResult::DataSet(dataset))
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
