//! 多源最短路径算法
//!
//! 支持多组起点和终点同时查找最短路径

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::core::error::{DBError, DBResult};
use crate::query::executor::base::{BaseExecutor, EdgeDirection, Executor as BaseExecutorTrait, ExecutorStats, HasStorage, InputExecutor, ExecutionResult, DBResult as ExecDBResult};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::{AlgorithmStats, Interims, TerminationMap, create_termination_map, cleanup_termination_map, mark_path_found, is_termination_complete, SelfLoopDedup};

/// 多源最短路径执行器
///
/// 同时处理多组(src, dst)路径查找请求
/// 使用双向BFS算法，支持单条/多条最短路径
pub struct MultiShortestPathExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    start_vids: Vec<Value>,
    end_vids: Vec<Value>,
    termination_map: TerminationMap,
    edge_direction: EdgeDirection,
    edge_types: Option<Vec<String>>,
    max_steps: usize,
    single_shortest: bool,
    limit: usize,
    step: usize,
    /// 左向历史路径
    history_left_paths: Interims,
    /// 右向历史路径
    history_right_paths: Interims,
    /// 当前左向路径
    left_paths: Interims,
    /// 当前右向路径
    right_paths: Interims,
    /// 上一步右向路径（用于奇数步交汇）
    pre_right_paths: Interims,
    /// 结果路径
    result_paths: Vec<Path>,
    /// 统计信息
    stats: AlgorithmStats,
    /// 输入执行器（用于获取边数据）
    left_input: Option<Box<ExecutorEnum<S>>>,
    right_input: Option<Box<ExecutorEnum<S>>>,
    /// 已找到路径计数
    found_count: usize,
}

impl<S: StorageClient> std::fmt::Debug for MultiShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("start_vids", &self.start_vids)
            .field("end_vids", &self.end_vids)
            .field("max_steps", &self.max_steps)
            .field("single_shortest", &self.single_shortest)
            .field("limit", &self.limit)
            .field("step", &self.step)
            .field("result_paths", &self.result_paths.len())
            .finish()
    }
}

