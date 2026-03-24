//! BFS最短路径执行器
//!
//! 使用双向广度优先搜索算法查找最短路径
//! 参考nebula-graph实现，支持双向BFS和路径拼接

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::{Edge, EdgeDirection, Path, Value, Vertex};
use crate::query::executor::base::{BaseExecutor, ExecutorConfig};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// BFS最短路径配置
pub struct BfsShortestPathConfig {
    pub steps: usize,
    pub edge_types: Vec<String>,
    pub with_cycle: bool,
    pub max_depth: Option<usize>,
    pub single_shortest: bool,
    pub limit: usize,
    pub start_vertex: Value,
    pub end_vertex: Value,
}

/// BFSShortestExecutor - BFS最短路径执行器
///
/// 使用双向广度优先搜索算法查找最短路径
/// 参考nebula-graph实现，支持双向BFS和路径拼接
pub struct BFSShortestExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    steps: usize,
    max_depth: Option<usize>,
    edge_types: Vec<String>,
    with_cycle: bool, // 是否允许回路（路径中重复访问顶点）
    with_loop: bool,  // 是否允许自环边
    single_shortest: bool,
    limit: usize,
    start_vertex: Value,
    end_vertex: Value,

    // 执行状态
    step: usize,
    left_visited_vids: HashSet<Value>,
    right_visited_vids: HashSet<Value>,
    all_left_edges: Vec<HashMap<Value, Edge>>,
    all_right_edges: Vec<HashMap<Value, Edge>>,
    current_paths: Vec<Path>,
    terminate_early: bool,

    // 统计信息
    nodes_visited: usize,
    edges_traversed: usize,
    execution_time_ms: u64,
}

