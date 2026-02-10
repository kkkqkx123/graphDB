#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::query::executor::admin::index::{
        CreateTagIndexExecutor, DropTagIndexExecutor, DescTagIndexExecutor, ShowTagIndexesExecutor,
        RebuildTagIndexExecutor,
        CreateEdgeIndexExecutor, DropEdgeIndexExecutor, DescEdgeIndexExecutor, ShowEdgeIndexesExecutor,
        RebuildEdgeIndexExecutor,
    };
    use crate::index::{Index, IndexType};
    use crate::storage::test_mock::MockStorage;
    use crate::query::executor::Executor;

    #[test]
    fn test_create_tag_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "person_name_index".to_string(),
            0,
            "person".to_string(),
            Vec::new(),
            vec!["name".to_string()],
            IndexType::TagIndex,
            false,
        );

        let mut executor = CreateTagIndexExecutor::new(1, storage, index);

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_tag_index_executor_with_if_not_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "person_name_index".to_string(),
            0,
            "person".to_string(),
            Vec::new(),
            vec!["name".to_string()],
            IndexType::TagIndex,
            false,
        );

        let mut executor = CreateTagIndexExecutor::with_if_not_exists(2, storage, index);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_tag_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropTagIndexExecutor::new(
            3,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_drop_tag_index_executor_with_if_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropTagIndexExecutor::with_if_exists(
            4,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_desc_tag_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DescTagIndexExecutor::new(
            5,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_tag_indexes_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowTagIndexesExecutor::new(6, storage, "test_space".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebuild_tag_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = RebuildTagIndexExecutor::new(
            7,
            storage,
            "test_space".to_string(),
            "person_name_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_edge_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "knows_weight_index".to_string(),
            0,
            "knows".to_string(),
            Vec::new(),
            vec!["weight".to_string()],
            IndexType::EdgeIndex,
            false,
        );

        let mut executor = CreateEdgeIndexExecutor::new(8, storage, index);

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_create_edge_index_executor_with_if_not_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "knows_weight_index".to_string(),
            0,
            "knows".to_string(),
            Vec::new(),
            vec!["weight".to_string()],
            IndexType::EdgeIndex,
            false,
        );

        let mut executor = CreateEdgeIndexExecutor::with_if_not_exists(9, storage, index);

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_edge_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropEdgeIndexExecutor::new(
            10,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_drop_edge_index_executor_with_if_exists() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DropEdgeIndexExecutor::with_if_exists(
            11,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_desc_edge_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = DescEdgeIndexExecutor::new(
            12,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_edge_indexes_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = ShowEdgeIndexesExecutor::new(13, storage, "test_space".to_string());

        let result = executor.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebuild_edge_index_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let mut executor = RebuildEdgeIndexExecutor::new(
            14,
            storage,
            "test_space".to_string(),
            "knows_weight_index".to_string(),
        );

        let result = executor.execute();
        assert!(result.is_ok());
        match result.unwrap() {
            crate::query::executor::base::ExecutionResult::Success => {}
            _ => panic!("Expected Success result"),
        }
    }

    #[test]
    fn test_executor_lifecycle() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "test_index".to_string(),
            0,
            "person".to_string(),
            Vec::new(),
            vec!["name".to_string()],
            IndexType::TagIndex,
            false,
        );
        let mut executor = CreateTagIndexExecutor::new(15, storage, index);

        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[test]
    fn test_executor_stats() {
        let storage = Arc::new(Mutex::new(MockStorage::new().unwrap()));
        let index = Index::new(
            0,
            "test_index".to_string(),
            0,
            "person".to_string(),
            Vec::new(),
            vec!["name".to_string()],
            IndexType::TagIndex,
            false,
        );
        let executor = CreateTagIndexExecutor::new(16, storage, index);

        assert_eq!(executor.id(), 16);
        assert_eq!(executor.name(), "CreateTagIndexExecutor");
        assert_eq!(executor.description(), "Creates a tag index");
        assert!(executor.stats().num_rows == 0);
    }
}
