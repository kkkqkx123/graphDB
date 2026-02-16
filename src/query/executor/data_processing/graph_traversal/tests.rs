#[cfg(test)]
mod tests {
    use crate::core::{Edge, Value, Vertex};
    use crate::query::executor::base::EdgeDirection;
    use crate::query::executor::data_processing::graph_traversal::algorithms::{EdgeWeightConfig, HeuristicFunction, ShortestPathAlgorithmType};
    use crate::query::executor::data_processing::graph_traversal::factory::GraphTraversalExecutorFactory;
    use crate::query::executor::data_processing::graph_traversal::traits::GraphTraversalExecutor;
    use crate::query::executor::traits::Executor;
    use crate::storage::{MockStorage, StorageClient};
    use std::sync::Arc;
use parking_lot::Mutex;

    async fn create_test_graph(_test_name: &str) -> Arc<Mutex<MockStorage>> {
        let storage = Arc::new(Mutex::new(MockStorage));
        let space = "default";

        // 创建测试图：A -> B -> C, A -> D
        {
            let mut storage_lock = storage.lock();

            // 创建顶点
            let vertex_a = Vertex::new(Value::String("A".to_string()), vec![]);
            let vertex_b = Vertex::new(Value::String("B".to_string()), vec![]);
            let vertex_c = Vertex::new(Value::String("C".to_string()), vec![]);
            let vertex_d = Vertex::new(Value::String("D".to_string()), vec![]);

            let id_a = storage_lock
                .insert_vertex(space, vertex_a)
                .expect("Failed to insert test vertex A");
            let id_b = storage_lock
                .insert_vertex(space, vertex_b)
                .expect("Failed to insert test vertex B");
            let id_c = storage_lock
                .insert_vertex(space, vertex_c)
                .expect("Failed to insert test vertex C");
            let id_d = storage_lock
                .insert_vertex(space, vertex_d)
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
                .insert_edge(space, edge_ab)
                .expect("Failed to insert test edge AB");
            storage_lock
                .insert_edge(space, edge_bc)
                .expect("Failed to insert test edge BC");
            storage_lock
                .insert_edge(space, edge_ad)
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
            EdgeDirection::Out,
            Some(vec!["connect".to_string()]),
            Some(1),
        );

        // 测试基本功能
        assert_eq!(executor.name(), "ExpandExecutor");
        assert_eq!(executor.id(), 1);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Out
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
            EdgeDirection::Out,
            Some(vec!["connect".to_string()]),
            Some(3),
            Some("true".to_string()),
        );

