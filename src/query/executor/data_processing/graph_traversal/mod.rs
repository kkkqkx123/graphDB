//! 图遍历执行器模块
//!
//! 包含所有与图遍历相关的执行器，包括：
//! - 单步扩展（Expand）
//! - 全路径扩展（ExpandAll）
//! - 完整遍历（Traverse）
//! - 最短路径（ShortestPath）
//! - 所有路径（AllPaths）
//! - 子图提取（Subgraph）

pub mod expand;
pub mod expand_all;
pub mod shortest_path;
pub mod traverse;

// 重新导出主要类型
pub use expand::ExpandExecutor;
pub use expand_all::ExpandAllExecutor;
pub use shortest_path::{ShortestPathAlgorithm, ShortestPathExecutor};
pub use traverse::TraverseExecutor;

/// 图遍历执行器的通用特征
pub trait GraphTraversalExecutor<S: crate::storage::StorageEngine> {
    /// 设置边方向
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection);

    /// 设置边类型过滤
    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>);

    /// 设置最大深度
    fn set_max_depth(&mut self, max_depth: Option<usize>);

    /// 获取当前边方向
    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection;

    /// 获取当前边类型过滤
    fn get_edge_types(&self) -> &Option<Vec<String>>;

    /// 获取当前最大深度
    fn get_max_depth(&self) -> &Option<usize>;
}

/// 为所有图遍历执行器实现通用特征
impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for ExpandExecutor<S> {
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection) {
        self.edge_direction = direction;
    }

    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
        self.edge_types = edge_types;
    }

    fn set_max_depth(&mut self, max_depth: Option<usize>) {
        self.max_depth = max_depth;
    }

    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection {
        &self.edge_direction
    }

    fn get_edge_types(&self) -> &Option<Vec<String>> {
        &self.edge_types
    }

    fn get_max_depth(&self) -> &Option<usize> {
        &self.max_depth
    }
}

impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for ExpandAllExecutor<S> {
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection) {
        self.edge_direction = direction;
    }

    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
        self.edge_types = edge_types;
    }

    fn set_max_depth(&mut self, max_depth: Option<usize>) {
        self.max_depth = max_depth;
    }

    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection {
        &self.edge_direction
    }

    fn get_edge_types(&self) -> &Option<Vec<String>> {
        &self.edge_types
    }

    fn get_max_depth(&self) -> &Option<usize> {
        &self.max_depth
    }
}

impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for TraverseExecutor<S> {
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection) {
        self.edge_direction = direction;
    }

    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
        self.edge_types = edge_types;
    }

    fn set_max_depth(&mut self, max_depth: Option<usize>) {
        self.max_depth = max_depth;
    }

    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection {
        &self.edge_direction
    }

    fn get_edge_types(&self) -> &Option<Vec<String>> {
        &self.edge_types
    }

    fn get_max_depth(&self) -> &Option<usize> {
        &self.max_depth
    }
}

impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for ShortestPathExecutor<S> {
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection) {
        self.edge_direction = direction;
    }

    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
        self.edge_types = edge_types;
    }

    fn set_max_depth(&mut self, _max_depth: Option<usize>) {
        // 对于最短路径，最大深度可以用来限制搜索范围
        // 这里我们将其存储起来，但具体实现可能需要调整
    }

    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection {
        &self.edge_direction
    }

    fn get_edge_types(&self) -> &Option<Vec<String>> {
        &self.edge_types
    }

    fn get_max_depth(&self) -> &Option<usize> {
        // ShortestPathExecutor 不直接使用 max_depth，但为了接口一致性返回None
        &None
    }
}

/// 图遍历执行器工厂
pub struct GraphTraversalExecutorFactory;

impl GraphTraversalExecutorFactory {
    /// 创建ExpandExecutor
    pub fn create_expand_executor<S: crate::storage::StorageEngine>(
        id: usize,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> ExpandExecutor<S> {
        ExpandExecutor::new(id, storage, edge_direction, edge_types, max_depth)
    }

    /// 创建ExpandAllExecutor
    pub fn create_expand_all_executor<S: crate::storage::StorageEngine + std::marker::Send>(
        id: usize,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> ExpandAllExecutor<S> {
        ExpandAllExecutor::new(id, storage, edge_direction, edge_types, max_depth)
    }

    /// 创建TraverseExecutor
    pub fn create_traverse_executor<S: crate::storage::StorageEngine>(
        id: usize,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        conditions: Option<String>,
    ) -> TraverseExecutor<S> {
        TraverseExecutor::new(
            id,
            storage,
            edge_direction,
            edge_types,
            max_depth,
            conditions,
        )
    }

    /// 创建ShortestPathExecutor
    pub fn create_shortest_path_executor<S: crate::storage::StorageEngine>(
        id: usize,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        start_vertex_ids: Vec<crate::core::Value>,
        end_vertex_ids: Vec<crate::core::Value>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
        algorithm: ShortestPathAlgorithm,
    ) -> ShortestPathExecutor<S> {
        ShortestPathExecutor::new(
            id,
            storage,
            start_vertex_ids,
            end_vertex_ids,
            edge_direction,
            edge_types,
            algorithm,
        )
    }
}

#[cfg(test)]
mod tests_impl {
    use super::*;
    use crate::config::test_config::test_config;
    use crate::core::{Edge, Value, Vertex};
    use crate::query::executor::base::{EdgeDirection, Executor};
    use crate::storage::{NativeStorage, StorageEngine};
    use std::sync::{Arc, Mutex};

    async fn create_test_graph(test_name: &str) -> Arc<Mutex<NativeStorage>> {
        let config = test_config();
        let db_path = config.test_db_path(&format!("test_graph_{}", test_name));
        let storage = Arc::new(Mutex::new(NativeStorage::new(db_path).unwrap()));

        // 创建测试图：A -> B -> C, A -> D
        {
            let mut storage_lock = storage.lock().unwrap();

            // 创建顶点
            let vertex_a = Vertex::new(Value::String("A".to_string()), vec![]);
            let vertex_b = Vertex::new(Value::String("B".to_string()), vec![]);
            let vertex_c = Vertex::new(Value::String("C".to_string()), vec![]);
            let vertex_d = Vertex::new(Value::String("D".to_string()), vec![]);

            let id_a = storage_lock.insert_node(vertex_a).unwrap();
            let id_b = storage_lock.insert_node(vertex_b).unwrap();
            let id_c = storage_lock.insert_node(vertex_c).unwrap();
            let id_d = storage_lock.insert_node(vertex_d).unwrap();

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

            storage_lock.insert_edge(edge_ab).unwrap();
            storage_lock.insert_edge(edge_bc).unwrap();
            storage_lock.insert_edge(edge_ad).unwrap();
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
        assert!(matches!(executor.get_edge_direction(), EdgeDirection::Out));
        assert!(executor.get_edge_types().is_some());
        assert_eq!(executor.get_max_depth(), &Some(1));
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
        assert_eq!(executor.get_max_depth(), &Some(2));
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
        assert!(matches!(executor.get_edge_direction(), EdgeDirection::Out));
        assert!(executor.get_edge_types().is_some());
        assert_eq!(executor.get_max_depth(), &Some(3));
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
            ShortestPathAlgorithm::BFS,
        );

        assert_eq!(executor.name(), "ShortestPathExecutor");
        assert_eq!(executor.id(), 4);
        assert!(matches!(executor.get_edge_direction(), EdgeDirection::Out));
        assert!(executor.get_edge_types().is_none());
    }
}
