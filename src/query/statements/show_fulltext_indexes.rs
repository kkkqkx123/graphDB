use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::query::ast::fulltext::ShowFulltextIndexesStmt;
use crate::query::executor::base::ExecutionResult;
use crate::core::Value;
use crate::core::error::DBError;
use crate::core::DataSet;

pub struct ShowFulltextIndexesExecutor {
    coordinator: Arc<FulltextCoordinator>,
}

impl ShowFulltextIndexesExecutor {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }

    pub async fn execute(
        &self,
        _stmt: &ShowFulltextIndexesStmt,
    ) -> Result<ExecutionResult, DBError> {
        let indexes = self.coordinator.list_indexes();

        let mut dataset = DataSet::new();
        dataset.add_column("Index Name".to_string(), crate::core::types::DataType::String);
        dataset.add_column("Tag".to_string(), crate::core::types::DataType::String);
        dataset.add_column("Field".to_string(), crate::core::types::DataType::String);
        dataset.add_column("Engine".to_string(), crate::core::types::DataType::String);
        dataset.add_column("Status".to_string(), crate::core::types::DataType::String);
        dataset.add_column("Doc Count".to_string(), crate::core::types::DataType::Int64);

        for metadata in indexes {
            let mut row = Vec::new();
            row.push(Value::String(metadata.index_name.clone()));
            row.push(Value::String(metadata.tag_name.clone()));
            row.push(Value::String(metadata.field_name.clone()));
            row.push(Value::String(metadata.engine_type.to_string()));
            row.push(Value::String(metadata.status.to_string()));
            row.push(Value::Int64(metadata.doc_count as i64));
            dataset.add_row(row);
        }

        Ok(ExecutionResult::DataSet(dataset))
    }
}
