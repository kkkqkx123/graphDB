use crate::query::executor::base::EdgeDirection;
use crate::storage::StorageEngine;

/// 图遍历执行器的通用特征
///
/// 这个trait为图遍历执行器提供统一的配置接口，
/// 所有图遍历执行器都应该实现这个trait来提供一致的配置管理
pub trait GraphTraversalExecutor<S: StorageEngine> {
    /// 设置边方向
    fn set_edge_direction(&mut self, direction: EdgeDirection);

    /// 设置边类型过滤
    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>);

    /// 设置最大深度
    fn set_max_depth(&mut self, max_depth: Option<usize>);

    /// 获取当前边方向
    fn get_edge_direction(&self) -> EdgeDirection;

    /// 获取当前边类型过滤
    fn get_edge_types(&self) -> Option<Vec<String>>;

    /// 获取当前最大深度
    fn get_max_depth(&self) -> Option<usize>;

    /// 验证执行器配置是否有效
    fn validate_config(&self) -> Result<(), String>;

    /// 获取执行器统计信息
    fn get_stats(&self) -> TraversalStats;
}

/// 图遍历统计信息
#[derive(Debug, Clone)]
pub struct TraversalStats {
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub execution_time_ms: u64,
    pub max_depth_reached: usize,
}

impl Default for TraversalStats {
    fn default() -> Self {
        Self {
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
            max_depth_reached: 0,
        }
    }
}
