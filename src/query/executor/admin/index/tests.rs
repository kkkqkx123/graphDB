#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::core::types::{Index, IndexType};
    use crate::query::executor::admin::index::{
        CreateEdgeIndexExecutor, CreateTagIndexExecutor, DescEdgeIndexExecutor,
        DescTagIndexExecutor, DropEdgeIndexExecutor, DropTagIndexExecutor,
        RebuildEdgeIndexExecutor, RebuildTagIndexExecutor, ShowEdgeIndexesExecutor,
        ShowTagIndexesExecutor,
    };
    use crate::query::executor::Executor;
    use crate::query::validator::context::ExpressionAnalysisContext;
    use crate::storage::test_mock::MockStorage;
    use parking_lot::Mutex;
    use std::sync::Arc;

    fn create_test_context() -> Arc<ExpressionAnalysisContext> {
        Arc::new(ExpressionAnalysisContext::new())
    }

    #[test]
    fn test_create_tag_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "person_name_index".to_string(),
            space_id: 0,
            schema_name: "person".to_string(),
            fields: Vec::new(),
            properties: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);

        let mut executor = CreateTagIndexExecutor::new(
            1,
            storage,
            "test_space".to_string(),
            index,
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_tag_index_executor_with_if_not_exists() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "person_name_index".to_string(),
            space_id: 0,
            schema_name: "person".to_string(),
            fields: Vec::new(),
            properties: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);

        let mut executor = CreateTagIndexExecutor::with_if_not_exists(
            2,
            storage,
            "test_space".to_string(),
            index,
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_tag_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DropTagIndexExecutor::new(
            3,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_drop_tag_index_executor_with_if_exists() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DropTagIndexExecutor::with_if_exists(
            4,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_desc_tag_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DescTagIndexExecutor::new(
            5,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_tag_indexes_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = ShowTagIndexesExecutor::new(
            6,
            storage,
            "test_space".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebuild_tag_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = RebuildTagIndexExecutor::new(
            7,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_edge_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "knows_weight_index".to_string(),
            space_id: 0,
            schema_name: "knows".to_string(),
            fields: Vec::new(),
            properties: vec!["weight".to_string()],
            index_type: IndexType::EdgeIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);

        let mut executor = CreateEdgeIndexExecutor::new(8, storage, index, create_test_context());

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_edge_index_executor_with_if_not_exists() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "knows_weight_index".to_string(),
            space_id: 0,
            schema_name: "knows".to_string(),
            fields: Vec::new(),
            properties: vec!["weight".to_string()],
            index_type: IndexType::EdgeIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);

        let mut executor =
            CreateEdgeIndexExecutor::with_if_not_exists(9, storage, index, create_test_context());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_edge_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DropEdgeIndexExecutor::new(
            10,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_drop_edge_index_executor_with_if_exists() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DropEdgeIndexExecutor::with_if_exists(
            11,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_desc_edge_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = DescEdgeIndexExecutor::new(
            12,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_edge_indexes_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = ShowEdgeIndexesExecutor::new(
            13,
            storage,
            "test_space".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebuild_edge_index_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let mut executor = RebuildEdgeIndexExecutor::new(
            14,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
            create_test_context(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.expect("Failed to execute query") {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "test_index".to_string(),
            space_id: 0,
            schema_name: "person".to_string(),
            fields: Vec::new(),
            properties: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);
        let mut executor = CreateTagIndexExecutor::new(
            15,
            storage,
            "test_space".to_string(),
            index,
            create_test_context(),
        );

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));
        let index_config = crate::core::types::IndexConfig {
            id: 0,
            name: "test_index".to_string(),
            space_id: 0,
            schema_name: "person".to_string(),
            fields: Vec::new(),
            properties: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            is_unique: false,
        };
        let index = Index::new(index_config);
        let executor = CreateTagIndexExecutor::new(
            16,
            storage,
            "test_space".to_string(),
            index,
            create_test_context(),
        );

        assert_eq!(executor.id(), 16);
        assert_eq!(executor.name(), "CreateTagIndexExecutor");
        assert_eq!(executor.description(), "Creates a tag index");
        assert!(executor.stats().num_rows == 0);
    }
}
