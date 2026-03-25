//! Definition of the `Graph Algorithms` trait
//!
//! Define a unified interface for various graph algorithms

use super::types::AlgorithmStats;
use crate::core::{Path, Value};
use crate::query::QueryError;

/// Shortest Path Algorithm Interface
///
/// All implementations of shortest path algorithms adhere to this trait.
pub trait ShortestPathAlgorithm {
    /// Find the shortest path.
    ///
    /// # Parameters
    /// `start_ids`: List of starting vertex IDs
    /// `end_ids`: List of target vertex IDs
    /// `edge_types`: Filter by edge type (None indicates no filtering).
    /// `max_depth`: The maximum search depth (None indicates no limit).
    /// `single_shortest`: Whether to return only the shortest path.
    /// `limit`: Returns the limit on the number of paths that can be returned.
    ///
    /// # Return
    /// List of found paths
    fn find_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError>;

    /// Obtain algorithm statistics information
    fn stats(&self) -> &AlgorithmStats;

    /// Obtaining variable algorithmic statistics information
    fn stats_mut(&mut self) -> &mut AlgorithmStats;
}

/// Pathfinding algorithm interface (used to find all paths, not just the shortest one)
pub trait PathFindingAlgorithm {
    /// Find all paths.
    ///
    /// # 参数
    /// - `start_ids`: 起始顶点ID列表
    /// - `end_ids`: 目标顶点ID列表
    /// `edge_types`: Filter by edge type
    /// `max_depth`: The maximum depth of the search.
    /// - `limit`: 返回路径数量限制
    ///
    /// # 返回
    /// List of all found paths
    fn find_all_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError>;

    /// 获取算法统计信息
    fn stats(&self) -> &AlgorithmStats;
}

/// Graph Traversal Algorithm Interface
pub trait TraversalAlgorithm {
    /// Traverse a graph
    ///
    /// # 参数
    /// - `start_ids`: 起始顶点ID列表
    /// - `edge_types`: 边类型过滤
    /// `max_depth`: The maximum depth of the traversal.
    /// `limit`: Returns the limit on the number of vertices.
    ///
    /// # 返回
    /// List of vertices that have been traversed
    fn traverse(
        &mut self,
        start_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        limit: usize,
    ) -> Result<Vec<Value>, QueryError>;

    /// 获取算法统计信息
    fn stats(&self) -> &AlgorithmStats;
}

/// Algorithm context
///
/// Provide the contextual information required for the execution of the algorithm.
#[derive(Debug, Clone)]
pub struct AlgorithmContext {
    /// Maximum search depth
    pub max_depth: Option<usize>,
    /// Limit on the number of results
    pub limit: usize,
    /// Should only the shortest path be returned?
    pub single_shortest: bool,
    /// Is it allowed for loops (repeated visits to the same vertex within the path)?
    pub with_cycle: bool,
    /// Are self-loop edges (A->A) allowed?
    pub with_loop: bool,
}

impl Default for AlgorithmContext {
    fn default() -> Self {
        Self {
            max_depth: None,
            limit: usize::MAX,
            single_shortest: false,
            with_cycle: false,
            with_loop: false,
        }
    }
}

impl AlgorithmContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_single_shortest(mut self, single_shortest: bool) -> Self {
        self.single_shortest = single_shortest;
        self
    }

    pub fn with_cycle(mut self, with_cycle: bool) -> Self {
        self.with_cycle = with_cycle;
        self
    }

    pub fn with_loop(mut self, with_loop: bool) -> Self {
        self.with_loop = with_loop;
        self
    }
}