impl<S: StorageClient + 'static> BFSShortestExecutor<S> {
    pub fn new(base_config: ExecutorConfig<S>, config: BfsShortestPathConfig) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "BFSShortestExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            steps: config.steps,
            max_depth: config.max_depth,
            edge_types: config.edge_types,
            with_cycle: config.with_cycle,
            with_loop: false,
            single_shortest: config.single_shortest,
            limit: config.limit,
            start_vertex: config.start_vertex,
            end_vertex: config.end_vertex,
            step: 1,
            left_visited_vids: HashSet::new(),
            right_visited_vids: HashSet::new(),
            all_left_edges: Vec::new(),
            all_right_edges: Vec::new(),
            current_paths: Vec::new(),
            terminate_early: false,
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
        }
    }

    /// 设置是否允许自环边
    pub fn with_loop(mut self, with_loop: bool) -> Self {
        self.with_loop = with_loop;
        self
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn with_cycle(&self) -> bool {
        self.with_cycle
    }

    pub fn single_shortest(&self) -> bool {
        self.single_shortest
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn nodes_visited(&self) -> usize {
        self.nodes_visited
    }

    pub fn edges_traversed(&self) -> usize {
        self.edges_traversed
    }

    pub fn execution_time_ms(&self) -> u64 {
        self.execution_time_ms
    }

    pub fn current_paths(&self) -> &[Path] {
        &self.current_paths
    }

    /// 构建路径 - 从输入中提取边并构建下一步的顶点集合
    fn build_path(
        &mut self,
        storage: &S,
        start_vids: &[Value],
        reverse: bool,
    ) -> DBResult<Vec<Value>> {
        let mut current_edges: HashMap<Value, Edge> = HashMap::new();
        let mut unique_dst: HashSet<Value> = HashSet::new();
        // 自环边去重跟踪
        let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();

        // 预留容量以提高性能
        if reverse {
            self.right_visited_vids.reserve(start_vids.len());
        } else {
            self.left_visited_vids.reserve(start_vids.len());
        }

        for start_vid in start_vids {
            // 获取当前顶点的出边或入边
            let edges = if self.edge_types.is_empty() {
                if reverse {
                    storage.get_node_edges("default", start_vid, EdgeDirection::In)?
                } else {
                    storage.get_node_edges("default", start_vid, EdgeDirection::Out)?
                }
            } else {
                // 根据边类型过滤
                let all_edges = if reverse {
                    storage.get_node_edges("default", start_vid, EdgeDirection::In)?
                } else {
                    storage.get_node_edges("default", start_vid, EdgeDirection::Out)?
                };
                // 过滤出指定类型的边
                all_edges
                    .into_iter()
                    .filter(|edge| self.edge_types.contains(&edge.edge_type))
                    .collect()
            };

            for edge in edges {
                self.edges_traversed += 1;
                let dst = if reverse {
                    (*edge.src).clone()
                } else {
                    (*edge.dst).clone()
                };

                // 检查是否是自环边
                let is_self_loop = *edge.src == *edge.dst;
                // 如果不允许自环边，进行去重
                if is_self_loop && !self.with_loop {
                    let key = (edge.edge_type.clone(), edge.ranking);
                    if !seen_self_loops.insert(key) {
                        continue; // 重复的自环边，跳过
                    }
                }

                // 检查是否已访问
                let already_visited = if reverse {
                    self.right_visited_vids.contains(&dst)
                } else {
                    self.left_visited_vids.contains(&dst)
                };

                if already_visited {
                    continue;
                }

                // 检查无环约束（路径中顶点唯一）
                if !self.with_cycle {
                    let in_path = self.left_visited_vids.contains(&dst)
                        || self.right_visited_vids.contains(&dst);
                    if in_path {
                        continue;
                    }
                }

                if unique_dst.insert(dst.clone()) {
                    current_edges.insert(dst, edge);
                }
            }
        }

        // 保存当前层的边
        if reverse {
            self.all_right_edges.push(current_edges);
        } else {
            self.all_left_edges.push(current_edges);
        }

        // 将新发现的顶点标记为已访问
        let new_vids: Vec<Value> = unique_dst.iter().cloned().collect();
        if reverse {
            self.right_visited_vids.extend(new_vids.clone());
        } else {
            self.left_visited_vids.extend(new_vids.clone());
        }

        self.nodes_visited += new_vids.len();

        Ok(new_vids)
    }

    /// 拼接路径 - 找到左右路径的交汇点并拼接成完整路径
    /// 返回是否应该提前终止搜索
    fn conjunct_paths(&mut self, current_step: usize) -> DBResult<bool> {
        if self.all_left_edges.is_empty() || self.all_right_edges.is_empty() {
            return Ok(false);
        }

        let left_edges = self
            .all_left_edges
            .last()
            .expect("Left edges should not be empty");

        // 查找交汇点
        let mut meet_vids: HashSet<Value> = HashSet::new();
        let mut odd_step = true;

        // 首先尝试与上一步的右边缘匹配
        if current_step > 1 && current_step - 2 < self.all_right_edges.len() {
            let prev_right_edges = &self.all_right_edges[current_step - 2];
            for vid in left_edges.keys() {
                if prev_right_edges.contains_key(vid) {
                    meet_vids.insert(vid.clone());
                }
            }
        }

        // 如果没有找到，尝试与当前步的右边缘匹配
        if meet_vids.is_empty() && !self.all_right_edges.is_empty() {
            odd_step = false;
            let right_edges = self
                .all_right_edges
                .last()
                .expect("Right edges should not be empty");
            for vid in left_edges.keys() {
                if right_edges.contains_key(vid) {
                    meet_vids.insert(vid.clone());
                }
            }
        }

        if meet_vids.is_empty() {
            return Ok(false);
        }

        // 为每个交汇点构建完整路径
        for meet_vid in meet_vids {
            if let Some(path) = self.create_path(&meet_vid, odd_step) {
                // 检查路径是否有重复边（路径中顶点唯一）
                if !self.with_cycle && path.has_duplicate_edges() {
                    continue;
                }

                self.current_paths.push(path);

                // 如果只找单条最短路径，找到后即可终止
                if self.single_shortest {
                    self.terminate_early = true;
                    return Ok(true);
                }

                // 检查是否达到限制
                if self.current_paths.len() >= self.limit {
                    self.terminate_early = true;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// 创建从起点到终点的完整路径
    fn create_path(&self, meet_vid: &Value, _odd_step: bool) -> Option<Path> {
        // 构建左半部分路径（从起点到交汇点）
        let left_path = self.build_half_path(meet_vid, false)?;

        // 构建右半部分路径（从终点到交汇点）
        let right_path = self.build_half_path(meet_vid, true)?;

        // 拼接路径：反转右半部分路径并追加到左半部分
        let mut full_path = left_path;

        // 反转右半部分路径的步骤
        let mut reversed_steps: Vec<crate::core::vertex_edge_path::Step> =
            right_path.steps.into_iter().rev().collect();

        // 反转每条边的方向
        for step in &mut reversed_steps {
            std::mem::swap(&mut step.edge.src, &mut step.edge.dst);
        }

        // 追加到完整路径
        full_path.steps.extend(reversed_steps);

        Some(full_path)
    }

    /// 构建半条路径
    fn build_half_path(&self, meet_vid: &Value, reverse: bool) -> Option<Path> {
        let all_edges = if reverse {
            &self.all_right_edges
        } else {
            &self.all_left_edges
        };

        if all_edges.is_empty() {
            return Some(Path::new(Vertex::new(meet_vid.clone(), vec![])));
        }

        // 从交汇点回溯到起点/终点
        let mut current_vid = meet_vid.clone();
        let mut steps: Vec<(Vertex, Edge)> = Vec::new();

        // 逆序遍历边层
        for edge_layer in all_edges.iter().rev() {
            if let Some(edge) = edge_layer.get(&current_vid) {
                // 反向搜索时，使用 edge.dst 作为下一个顶点
                let next_vid = if reverse {
                    (*edge.dst).clone()
                } else {
                    (*edge.src).clone()
                };
                steps.push((Vertex::new(next_vid.clone(), vec![]), edge.clone()));
                current_vid = next_vid;
            } else {
                break;
            }
        }

        // 构建路径
        if steps.is_empty() {
            return Some(Path::new(Vertex::new(meet_vid.clone(), vec![])));
        }

        let mut path = Path::new(steps.last()?.0.clone());
        for (vertex, edge) in steps.iter().rev() {
            path.add_step(crate::core::vertex_edge_path::Step {
                dst: Box::new(vertex.clone()),
                edge: Box::new(edge.clone()),
            });
        }

        Some(path)
    }
}

impl<S: StorageClient + 'static> Executor<S> for BFSShortestExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // 重置状态
        self.step = 1;
        self.left_visited_vids.clear();
        self.right_visited_vids.clear();
        self.all_left_edges.clear();
        self.all_right_edges.clear();
        self.current_paths.clear();
        self.terminate_early = false;

        // 初始化：将起点和终点加入已访问集合
        self.left_visited_vids.insert(self.start_vertex.clone());
        self.right_visited_vids.insert(self.end_vertex.clone());

        // 双向BFS主循环
        let max_steps = self.steps;
        let start_vertex = self.start_vertex.clone();
        let end_vertex = self.end_vertex.clone();
        let mut terminate_early = false;

        for current_step in 1..=max_steps {
            if terminate_early {
                break;
            }

            // 从起点方向扩展
            let left_vids: Vec<Value> = if current_step == 1 {
                vec![start_vertex.clone()]
            } else {
                let last_left_edges = self.all_left_edges.last();
                match last_left_edges {
                    Some(edges) => edges.keys().cloned().collect(),
                    None => Vec::new(),
                }
            };

            let right_vids: Vec<Value> = if current_step == 1 {
                vec![end_vertex.clone()]
            } else {
                let last_right_edges = self.all_right_edges.last();
                match last_right_edges {
                    Some(edges) => edges.keys().cloned().collect(),
                    None => Vec::new(),
                }
            };

            let left_has_vids = !left_vids.is_empty();
            let right_has_vids = !right_vids.is_empty();

            // 从起点方向扩展
            if left_has_vids {
                let storage = self.get_storage().clone();
                let storage_guard = storage.lock();
                self.build_path(&storage_guard, &left_vids, false)?;
            }

            // 从终点方向扩展
            if right_has_vids {
                let storage = self.get_storage().clone();
                let storage_guard = storage.lock();
                self.build_path(&storage_guard, &right_vids, true)?;
            }

            // 检查是否有交汇点并拼接路径
            let should_terminate = self.conjunct_paths(current_step)?;
            if should_terminate {
                terminate_early = true;
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;
        self.execution_time_ms = execution_time;

        let rows: Vec<Vec<Value>> = self
            .current_paths
            .clone()
            .into_iter()
            .map(|p| vec![Value::Path(p)])
            .collect();

        Ok(ExecutionResult::Values(
            rows.into_iter().flatten().collect(),
        ))
    }

    fn open(&mut self) -> DBResult<()> {
        self.step = 1;
        self.left_visited_vids.clear();
        self.right_visited_vids.clear();
        self.all_left_edges.clear();
        self.all_right_edges.clear();
        self.current_paths.clear();
        self.terminate_early = false;
        self.nodes_visited = 0;
        self.edges_traversed = 0;
        self.execution_time_ms = 0;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + 'static> HasStorage<S> for BFSShortestExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}
