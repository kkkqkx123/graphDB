//! 图遍历执行器测试模块

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Value, Vertex, Edge, Tag};
    use crate::storage::native_storage::NativeStorage;
    use crate::query::executor::base::{Executor, ExecutionResult, EdgeDirection};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    /// 创建测试图数据
    async fn create_test_graph() -> Arc<Mutex<NativeStorage>> {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_graph_traversal").unwrap()));
        
        let mut storage_lock = storage.lock().unwrap();
        
        // 创建测试标签
        let person_tag = Tag::new("person".to_string(), HashMap::new());
        let city_tag = Tag::new("city".to_string(), HashMap::new());
        
        // 创建顶点
        let alice = Vertex::new(Value::String("alice".to_string()), vec![person_tag.clone()]);
        let bob = Vertex::new(Value::String("bob".to_string()), vec![person_tag.clone()]);
        let charlie = Vertex::new(Value::String("charlie".to_string()), vec![person_tag.clone()]);
        let david = Vertex::new(Value::String("david".to_string()), vec![person_tag.clone()]);
        let beijing = Vertex::new(Value::String("beijing".to_string()), vec![city_tag.clone()]);
        let shanghai = Vertex::new(Value::String("shanghai".to_string()), vec![city_tag.clone()]);
        
        // 插入顶点
        let alice_id = storage_lock.insert_node(alice).unwrap();
        let bob_id = storage_lock.insert_node(bob).unwrap();
        let charlie_id = storage_lock.insert_node(charlie).unwrap();
        let david_id = storage_lock.insert_node(david).unwrap();
        let beijing_id = storage_lock.insert_node(beijing).unwrap();
        let shanghai_id = storage_lock.insert_node(shanghai).unwrap();
        
        // 创建边
        let alice_bob = Edge::new(alice_id.clone(), bob_id.clone(), "knows".to_string(), 1, HashMap::new());
        let bob_charlie = Edge::new(bob_id.clone(), charlie_id.clone(), "knows".to_string(), 1, HashMap::new());
        let charlie_david = Edge::new(charlie_id.clone(), david_id.clone(), "knows".to_string(), 1, HashMap::new());
        let alice_david = Edge::new(alice_id.clone(), david_id.clone(), "knows".to_string(), 2, HashMap::new());
        
        let alice_beijing = Edge::new(alice_id.clone(), beijing_id.clone(), "lives_in".to_string(), 1, HashMap::new());
        let bob_shanghai = Edge::new(bob_id.clone(), shanghai_id.clone(), "lives_in".to_string(), 1, HashMap::new());
        let charlie_beijing = Edge::new(charlie_id.clone(), beijing_id.clone(), "lives_in".to_string(), 1, HashMap::new());
        
        // 插入边
        storage_lock.insert_edge(alice_bob).unwrap();
        storage_lock.insert_edge(bob_charlie).unwrap();
        storage_lock.insert_edge(charlie_david).unwrap();
        storage_lock.insert_edge(alice_david).unwrap();
        storage_lock.insert_edge(alice_beijing).unwrap();
        storage_lock.insert_edge(bob_shanghai).unwrap();
        storage_lock.insert_edge(charlie_beijing).unwrap();
        
        storage
    }

    #[tokio::test]
    async fn test_expand_executor_basic() {
        let storage = create_test_graph().await;
        let mut executor = super::ExpandExecutor::new(
            1,
            storage.clone(),
            EdgeDirection::Out,
            Some(vec!["knows".to_string()]),
            Some(1),
        );
        
        // 创建输入执行器，返回alice节点
        struct MockInputExecutor {
            storage: Arc<Mutex<NativeStorage>>,
        }
        
        #[async_trait::async_trait]
        impl Executor<NativeStorage> for MockInputExecutor {
            async fn execute(&mut self) -> Result<ExecutionResult, crate::query::QueryError> {
                let storage = self.storage.lock().unwrap();
                if let Some(alice) = storage.get_node(&Value::String("alice".to_string())).unwrap() {
                    Ok(ExecutionResult::Vertices(vec![alice]))
                } else {
                    Ok(ExecutionResult::Vertices(vec![]))
                }
            }
            
            fn open(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn close(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn id(&self) -> usize { 0 }
            fn name(&self) -> &str { "MockInputExecutor" }
        }
        
        let mut input_executor = MockInputExecutor { storage: storage.clone() };
        executor.set_input(Box::new(input_executor));
        
        // 执行扩展
        executor.open().unwrap();
        let result = executor.execute().await.unwrap();
        executor.close().unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Vertices(vertices) => {
                // alice的knows邻居应该是bob和david
                assert_eq!(vertices.len(), 2);
                let vertex_names: Vec<String> = vertices.iter()
                    .map(|v| match &*v.vid {
                        Value::String(name) => name.clone(),
                        _ => panic!("Expected string vertex ID"),
                    })
                    .collect();
                assert!(vertex_names.contains(&"bob".to_string()));
                assert!(vertex_names.contains(&"david".to_string()));
            }
            _ => panic!("Expected vertices result"),
        }
    }

    #[tokio::test]
    async fn test_expand_all_executor_basic() {
        let storage = create_test_graph().await;
        let mut executor = super::ExpandAllExecutor::new(
            2,
            storage.clone(),
            EdgeDirection::Out,
            Some(vec!["knows".to_string()]),
            Some(2),
        );
        
        // 创建输入执行器，返回alice节点
        struct MockInputExecutor {
            storage: Arc<Mutex<NativeStorage>>,
        }
        
        #[async_trait::async_trait]
        impl Executor<NativeStorage> for MockInputExecutor {
            async fn execute(&mut self) -> Result<ExecutionResult, crate::query::QueryError> {
                let storage = self.storage.lock().unwrap();
                if let Some(alice) = storage.get_node(&Value::String("alice".to_string())).unwrap() {
                    Ok(ExecutionResult::Vertices(vec![alice]))
                } else {
                    Ok(ExecutionResult::Vertices(vec![]))
                }
            }
            
            fn open(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn close(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn id(&self) -> usize { 0 }
            fn name(&self) -> &str { "MockInputExecutor" }
        }
        
        let mut input_executor = MockInputExecutor { storage: storage.clone() };
        executor.set_input(Box::new(input_executor));
        
        // 执行扩展
        executor.open().unwrap();
        let result = executor.execute().await.unwrap();
        executor.close().unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Values(paths) => {
                // 应该有多条路径
                assert!(!paths.is_empty());
                
                // 检查路径结构
                for path_value in &paths {
                    match path_value {
                        Value::List(path_steps) => {
                            // 路径应该包含起始节点和至少一个边
                            assert!(!path_steps.is_empty());
                            assert!(path_steps.len() >= 3); // 至少：节点、边、节点
                        }
                        _ => panic!("Expected list value for path"),
                    }
                }
            }
            _ => panic!("Expected values result"),
        }
    }

    #[tokio::test]
    async fn test_traverse_executor_basic() {
        let storage = create_test_graph().await;
        let mut executor = super::TraverseExecutor::new(
            3,
            storage.clone(),
            EdgeDirection::Out,
            Some(vec!["knows".to_string()]),
            Some(2),
            None,
        );
        
        // 创建输入执行器，返回alice节点
        struct MockInputExecutor {
            storage: Arc<Mutex<NativeStorage>>,
        }
        
        #[async_trait::async_trait]
        impl Executor<NativeStorage> for MockInputExecutor {
            async fn execute(&mut self) -> Result<ExecutionResult, crate::query::QueryError> {
                let storage = self.storage.lock().unwrap();
                if let Some(alice) = storage.get_node(&Value::String("alice".to_string())).unwrap() {
                    Ok(ExecutionResult::Vertices(vec![alice]))
                } else {
                    Ok(ExecutionResult::Vertices(vec![]))
                }
            }
            
            fn open(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn close(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn id(&self) -> usize { 0 }
            fn name(&self) -> &str { "MockInputExecutor" }
        }
        
        let mut input_executor = MockInputExecutor { storage: storage.clone() };
        executor.set_input(Box::new(input_executor));
        
        // 执行遍历
        executor.open().unwrap();
        let result = executor.execute().await.unwrap();
        executor.close().unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Values(paths) => {
                // 应该有遍历路径
                assert!(!paths.is_empty());
                
                // 检查路径结构
                for path_value in &paths {
                    match path_value {
                        Value::List(path_steps) => {
                            // 路径应该包含起始节点和边
                            assert!(!path_steps.is_empty());
                        }
                        _ => panic!("Expected list value for path"),
                    }
                }
            }
            _ => panic!("Expected values result"),
        }
    }

    #[tokio::test]
    async fn test_shortest_path_executor_bfs() {
        let storage = create_test_graph().await;
        let mut executor = super::ShortestPathExecutor::new(
            4,
            storage.clone(),
            vec![Value::String("alice".to_string())],
            vec![Value::String("charlie".to_string())],
            EdgeDirection::Out,
            Some(vec!["knows".to_string()]),
            super::ShortestPathAlgorithm::BFS,
        );
        
        // 执行最短路径计算
        executor.open().unwrap();
        let result = executor.execute().await.unwrap();
        executor.close().unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Values(paths) => {
                // 应该找到最短路径 alice -> bob -> charlie
                assert!(!paths.is_empty());
                
                // 检查路径结构
                for path_value in &paths {
                    match path_value {
                        Value::List(path_steps) => {
                            // 路径应该包含：alice, edge, bob, edge, charlie
                            assert_eq!(path_steps.len(), 5);
                            
                            // 检查起始节点
                            match &path_steps[0] {
                                Value::Vertex(vertex) => {
                                    match &*vertex.vid {
                                        Value::String(name) => assert_eq!(name, "alice"),
                                        _ => panic!("Expected string vertex ID"),
                                    }
                                }
                                _ => panic!("Expected vertex"),
                            }
                        }
                        _ => panic!("Expected list value for path"),
                    }
                }
            }
            _ => panic!("Expected values result"),
        }
    }

    #[tokio::test]
    async fn test_shortest_path_executor_dijkstra() {
        let storage = create_test_graph().await;
        let mut executor = super::ShortestPathExecutor::new(
            5,
            storage.clone(),
            vec![Value::String("alice".to_string())],
            vec![Value::String("david".to_string())],
            EdgeDirection::Out,
            Some(vec!["knows".to_string()]),
            super::ShortestPathAlgorithm::Dijkstra,
        );
        
        // 执行最短路径计算
        executor.open().unwrap();
        let result = executor.execute().await.unwrap();
        executor.close().unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Values(paths) => {
                // 应该找到最短路径 alice -> david (权重2) 而不是 alice -> bob -> charlie -> david (权重3)
                assert!(!paths.is_empty());
                
                // 检查路径结构
                for path_value in &paths {
                    match path_value {
                        Value::List(path_steps) => {
                            // 直接路径应该只有3个元素：alice, edge, david
                            assert_eq!(path_steps.len(), 3);
                        }
                        _ => panic!("Expected list value for path"),
                    }
                }
            }
            _ => panic!("Expected values result"),
        }
    }

    #[tokio::test]
    async fn test_edge_directions() {
        let storage = create_test_graph().await;
        
        // 测试出边方向
        let mut executor_out = super::ExpandExecutor::new(
            6,
            storage.clone(),
            EdgeDirection::Out,
            None,
            Some(1),
        );
        
        // 测试入边方向
        let mut executor_in = super::ExpandExecutor::new(
            7,
            storage.clone(),
            EdgeDirection::In,
            None,
            Some(1),
        );
        
        // 测试双向
        let mut executor_both = super::ExpandExecutor::new(
            8,
            storage.clone(),
            EdgeDirection::Both,
            None,
            Some(1),
        );
        
        // 创建输入执行器，返回beijing节点
        struct MockInputExecutor {
            storage: Arc<Mutex<NativeStorage>>,
        }
        
        #[async_trait::async_trait]
        impl Executor<NativeStorage> for MockInputExecutor {
            async fn execute(&mut self) -> Result<ExecutionResult, crate::query::QueryError> {
                let storage = self.storage.lock().unwrap();
                if let Some(beijing) = storage.get_node(&Value::String("beijing".to_string())).unwrap() {
                    Ok(ExecutionResult::Vertices(vec![beijing]))
                } else {
                    Ok(ExecutionResult::Vertices(vec![]))
                }
            }
            
            fn open(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn close(&mut self) -> Result<(), crate::query::QueryError> { Ok(()) }
            fn id(&self) -> usize { 0 }
            fn name(&self) -> &str { "MockInputExecutor" }
        }
        
        let mut input_executor = MockInputExecutor { storage: storage.clone() };
        
        // 测试出边方向（beijing作为lives_in边的目标，没有出边）
        executor_out.set_input(Box::new(input_executor));
        executor_out.open().unwrap();
        let result_out = executor_out.execute().await.unwrap();
        executor_out.close().unwrap();
        
        match &result_out {
            ExecutionResult::Vertices(vertices) => {
                assert_eq!(vertices.len(), 0); // beijing没有出边
            }
            _ => panic!("Expected vertices result"),
        }
        
        // 测试入边方向（beijing作为lives_in边的目标，有入边）
        let mut input_executor2 = MockInputExecutor { storage: storage.clone() };
        executor_in.set_input(Box::new(input_executor2));
        executor_in.open().unwrap();
        let result_in = executor_in.execute().await.unwrap();
        executor_in.close().unwrap();
        
        match &result_in {
            ExecutionResult::Vertices(vertices) => {
                assert!(vertices.len() > 0); // beijing有入边
            }
            _ => panic!("Expected vertices result"),
        }
    }
}