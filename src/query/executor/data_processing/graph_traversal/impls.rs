use super::*;
use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
use crate::query::executor::data_processing::graph_traversal::expand_all::ExpandAllExecutor;
use crate::query::executor::data_processing::graph_traversal::shortest_path::ShortestPathExecutor;
use crate::query::executor::data_processing::graph_traversal::traits::TraversalStats;
use crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor;

/// 宏定义：为图遍历执行器实现通用特征
macro_rules! impl_graph_traversal_executor {
    ($executor:ty) => {
        impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for $executor {
            fn set_edge_direction(
                &mut self,
                direction: crate::query::executor::base::EdgeDirection,
            ) {
                self.edge_direction = direction;
            }

            fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
                self.edge_types = edge_types;
            }

            fn set_max_depth(&mut self, max_depth: Option<usize>) {
                self.max_depth = max_depth;
            }

            fn get_edge_direction(&self) -> crate::query::executor::base::EdgeDirection {
                self.edge_direction.clone()
            }

            fn get_edge_types(&self) -> Option<Vec<String>> {
                self.edge_types.clone()
            }

            fn get_max_depth(&self) -> Option<usize> {
                self.max_depth
            }

            fn validate_config(&self) -> Result<(), String> {
                // 基本配置验证
                if let Some(max_depth) = self.max_depth {
                    if max_depth == 0 {
                        return Err("最大深度不能为0".to_string());
                    }
                }
                Ok(())
            }

            fn get_stats(&self) -> TraversalStats {
                // 默认统计信息
                TraversalStats::default()
            }
        }
    };
}

// 为所有图遍历执行器实现通用特征
impl_graph_traversal_executor!(ExpandExecutor<S>);

// 使用宏为其他执行器实现通用特征
impl_graph_traversal_executor!(ExpandAllExecutor<S>);
impl_graph_traversal_executor!(TraverseExecutor<S>);

/// 为ShortestPathExecutor提供特殊实现
impl<S: crate::storage::StorageEngine> GraphTraversalExecutor<S> for ShortestPathExecutor<S> {
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection) {
        self.edge_direction = direction;
    }

    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>) {
        self.edge_types = edge_types;
    }

    fn set_max_depth(&mut self, max_depth: Option<usize>) {
        // 最短路径算法支持最大深度限制
        // 这可以用于限制搜索范围，避免无限循环
        self.max_depth = max_depth;
    }

    fn get_edge_direction(&self) -> crate::query::executor::base::EdgeDirection {
        self.edge_direction.clone()
    }

    fn get_edge_types(&self) -> Option<Vec<String>> {
        self.edge_types.clone()
    }

    fn get_max_depth(&self) -> Option<usize> {
        // 返回实际设置的最大深度
        self.max_depth
    }

    fn validate_config(&self) -> Result<(), String> {
        // 最短路径的特殊验证
        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err("最短路径的最大深度不能为0".to_string());
            }
        }

        // 验证算法选择是否有效
        // 注意：algorithm字段是私有的，这里暂时跳过验证
        // 在实际实现中，应该通过公共方法来获取算法类型

        Ok(())
    }

    fn get_stats(&self) -> TraversalStats {
        // 提供最短路径特定的统计信息
        TraversalStats {
            nodes_visited: self.nodes_visited,
            edges_traversed: self.edges_traversed,
            execution_time_ms: self.execution_time_ms,
            max_depth_reached: self.max_depth_reached,
        }
    }
}
