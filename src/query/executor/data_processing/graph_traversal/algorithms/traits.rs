//! 图算法 trait 定义
//!
//! 定义各种图算法的统一接口

use crate::core::{Path, Value};
use crate::query::QueryError;
use super::types::AlgorithmStats;

/// 最短路径算法接口
///
/// 所有最短路径算法实现此 trait
pub trait ShortestPathAlgorithm {
    /// 查找最短路径
    ///
    /// # 参数
    /// - `start_ids`: 起始顶点ID列表
    /// - `end_ids`: 目标顶点ID列表
    /// - `edge_types`: 边类型过滤（None表示不过滤）
    /// - `max_depth`: 最大搜索深度（None表示无限制）
    /// - `single_shortest`: 是否只返回一条最短路径
    /// - `limit`: 返回路径数量限制
    ///
    /// # 返回
    /// 找到的路径列表
    fn find_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError>;

    /// 获取算法统计信息
    fn stats(&self) -> &AlgorithmStats;

    /// 获取可变的算法统计信息
    fn stats_mut(&mut self) -> &mut AlgorithmStats;
}

/// 路径查找算法接口（用于查找所有路径，不只是最短路径）
pub trait PathFindingAlgorithm {
    /// 查找所有路径
    ///
    /// # 参数
    /// - `start_ids`: 起始顶点ID列表
    /// - `end_ids`: 目标顶点ID列表
    /// - `edge_types`: 边类型过滤
    /// - `max_depth`: 最大搜索深度
    /// - `limit`: 返回路径数量限制
    ///
    /// # 返回
    /// 找到的所有路径列表
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

/// 图遍历算法接口
pub trait TraversalAlgorithm {
    /// 遍历图
    ///
    /// # 参数
    /// - `start_ids`: 起始顶点ID列表
    /// - `edge_types`: 边类型过滤
    /// - `max_depth`: 最大遍历深度
    /// - `limit`: 返回顶点数量限制
    ///
    /// # 返回
    /// 遍历到的顶点列表
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

/// 算法上下文
///
/// 提供算法执行所需的上下文信息
#[derive(Debug, Clone)]
pub struct AlgorithmContext {
    /// 最大搜索深度
    pub max_depth: Option<usize>,
    /// 结果数量限制
    pub limit: usize,
    /// 是否只返回单条最短路径
    pub single_shortest: bool,
    /// 是否检测环路
    pub no_loop: bool,
}

impl Default for AlgorithmContext {
    fn default() -> Self {
        Self {
            max_depth: None,
            limit: usize::MAX,
            single_shortest: false,
            no_loop: true,
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

    pub fn with_no_loop(mut self, no_loop: bool) -> Self {
        self.no_loop = no_loop;
        self
    }
}
