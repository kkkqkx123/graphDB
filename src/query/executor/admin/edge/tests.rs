#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::query::executor::admin::edge::{
        CreateEdgeExecutor, AlterEdgeExecutor, DescEdgeExecutor, DropEdgeExecutor, ShowEdgesExecutor,
    };
    use crate::query::executor::admin::edge::create_edge::ExecutorEdgeInfo;
    use crate::query::executor::admin::edge::alter_edge::{AlterEdgeInfo, AlterEdgeItem};
    use crate::core::types::PropertyDef;
    use crate::core::DataType;
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[test]
    fn test_create_edge_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let properties = vec![
            PropertyDef::new("weight".to_string(), DataType::Double),
            PropertyDef::new("since".to_string(), DataType::Int64),
        ];
        let edge_info = ExecutorEdgeInfo::new("test_space".to_string(), "knows".to_string())
            .with_properties(properties);

        let mut executor = CreateEdgeExecutor::new(1, storage, edge_info);

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_edge_executor_with_if_not_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let edge_info = ExecutorEdgeInfo::new("test_space".to_string(), "knows".to_string());

        let mut executor = CreateEdgeExecutor::with_if_not_exists(2, storage, edge_info);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_alter_edge_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let new_prop = PropertyDef::new("label".to_string(), DataType::String);
        let items = vec![
            AlterEdgeItem::add_property(new_prop),
            AlterEdgeItem::drop_property("old_field".to_string()),
        ];
        let alter_info = AlterEdgeInfo::new("test_space".to_string(), "knows".to_string())
            .with_items(items);

        let mut executor = AlterEdgeExecutor::new(3, storage, alter_info);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_edge_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = DropEdgeExecutor::new(4, storage, "test_space".to_string(), "knows".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_drop_edge_executor_with_if_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = DropEdgeExecutor::with_if_exists(5, storage, "test_space".to_string(), "knows".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_desc_edge_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = DescEdgeExecutor::new(6, storage, "test_space".to_string(), "knows".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_edges_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let mut executor = ShowEdgesExecutor::new(7, storage, "test_space".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_edge_info_builder() {
        let properties = vec![
            PropertyDef::new("weight".to_string(), DataType::Double),
            PropertyDef::new("since".to_string(), DataType::Int64),
        ];
        let edge_info = ExecutorEdgeInfo::new("my_space".to_string(), "knows".to_string())
            .with_properties(properties)
            .with_comment("Friend relationship".to_string());

        assert_eq!(edge_info.space_name, "my_space");
        assert_eq!(edge_info.edge_name, "knows");
        assert_eq!(edge_info.properties.len(), 2);
        assert_eq!(edge_info.comment, Some("Friend relationship".to_string()));
    }

    #[test]
    fn test_alter_edge_info_builder() {
        let new_prop = PropertyDef::new("label".to_string(), DataType::String);
        let items = vec![
            AlterEdgeItem::add_property(new_prop),
            AlterEdgeItem::drop_property("old_field".to_string()),
        ];
        let alter_info = AlterEdgeInfo::new("test_space".to_string(), "knows".to_string())
            .with_items(items);

        assert_eq!(alter_info.items.len(), 2);
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let edge_info = ExecutorEdgeInfo::new("test_space".to_string(), "knows".to_string());
        let mut executor = CreateEdgeExecutor::new(8, storage, edge_info);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let edge_info = ExecutorEdgeInfo::new("test_space".to_string(), "knows".to_string());
        let executor = CreateEdgeExecutor::new(9, storage, edge_info);

        assert_eq!(executor.id(), 9);
        assert_eq!(executor.name(), "CreateEdgeExecutor");
        assert_eq!(executor.description(), "Creates a new edge type");
        assert!(executor.stats().num_rows == 0);
    }
}
