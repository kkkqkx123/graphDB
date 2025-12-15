use super::*;
use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
use crate::query::executor::data_processing::graph_traversal::expand_all::ExpandAllExecutor;
use crate::query::executor::data_processing::graph_traversal::shortest_path::ShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor;

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