        assert_eq!(executor.name(), "TraverseExecutor");
        assert_eq!(executor.id(), 3);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Out
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
            EdgeDirection::Out,
            None,
            Some(10), // 添加max_depth参数
            ShortestPathAlgorithmType::BFS,
        );

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 4);
        assert!(matches!(
            executor.get_edge_direction(),
            EdgeDirection::Out
        ));
        assert!(executor.get_edge_types().is_none());
    }

    /// 创建带权测试图
    /// 图结构: A --(weight: 1)--> B --(weight: 2)--> C
    ///         \--(weight: 5)--> D --(weight: 1)--> C
    /// 最短路径(按权重): A->B->C (总权重: 3)
    /// 最短路径(按步数): A->B->C 或 A->D->C (都是2步)
    async fn create_weighted_test_graph(_test_name: &str) -> Arc<Mutex<MockStorage>> {
        let storage = Arc::new(Mutex::new(MockStorage));
        let space = "default";

        {
            let mut storage_lock = storage.lock();

            // 创建顶点
            let vertex_a = Vertex::new(Value::String("A".to_string()), vec![]);
            let vertex_b = Vertex::new(Value::String("B".to_string()), vec![]);
            let vertex_c = Vertex::new(Value::String("C".to_string()), vec![]);
            let vertex_d = Vertex::new(Value::String("D".to_string()), vec![]);

            let id_a = storage_lock
                .insert_vertex(space, vertex_a)
                .expect("Failed to insert test vertex A");
            let id_b = storage_lock
                .insert_vertex(space, vertex_b)
                .expect("Failed to insert test vertex B");
            let id_c = storage_lock
                .insert_vertex(space, vertex_c)
                .expect("Failed to insert test vertex C");
            let id_d = storage_lock
                .insert_vertex(space, vertex_d)
                .expect("Failed to insert test vertex D");

            // 创建带权边
            let mut props_ab = std::collections::HashMap::new();
            props_ab.insert("weight".to_string(), Value::Int(1));
            let edge_ab = Edge::new(
                id_a.clone(),
                id_b.clone(),
                "connect".to_string(),
                1, // ranking also set to 1 for testing
                props_ab,
            );

            let mut props_bc = std::collections::HashMap::new();
            props_bc.insert("weight".to_string(), Value::Int(2));
            let edge_bc = Edge::new(
                id_b.clone(),
                id_c.clone(),
                "connect".to_string(),
                2,
                props_bc,
            );

            let mut props_ad = std::collections::HashMap::new();
            props_ad.insert("weight".to_string(), Value::Int(5));
            let edge_ad = Edge::new(
                id_a.clone(),
                id_d.clone(),
                "connect".to_string(),
                5,
                props_ad,
            );

            let mut props_dc = std::collections::HashMap::new();
            props_dc.insert("weight".to_string(), Value::Int(1));
            let edge_dc = Edge::new(
                id_d.clone(),
                id_c.clone(),
                "connect".to_string(),
                1,
                props_dc,
            );

            storage_lock
                .insert_edge(space, edge_ab)
                .expect("Failed to insert test edge AB");
            storage_lock
                .insert_edge(space, edge_bc)
                .expect("Failed to insert test edge BC");
            storage_lock
                .insert_edge(space, edge_ad)
                .expect("Failed to insert test edge AD");
            storage_lock
                .insert_edge(space, edge_dc)
                .expect("Failed to insert test edge DC");
        }

        storage
    }

    #[tokio::test]
    async fn test_weighted_shortest_path_with_property() {
        let storage = create_weighted_test_graph("weighted_shortest_path_prop").await;

        // 使用属性权重创建执行器
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            5,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::Dijkstra,
        ).with_weight_config(EdgeWeightConfig::Property("weight".to_string()));

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 5);
    }

    #[tokio::test]
    async fn test_weighted_shortest_path_with_ranking() {
        let storage = create_weighted_test_graph("weighted_shortest_path_ranking").await;

        // 使用ranking作为权重创建执行器
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            6,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::Dijkstra,
        ).with_weight_config(EdgeWeightConfig::Ranking);

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 6);
    }

    #[tokio::test]
    async fn test_unweighted_shortest_path() {
        let storage = create_weighted_test_graph("unweighted_shortest_path").await;

        // 使用无权图配置创建执行器
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            7,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::BFS,
        ).with_weight_config(EdgeWeightConfig::Unweighted);

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 7);
    }

    // 创建带坐标属性的测试图，用于A*算法测试
    async fn create_spatial_test_graph(_test_name: &str) -> Arc<Mutex<MockStorage>> {
        let storage = Arc::new(Mutex::new(MockStorage));
        let space = "default";

        // 创建空间测试图：A(0,0) -> B(3,4) -> C(6,8), A -> D(1,1) -> C
        // 使用欧几里得距离作为启发式
        {
            let mut storage_lock = storage.lock();

            // 创建带坐标属性的顶点
            let mut props_a = std::collections::HashMap::new();
            props_a.insert("lat".to_string(), Value::Float(0.0));
            props_a.insert("lon".to_string(), Value::Float(0.0));
            let vertex_a = Vertex::new_with_properties(Value::String("A".to_string()), vec![], props_a);

            let mut props_b = std::collections::HashMap::new();
            props_b.insert("lat".to_string(), Value::Float(3.0));
            props_b.insert("lon".to_string(), Value::Float(4.0));
            let vertex_b = Vertex::new_with_properties(Value::String("B".to_string()), vec![], props_b);

            let mut props_c = std::collections::HashMap::new();
            props_c.insert("lat".to_string(), Value::Float(6.0));
            props_c.insert("lon".to_string(), Value::Float(8.0));
            let vertex_c = Vertex::new_with_properties(Value::String("C".to_string()), vec![], props_c);

            let mut props_d = std::collections::HashMap::new();
            props_d.insert("lat".to_string(), Value::Float(1.0));
            props_d.insert("lon".to_string(), Value::Float(1.0));
            let vertex_d = Vertex::new_with_properties(Value::String("D".to_string()), vec![], props_d);

            let id_a = storage_lock
                .insert_vertex(space, vertex_a)
                .expect("Failed to insert test vertex A");
            let id_b = storage_lock
                .insert_vertex(space, vertex_b)
                .expect("Failed to insert test vertex B");
            let id_c = storage_lock
                .insert_vertex(space, vertex_c)
                .expect("Failed to insert test vertex C");
            let id_d = storage_lock
                .insert_vertex(space, vertex_d)
                .expect("Failed to insert test vertex D");

            // 创建带权重的边
            let mut props_ab = std::collections::HashMap::new();
            props_ab.insert("weight".to_string(), Value::Int(5));
            let edge_ab = Edge::new(
                id_a.clone(),
                id_b.clone(),
                "connect".to_string(),
                5,
                props_ab,
            );

            let mut props_bc = std::collections::HashMap::new();
            props_bc.insert("weight".to_string(), Value::Int(5));
            let edge_bc = Edge::new(
                id_b.clone(),
                id_c.clone(),
                "connect".to_string(),
                5,
                props_bc,
            );

            let mut props_ad = std::collections::HashMap::new();
            props_ad.insert("weight".to_string(), Value::Int(2));
            let edge_ad = Edge::new(
                id_a.clone(),
                id_d.clone(),
                "connect".to_string(),
                2,
                props_ad,
            );

            let mut props_dc = std::collections::HashMap::new();
            props_dc.insert("weight".to_string(), Value::Int(8));
            let edge_dc = Edge::new(
                id_d.clone(),
                id_c.clone(),
                "connect".to_string(),
                8,
                props_dc,
            );

            storage_lock
                .insert_edge(space, edge_ab)
                .expect("Failed to insert test edge AB");
            storage_lock
                .insert_edge(space, edge_bc)
                .expect("Failed to insert test edge BC");
            storage_lock
                .insert_edge(space, edge_ad)
                .expect("Failed to insert test edge AD");
            storage_lock
                .insert_edge(space, edge_dc)
                .expect("Failed to insert test edge DC");
        }

        storage
    }

    #[tokio::test]
    async fn test_astar_with_spatial_heuristic() {
        let storage = create_spatial_test_graph("astar_spatial").await;

        // 使用A*算法，带空间启发式
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            8,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::AStar,
        )
        .with_weight_config(EdgeWeightConfig::Property("weight".to_string()))
        .with_heuristic_config(HeuristicFunction::PropertyDistance("lat".to_string(), "lon".to_string()));

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 8);
    }

    #[tokio::test]
    async fn test_astar_without_heuristic() {
        let storage = create_spatial_test_graph("astar_no_heuristic").await;

        // 使用A*算法，但无启发式（退化为Dijkstra）
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            9,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::AStar,
        )
        .with_weight_config(EdgeWeightConfig::Property("weight".to_string()))
        .with_heuristic_config(HeuristicFunction::Zero);

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 9);
    }

    #[tokio::test]
    async fn test_astar_with_scale_heuristic() {
        let storage = create_spatial_test_graph("astar_scale").await;

        // 使用A*算法，带固定缩放因子启发式
        let executor = GraphTraversalExecutorFactory::create_shortest_path_executor(
            10,
            storage.clone(),
            vec![Value::String("A".to_string())],
            vec![Value::String("C".to_string())],
            EdgeDirection::Out,
            None,
            Some(10),
            ShortestPathAlgorithmType::AStar,
        )
        .with_weight_config(EdgeWeightConfig::Property("weight".to_string()))
        .with_heuristic_config(HeuristicFunction::ScaleFactor(0.5));

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 10);
    }
}
