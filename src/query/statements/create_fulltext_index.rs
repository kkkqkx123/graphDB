use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::query::ast::fulltext::CreateFulltextIndexStmt;
use crate::query::executor::base::ExecutionResult;
use crate::core::error::DBError;

pub struct CreateFulltextIndexExecutor {
    coordinator: Arc<FulltextCoordinator>,
}

impl CreateFulltextIndexExecutor {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }

    pub async fn execute(
        &self,
        stmt: &CreateFulltextIndexStmt,
        space_id: u64,
    ) -> Result<ExecutionResult, DBError> {
        if stmt.fields.len() != 1 {
            return Err(DBError::Internal(
                "Multi-field fulltext index not yet supported".to_string()
            ));
        }

        let field_name = &stmt.fields[0].name;
        let tag_name = &stmt.tag_name.name;

        let index_id = self.coordinator
            .create_index(space_id, tag_name, field_name, stmt.engine_type)
            .await
            .map_err(|e| DBError::Index(e.to_string()))?;

        Ok(ExecutionResult::Success)
    }
}
