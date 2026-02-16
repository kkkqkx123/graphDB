//! 算法模块
//!
//! 包含图相关算法实现

pub mod astar;
pub mod bfs;
pub mod bellman_ford;
pub mod bidirectional_bfs;
pub mod connected_components;
pub mod cycle_detection;
pub mod dfs;
pub mod dijkstra;
pub mod floyd_warshall;
pub mod multi_source_shortest_path;
pub mod reservoir_sampling;
pub mod strongly_connected_components;
pub mod subgraph;
pub mod topological_sort;

// 重新导出常用算法结构体
pub use astar::AStar;
pub use bfs::Bfs;
pub use bellman_ford::{BellmanFord, BellmanFordResult};
pub use bidirectional_bfs::BidirectionalBfs;
pub use connected_components::ConnectedComponents;
pub use cycle_detection::CycleDetection;
pub use dfs::Dfs;
pub use dijkstra::Dijkstra;
pub use floyd_warshall::{FloydWarshall, FloydWarshallResult};
pub use multi_source_shortest_path::{MultiSourceShortestPath, PathResult};
pub use reservoir_sampling::{GraphSampling, ReservoirSampling, ReservoirSamplingAlgo};
pub use strongly_connected_components::StronglyConnectedComponents;
pub use subgraph::{EdgeDirection, SubgraphExtractor, SubgraphResult};
pub use topological_sort::TopologicalSort;
