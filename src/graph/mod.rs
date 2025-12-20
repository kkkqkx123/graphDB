//! 图操作核心模块
//!
//! 包含图相关的核心操作，包括事务管理、索引系统和表达式计算

pub mod batch_operation;
pub mod index;
pub mod response;
pub mod result_set;
pub mod schema;
pub mod transaction;
pub mod utils;

// 重新导出图操作相关功能
pub use batch_operation::*;
pub use index::*;
pub use response::*;
pub use result_set::*;
pub use schema::{DataType as SchemaDataType, EntityType, IndexDef, PropertyDef, SchemaDef};
pub use transaction::*;
pub use utils::*;

#[cfg(test)]
#[allow(hidden_glob_reexports)]
mod tests {
    use super::ApiResponse;
    use super::GraphData;
    use super::GraphResponse;
    use super::ResultSet;
    use crate::core::{Tag, Value, Vertex};
    use std::collections::HashMap;

    #[test]
    fn test_graph_response() {
        let vertex = Vertex::new(
            Value::Int(1),
            vec![Tag::new("person".to_string(), HashMap::new())],
        );

        let response = GraphResponse::success_with_data(GraphData::Vertex(vertex), 10);

        assert!(response.success);
        assert_eq!(response.execution_time_ms, 10);
    }

    #[test]
    fn test_result_set() {
        let mut result_set = ResultSet::new(vec!["name".to_string(), "age".to_string()]);
        result_set.add_row(vec![Value::String("Alice".to_string()), Value::Int(30)]);
        result_set.add_row(vec![Value::String("Bob".to_string()), Value::Int(25)]);

        assert_eq!(result_set.columns, vec!["name", "age"]);
        assert_eq!(result_set.rows.len(), 2);
    }

    #[test]
    fn test_api_response() {
        let data = vec![1, 2, 3];
        let api_response = ApiResponse::success(data, "Query executed successfully".to_string());

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());
        assert!(api_response.error.is_none());
    }
}
