//! Describe Fulltext Index Executor

use parking_lot::Mutex;
use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::core::error::{DBError, QueryError};
use crate::core::DataSet;
use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Executor for describing full-text index metadata
#[derive(Debug)]
pub struct DescribeFulltextIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    space_id: u64,
    coordinator: Arc<FulltextCoordinator>,
}

impl<S: StorageClient> DescribeFulltextIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        space_id: u64,
        expr_context: Arc<ExpressionAnalysisContext>,
        coordinator: Arc<FulltextCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "DescribeFulltextIndexExecutor".to_string(),
                storage,
                expr_context,
            ),
            index_name,
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

impl<S: StorageClient> HasStorage<S> for DescribeFulltextIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for DescribeFulltextIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let parsed = self.parse_index_name();

        let (tag_name, field_name) = match parsed {
            Some((t, f)) => (t, f),
            None => {
                return Err(DBError::Query(QueryError::ExecutionError(format!(
                    "Invalid fulltext index name format: '{}'. Expected format: <prefix>_<tag>_<field>",
                    self.index_name
                ))));
            }
        };

        let metadata = self
            .coordinator
            .get_engine(self.space_id, &tag_name, &field_name);

        match metadata {
            Some(_) => {
                let col_names = vec![
                    "Index Name".to_string(),
                    "Space ID".to_string(),
                    "Tag Name".to_string(),
                    "Field Name".to_string(),
                ];

                let row = vec![
                    Value::String(self.index_name.clone()),
                    Value::Int(self.space_id as i64),
                    Value::String(tag_name),
                    Value::String(field_name),
                ];

                let dataset = DataSet {
                    col_names,
                    rows: vec![row],
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            None => Err(DBError::Query(QueryError::ExecutionError(format!(
                "Fulltext index '{}' not found",
                self.index_name
            )))),
        }
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
