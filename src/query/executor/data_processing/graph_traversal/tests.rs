#[cfg(test)]
mod tests {
    use crate::config::test_config::test_config;
    use crate::core::{Edge, Value, Vertex};
    use crate::query::executor::base::EdgeDirection;
    use crate::query::executor::data_processing::graph_traversal::factory::GraphTraversalExecutorFactory;
    use crate::query::executor::data_processing::graph_traversal::shortest_path::ShortestPathAlgorithm;
    use crate::query::executor::data_processing::graph_traversal::traits::GraphTraversalExecutor;
    use crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor;
    use crate::query::executor::traits::Executor;
    use crate::storage::{NativeStorage, StorageEngine};
    use std::sync::{Arc, Mutex};

    async fn create_test_graph(test_name: &str) -> Arc<Mutex<NativeStorage>> {
        let config = test_config();
        let db_path = config.test_db_path(&format!("test_graph_{}", test_name));
        let storage = Arc::new(Mutex::new(
            NativeStorage::new(db_path).expect("Failed to create test storage"),
        ));

        // 创建测试图：A -> B -> C, A -> D
        {
            let mut storage_lock = storage
                .lock()
                .expect("Test storage lock should not be poisoned");

            // 创建顶点
            let vertex_a = Vertex::new(Value::String("A".to_string()), vec![]);
            let vertex_b = Vertex::new(Value::String("B".to_string()), vec![]);
            let vertex_c = Vertex::new(Value::String("C".to_string()), vec![]);
            let vertex_d = Vertex::new(Value::String("D".to_string()), vec![]);

            let id_a = storage_lock
                .insert_node(vertex_a)
                .expect("Failed to insert test vertex A");
            let id_b = storage_lock
                .insert_node(vertex_b)
                .expect("Failed to insert test vertex B");
            let id_c = storage_lock
                .insert_node(vertex_c)
                .expect("Failed to insert test vertex C");
            let id_d = storage_lock
                .insert_node(vertex_d)
                .expect("Failed to insert test vertex D");

            // 创建边
            let edge_ab = Edge::new(
                id_a.clone(),
                id_b.clone(),
                "connect".to_string(),
                0,
                std::collections::HashMap::new(),
            );
            let edge_bc = Edge::new(
                id_b.clone(),
                id_c.clone(),
                "connect".to_string(),
                0,
                std::collections::HashMap::new(),
            );
            let edge_ad = Edge::new(
                id_a.clone(),
                id_d.clone(),
                "connect".to_string(),
                0,
                std::collections::HashMap::new(),
            );

            storage_lock
                .insert_edge(edge_ab)
                .expect("Failed to insert test edge AB");
            storage_lock
                .insert_edge(edge_bc)
                .expect("Failed to insert test edge BC");
            storage_lock
                .insert_edge(edge_ad)
                .expect("Failed to insert test edge AD");
        }

        storage
    }

    #[tokio::test]
    async fn test_expand_executor() {
        let storage = create_test_graph("expand").await;
        let executor = GraphTraversalExecutorFactory::create_expand_executor(
            1,
            storage,
            EdgeDirection::Outgoing,
            Some(vec!["connect".to_string()]),
            Some(1),
        );

        // 测试基本功能
        assert_eq!(executor.name(), "ExpandExecutor");
        assert_eq!(executor.id(), 1);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Outgoing
        ));
        assert!(executor.get_edge_types().is_some());
        assert_eq!(executor.get_max_depth(), Some(1));
    }

    #[tokio::test]
    async fn test_expand_all_executor() {
        let storage = create_test_graph("expand_all").await;
        let executor = GraphTraversalExecutorFactory::create_expand_all_executor(
            2,
            storage,
            EdgeDirection::Both,
            None,
            Some(2),
        );

        assert_eq!(executor.name(), "ExpandAllExecutor");
        assert_eq!(executor.id(), 2);
        assert!(matches!(executor.get_edge_direction(), EdgeDirection::Both));
        assert!(executor.get_edge_types().is_none());
        assert_eq!(executor.get_max_depth(), Some(2));
    }

    #[tokio::test]
    async fn test_traverse_executor() {
        let storage = create_test_graph("traverse").await;
        let executor = GraphTraversalExecutorFactory::create_traverse_executor(
            3,
            storage,
            EdgeDirection::Outgoing,
            Some(vec!["connect".to_string()]),
            Some(3),
            Some("true".to_string()),
        );

        assert_eq!(executor.name(), "TraverseExecutor");
        assert_eq!(executor.id(), 3);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Outgoing
        ));
        assert!(executor.get_edge_types().is_some());
        assert_eq!(executor.get_max_depth(), Some(3));
    }

    #[tokio::test]
    async fn test_shortest_path_executor() {
        let storage = create_test_graph("shortest_path").await;
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            4,
            storage,
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Outgoing,
            None,
            Some(10), // 添加max_depth参数
            ShortestPathAlgorithm::BFS,
        );

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 4);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Outgoing
        ));
        assert!(executor.get_edge_types().is_none());
    }
}