impl<S: StorageClient> MultiShortestPathExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vids: Vec<Value>,
        end_vids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_steps: usize,
    ) -> Self {
        let termination_map = create_termination_map(&start_vids, &end_vids);
        
        Self {
            base: BaseExecutor::new(id, "MultiShortestPathExecutor".to_string(), storage),
            start_vids,
            end_vids,
            termination_map,
            edge_direction,
            edge_types,
            max_steps,
            single_shortest: false,
            limit: usize::MAX,
            step: 1,
            history_left_paths: HashMap::new(),
            history_right_paths: HashMap::new(),
            left_paths: HashMap::new(),
            right_paths: HashMap::new(),
            pre_right_paths: HashMap::new(),
            result_paths: Vec::new(),
            stats: AlgorithmStats::new(),
            left_input: None,
            right_input: None,
            found_count: 0,
        }
    }

    pub fn with_limits(mut self, single_shortest: bool, limit: usize) -> Self {
        self.single_shortest = single_shortest;
        self.limit = limit;
        self
    }

    pub fn with_inputs(
        mut self,
        left_input: Box<ExecutorEnum<S>>,
        right_input: Box<ExecutorEnum<S>>,
    ) -> Self {
        self.left_input = Some(left_input);
        self.right_input = Some(right_input);
        self
    }

    /// 初始化历史路径
    fn init(&mut self) {
        // 初始化左向历史路径
        for src in &self.start_vids {
            let path = Path::new(Vertex::with_vid(src.clone()));
            let mut src_map = HashMap::new();
            src_map.insert(src.clone(), vec![path.clone()]);
            self.history_left_paths.insert(src.clone(), src_map);
        }

        // 初始化右向历史路径
        for dst in &self.end_vids {
            let path = Path::new(Vertex::with_vid(dst.clone()));
            let mut dst_map = HashMap::new();
            dst_map.insert(dst.clone(), vec![path.clone()]);
            self.history_right_paths.insert(dst.clone(), dst_map.clone());
            self.pre_right_paths.insert(dst.clone(), dst_map);
        }
    }

    /// 从存储获取邻居边
    fn get_neighbors(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> DBResult<Vec<(Value, Edge)>> {
        let storage = self.base.storage.as_ref()
            .ok_or_else(|| DBError::Storage(
                crate::core::error::StorageError::DbError("Storage not set".to_string())
            ))?;
        let storage = storage.lock();

        let edges = storage
            .get_node_edges("default", node_id, direction)
            .map_err(|e| DBError::Storage(
                crate::core::error::StorageError::DbError(e.to_string())
            ))?;

        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        // 自环边去重
        let mut dedup = SelfLoopDedup::new();

        let neighbors = filtered_edges
            .into_iter()
            .filter(|edge| dedup.should_include(edge))
            .filter_map(|edge| match direction {
                EdgeDirection::In => {
                    if *edge.dst == *node_id {
                        Some(((*edge.src).clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Out => {
                    if *edge.src == *node_id {
                        Some(((*edge.dst).clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Both => {
                    if *edge.src == *node_id {
                        Some(((*edge.dst).clone(), edge))
                    } else if *edge.dst == *node_id {
                        Some(((*edge.src).clone(), edge))
                    } else {
                        None
                    }
                }
            })
            .collect();

        Ok(neighbors)
    }

    /// 创建新路径（扩展已有路径）
    fn create_paths(paths: &[Path], edge: &Edge) -> Vec<Path> {
        paths
            .iter()
            .map(|p| {
                let mut new_path = p.clone();
                let dst_vertex = Vertex::with_vid(edge.dst.as_ref().clone());
                new_path.steps.push(Step::new(
                    dst_vertex,
                    edge.edge_type.clone(),
                    edge.edge_type.clone(),
                    edge.ranking,
                ));
                new_path
            })
            .collect()
    }

    /// 构建路径（从左或右输入）
    fn build_path(&mut self, reverse: bool) -> DBResult<()> {
        let history_paths = if reverse {
            &self.history_right_paths
        } else {
            &self.history_left_paths
        };

        // 获取需要扩展的顶点
        let expand_vids: Vec<Value> = if self.step == 1 {
            if reverse {
                self.end_vids.clone()
            } else {
                self.start_vids.clone()
            }
        } else {
            history_paths.keys().cloned().collect()
        };

        // 先收集所有邻居信息，避免借用冲突
        let mut all_neighbors: Vec<(Value, Vec<(Value, Edge)>)> = Vec::new();
        for vid in &expand_vids {
            let neighbors = self.get_neighbors(vid, self.edge_direction)?;
            self.stats.increment_edges_traversed(neighbors.len());
            all_neighbors.push((vid.clone(), neighbors));
        }

        // 处理收集到的邻居信息
        for (vid, neighbors) in all_neighbors {
            for (neighbor_id, edge) in neighbors {
                // 跳过自环
                if neighbor_id == vid {
                    continue;
                }

                let current_paths = if reverse {
                    &mut self.right_paths
                } else {
                    &mut self.left_paths
                };

                if self.step == 1 {
                    // 第一步：创建初始路径
                    let src_vertex = Vertex::with_vid(vid.clone());
                    let dst_vertex = Vertex::with_vid(neighbor_id.clone());
                    let path = Path {
                        src: Box::new(src_vertex),
                        steps: vec![Step::new(
                            dst_vertex,
                            edge.edge_type.clone(),
                            edge.edge_type.clone(),
                            edge.ranking,
                        )],
                    };

                    let entry = current_paths.entry(neighbor_id.clone()).or_insert_with(HashMap::new);
                    let src_paths = entry.entry(vid.clone()).or_insert_with(Vec::new);
                    src_paths.push(path);
                } else {
                    // 后续步骤：从历史路径扩展
                    if let Some(pre_paths) = history_paths.get(&vid) {
                        for (src_id, paths) in pre_paths {
                            // 检查是否形成环路
                            if let Some(history_dst) = history_paths.get(&neighbor_id) {
                                if history_dst.contains_key(src_id) {
                                    continue; // 环路检测
                                }
                            }

                            let new_paths = Self::create_paths(paths, &edge);
                            
                            let entry = current_paths.entry(neighbor_id.clone()).or_insert_with(HashMap::new);
                            let src_paths = entry.entry(src_id.clone()).or_insert_with(Vec::new);
                            src_paths.extend(new_paths);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 路径交汇（奇数步或偶数步）
    fn conjunct_path(&mut self, odd_step: bool) -> DBResult<bool> {
        let right_paths = if odd_step {
            &self.pre_right_paths
        } else {
            &self.right_paths
        };

        // 收集需要处理的路径对，避免借用冲突
        let mut path_pairs: Vec<(Value, Value, Vec<Path>, Vec<Path>)> = Vec::new();

        // 查找交汇点
        for (meet_vid, left_src_map) in &self.left_paths {
            if let Some(right_src_map) = right_paths.get(meet_vid) {
                // 在交汇点找到匹配
                for (left_src, left_paths) in left_src_map {
                    for (right_src, right_paths) in right_src_map {
                        // 检查是否是有效的(src, dst)对
                        if self.is_valid_pair(left_src, right_src) {
                            path_pairs.push((
                                left_src.clone(),
                                right_src.clone(),
                                left_paths.clone(),
                                right_paths.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // 处理收集到的路径对
        for (left_src, right_src, left_paths, right_paths) in path_pairs {
            self.build_result_paths(&left_paths, &right_paths, &left_src, &right_src)?;
            
            if self.single_shortest {
                mark_path_found(&mut self.termination_map, &left_src, &right_src);
            }
        }

        // 清理已找到的路径对
        if self.single_shortest {
            cleanup_termination_map(&mut self.termination_map);
        }

        // 检查是否终止
        if is_termination_complete(&self.termination_map) {
            return Ok(true);
        }

        if self.found_count >= self.limit {
            return Ok(true);
        }

        // 检查步数限制
        if self.step * 2 > self.max_steps {
            return Ok(true);
        }

        Ok(false)
    }

    /// 检查是否是有效的(src, dst)对
    fn is_valid_pair(&self, src: &Value, dst: &Value) -> bool {
        if let Some(pairs) = self.termination_map.get(src) {
            pairs.iter().any(|(d, found)| d == dst && *found)
        } else {
            false
        }
    }

    /// 构建结果路径
    fn build_result_paths(
        &mut self,
        left_paths: &[Path],
        right_paths: &[Path],
        _src: &Value,
        _dst: &Value,
    ) -> DBResult<()> {
        for left_path in left_paths {
            for right_path in right_paths {
                // 拼接路径
                let mut full_path = left_path.clone();
                let mut reversed_right = right_path.clone();
                reversed_right.reverse();
                
                // 合并步骤
                full_path.steps.extend(reversed_right.steps);

                // 检查重复边
                if self.has_duplicate_edges(&full_path) {
                    continue;
                }

                self.result_paths.push(full_path);
                self.found_count += 1;

                if self.found_count >= self.limit {
                    return Ok(());
                }

                if self.single_shortest {
                    // 单条最短路径模式下，找到一对就停止
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// 检查路径是否有重复边
    fn has_duplicate_edges(&self, path: &Path) -> bool {
        let mut edge_set = HashSet::new();
        
        for step in &path.steps {
            let edge_key = format!(
                "{}_{}_{}",
                step.src_vid(),
                step.dst_vid(),
                step.ranking()
            );
            if !edge_set.insert(edge_key) {
                return true;
            }
        }
        
        false
    }

    /// 更新历史路径
    fn update_history(&mut self) {
        // 将当前左向路径合并到历史
        for (dst, src_map) in &self.left_paths {
            let history_entry = self.history_left_paths.entry(dst.clone()).or_insert_with(HashMap::new);
            for (src, paths) in src_map {
                let src_entry = history_entry.entry(src.clone()).or_insert_with(Vec::new);
                src_entry.extend(paths.clone());
            }
        }

        // 将当前右向路径合并到历史
        for (dst, src_map) in &self.right_paths {
            let history_entry = self.history_right_paths.entry(dst.clone()).or_insert_with(HashMap::new);
            for (src, paths) in src_map {
                let src_entry = history_entry.entry(src.clone()).or_insert_with(Vec::new);
                src_entry.extend(paths.clone());
            }
        }

        // 保存当前右向路径供下一步使用
        self.pre_right_paths = self.right_paths.clone();
        
        // 清空当前路径
        self.left_paths.clear();
        self.right_paths.clear();
    }

    /// 执行多源最短路径查找
    pub fn execute_multi_path(&mut self) -> DBResult<Vec<Path>> {
        let start_time = Instant::now();
        
        self.init();

        loop {
            // 构建左向路径
            self.build_path(false)?;
            
            // 构建右向路径
            self.build_path(true)?;

            // 奇数步交汇
            if self.conjunct_path(true)? {
                break;
            }

            // 偶数步交汇
            if self.conjunct_path(false)? {
                break;
            }

            // 更新历史路径
            self.update_history();
            
            self.step += 1;

            // 检查步数限制
            if self.step * 2 > self.max_steps {
                break;
            }
        }

        self.stats.set_execution_time(start_time.elapsed().as_millis() as u64);
        
        Ok(self.result_paths.clone())
    }
}

impl<S: StorageClient + Send + 'static> BaseExecutorTrait<S> for MultiShortestPathExecutor<S> {
    fn execute(&mut self) -> ExecDBResult<ExecutionResult> {
        let paths = self.execute_multi_path()
            .map_err(|e| crate::core::error::DBError::Query(
                crate::query::QueryError::ExecutionError(e.to_string())
            ))?;
        
        Ok(ExecutionResult::Paths(paths))
    }

    fn open(&mut self) -> ExecDBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> ExecDBResult<()> {
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
        "Multi-source shortest path executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for MultiShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for MultiShortestPathExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        if self.left_input.is_none() {
            self.left_input = Some(Box::new(input));
        } else if self.right_input.is_none() {
            self.right_input = Some(Box::new(input));
        }
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.left_input.as_ref().map(|b| b.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Value, Vertex, Path};
    use crate::storage::MockStorage;

    #[test]
    fn test_termination_map_creation() {
        let start_vids = vec![Value::from("a"), Value::from("b")];
        let end_vids = vec![Value::from("c"), Value::from("d")];
        
        let map = create_termination_map(&start_vids, &end_vids);
        
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&Value::from("a")));
        assert!(map.contains_key(&Value::from("b")));
        
        let a_pairs = map.get(&Value::from("a")).unwrap();
        assert_eq!(a_pairs.len(), 2);
    }

    #[test]
    fn test_mark_path_found() {
        let start_vids = vec![Value::from("a")];
        let end_vids = vec![Value::from("b")];
        
        let mut map = create_termination_map(&start_vids, &end_vids);
        
        assert!(mark_path_found(&mut map, &Value::from("a"), &Value::from("b")));
        
        let pairs = map.get(&Value::from("a")).unwrap();
        assert!(!pairs[0].1); // found应该被标记为false
    }

    #[test]
    fn test_cleanup_termination_map() {
        let start_vids = vec![Value::from("a")];
        let end_vids = vec![Value::from("b"), Value::from("c")];
        
        let mut map = create_termination_map(&start_vids, &end_vids);
        mark_path_found(&mut map, &Value::from("a"), &Value::from("b"));
        cleanup_termination_map(&mut map);
        
        assert_eq!(map.len(), 1);
        let pairs = map.get(&Value::from("a")).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, Value::from("c"));
    }

    #[test]
    fn test_create_paths() {
        let path = Path::new(Vertex::with_vid(Value::from("a")));
        
        let edge = Edge::new(
            Value::from("a"),
            Value::from("b"),
            "edge".to_string(),
            0,
            HashMap::new(),
        );
        
        let new_paths = MultiShortestPathExecutor::<MockStorage>::create_paths(&[path], &edge);
        
        assert_eq!(new_paths.len(), 1);
        assert_eq!(new_paths[0].steps.len(), 1);
    }

    #[test]
    fn test_has_duplicate_edges() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let executor = MultiShortestPathExecutor::new(
            1,
            storage,
            vec![Value::from("a")],
            vec![Value::from("d")],
            EdgeDirection::Out,
            None,
            10,
        );

        // 创建路径
        let path = Path {
            src: Box::new(Vertex::with_vid(Value::from("a"))),
            steps: vec![
                Step::new(
                    Vertex::with_vid(Value::from("b")),
                    "e".to_string(),
                    "e".to_string(),
                    0,
                ),
                Step::new(
                    Vertex::with_vid(Value::from("c")),
                    "e".to_string(),
                    "e".to_string(),
                    0,
                ),
            ],
        };
        
        // 这个测试需要实际的边数据，简化处理
        // 实际测试应该在集成测试中进行
        assert!(!executor.has_duplicate_edges(&path));
    }
}
