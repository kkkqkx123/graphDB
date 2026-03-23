//! 图算法模块
//!
//! 包含各种图遍历和路径查找算法的实现
//!
//! # 算法列表
//! - `a_star`: A*启发式搜索算法
//! - `bidirectional_bfs`: 双向BFS最短路径算法
//! - `bfs_shortest`: BFS最短路径执行器
//! - `dijkstra`: Dijkstra最短路径算法
//! - `multi_shortest_path`: 多源最短路径算法
//! - `subgraph_executor`: 子图查询执行器

pub mod a_star;
pub mod bfs_shortest;
pub mod bidirectional_bfs;
pub mod dijkstra;
pub mod multi_shortest_path;
pub mod subgraph_executor;
pub mod traits;
pub mod types;

// 重新导出算法类型
pub use a_star::AStar;
pub use bfs_shortest::BFSShortestExecutor;
pub use bidirectional_bfs::BidirectionalBFS;
pub use dijkstra::Dijkstra;
pub use multi_shortest_path::MultiShortestPathExecutor;
pub use subgraph_executor::{SubgraphConfig, SubgraphExecutor, SubgraphResult};
pub use traits::{
    AlgorithmContext, PathFindingAlgorithm, ShortestPathAlgorithm, TraversalAlgorithm,
};
pub use types::{
    cleanup_termination_map, combine_npaths, create_termination_map, has_duplicate_edges,
    is_termination_complete, mark_path_found, AlgorithmStats, BidirectionalBFSState, DistanceNode,
    EdgeWeightConfig, HeuristicFunction, Interims, MultiPathRequest, SelfLoopDedup,
    ShortestPathAlgorithmType, TerminationMap,
};
