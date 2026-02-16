//! 图算法共享类型定义
//!
//! 包含各种图算法使用的共享数据结构

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::core::{Edge, NPath, Path, Value};

/// 自环边去重辅助结构
/// 用于在遍历过程中跟踪已处理的自环边
#[derive(Debug, Default)]
pub struct SelfLoopDedup {
    seen: HashSet<(String, i64)>,
}

impl SelfLoopDedup {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    /// 检查并记录自环边
    /// 返回 true 表示该边应该被包含（首次出现）
    /// 返回 false 表示该边应该被跳过（重复的自环边）
    pub fn should_include(&mut self, edge: &Edge) -> bool {
        let is_self_loop = *edge.src == *edge.dst;
        if is_self_loop {
            let key = (edge.edge_type.clone(), edge.ranking);
            self.seen.insert(key)
        } else {
            true
        }
    }
}

/// Dijkstra距离节点
#[derive(Debug, Clone)]
pub struct DistanceNode {
    pub distance: f64,
    pub vertex_id: Value,
}

impl Eq for DistanceNode {}

impl PartialEq for DistanceNode {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.vertex_id == other.vertex_id
    }
}

impl Ord for DistanceNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.distance.partial_cmp(&self.distance).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for DistanceNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// 双向BFS状态
#[derive(Debug, Clone)]
pub struct BidirectionalBFSState {
    /// 使用 NPath 替代 Path 存储中间结果，减少内存复制
    pub left_queue: VecDeque<(Value, Arc<NPath>)>,
    pub right_queue: VecDeque<(Value, Arc<NPath>)>,
    /// 使用 NPath 缓存访问过的路径
    pub left_visited: HashMap<Value, (Arc<NPath>, f64)>,
    pub right_visited: HashMap<Value, (Arc<NPath>, f64)>,
    pub left_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>>,
    pub right_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>>,
}

impl BidirectionalBFSState {
    pub fn new() -> Self {
        Self {
            left_queue: VecDeque::new(),
            right_queue: VecDeque::new(),
            left_visited: HashMap::new(),
            right_visited: HashMap::new(),
            left_edges: Vec::new(),
            right_edges: Vec::new(),
        }
    }
}

impl Default for BidirectionalBFSState {
    fn default() -> Self {
        Self::new()
    }
}

/// 算法统计信息
#[derive(Debug, Clone, Default)]
pub struct AlgorithmStats {
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub execution_time_ms: u64,
}

impl AlgorithmStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_nodes_visited(&mut self) {
        self.nodes_visited += 1;
    }

    pub fn increment_edges_traversed(&mut self, count: usize) {
        self.edges_traversed += count;
    }

    pub fn set_execution_time(&mut self, time_ms: u64) {
        self.execution_time_ms = time_ms;
    }
}

/// 最短路径算法类型
#[derive(Debug, Clone)]
pub enum ShortestPathAlgorithmType {
    BFS,
    Dijkstra,
    AStar,
}

/// 路径拼接工具函数
/// 左路径从起点到中间，右路径从终点到中间
pub fn combine_npaths(left: &Arc<NPath>, right: &Arc<NPath>) -> Option<Path> {
    // 检查两条路径是否在同一个顶点交汇
    if left.vertex().vid.as_ref() != right.vertex().vid.as_ref() {
        return None;
    }

    // 构建从左起点到交汇点的路径
    let left_path = left.to_path();

    // 构建从右起点到交汇点的路径，然后反转
    let mut right_path = right.to_path();
    right_path.reverse();

    // 合并两条路径
    let mut combined = left_path;
    combined.steps.extend(right_path.steps);

    Some(combined)
}

/// 检查路径是否有重复边
pub fn has_duplicate_edges(path: &Path) -> bool {
    let mut edge_set = HashSet::new();

    for step in &path.steps {
        let edge = &step.edge;
        let edge_key = format!("{}_{}_{}", edge.src, edge.dst, edge.ranking);
        if !edge_set.insert(edge_key) {
            return true;
        }
    }

    false
}
