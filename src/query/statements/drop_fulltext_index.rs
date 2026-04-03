use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::query::ast::fulltext::DropFulltextIndexStmt;
use crate::query::executor::base::ExecutionResult;
use crate::core::error::DBError;

pub struct DropFulltextIndexExecutor {
    coordinator: Arc<FulltextCoordinator>,
}

impl DropFulltextIndexExecutor {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }

    pub async fn execute(
        &self,
        stmt: &DropFulltextIndexStmt,
        space_id: u64,
    ) -> Result<ExecutionResult, DBError> {
        let index_name = &stmt.index_name.name;

        let parts: Vec<&str> = index_name.split('_').collect();
        if parts.len() != 3 {
            return Err(DBError::Internal(
                format!("Invalid index name format: {}", index_name)
            ));
        }

        let tag_name = parts[1];
        let field_name = parts[2];

        self.coordinator
            .drop_index(space_id, tag_name, field_name)
            .await
            .map_err(|e| DBError::Index(e.to_string()))?;

        Ok(ExecutionResult::Success)
    }
}
