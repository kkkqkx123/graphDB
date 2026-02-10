use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::core::error::DBError;
use crate::core::{Edge, EdgeDirection, NullType, Path, Value, Vertex};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use crate::utils::safe_lock;

use crate::expression::context::traits::VariableContext;

/// FulltextIndexScanExecutor - 全文索引扫描执行器
///
/// 用于执行全文索引扫描操作，基于索引名称查找匹配的顶点
pub struct FulltextIndexScanExecutor<S: StorageClient + Send + Sync + 'static> {
    base: BaseExecutor<S>,
    space_id: i32,
    index_name: String,
    query: String,
    limit: Option<usize>,
    is_edge: bool,
    schema_id: i32,
}

impl<S: StorageClient + Send + Sync + 'static> FulltextIndexScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_id: i32,
        index_name: &str,
        query: &str,
        limit: Option<usize>,
        is_edge: bool,
        schema_id: i32,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FulltextIndexScanExecutor".to_string(), storage),
            space_id,
            index_name: index_name.to_string(),
            query: query.to_string(),
            limit,
            is_edge,
            schema_id,
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn limit(&self) -> Option<usize> {
        self.limit
    }

    pub fn is_edge(&self) -> bool {
        self.is_edge
    }

    pub fn schema_id(&self) -> i32 {
        self.schema_id
    }

    /// 执行全文索引搜索
    /// 首先获取索引配置，然后使用存储层的lookup_index方法查找匹配的顶点ID
    fn search_fulltext_index(&self, storage: &S) -> DBResult<Vec<(Value, f32)>> {
        // 获取空间名称
        let space_name = self.get_space_name(storage)?;

        // 使用存储层的全文索引查找功能
        // 这里返回的是匹配文档的ID列表和相似度分数
        let index_results = storage.lookup_index(&space_name, &self.index_name, &Value::String(self.query.clone()))
            .map_err(|e| DBError::Storage(e))?;

        // 将结果转换为 (Value, score) 格式
        // 注意：当前存储层返回的是Value列表，我们需要扩展它以支持分数
        // 这里暂时使用默认分数1.0
        let results: Vec<(Value, f32)> = index_results
            .into_iter()
            .map(|v| (v, 1.0f32))
            .collect();

        Ok(results)
    }

    /// 根据space_id获取空间名称
    fn get_space_name(&self, _storage: &S) -> DBResult<String> {
        // 尝试通过ID查找空间
        // 由于StorageClient没有直接通过ID获取空间的方法，我们使用默认空间
        // 在实际实现中，应该添加通过ID获取空间的方法
        Ok("default".to_string())
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for FulltextIndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("FulltextIndexScanExecutor storage lock should not be poisoned");

        // 执行全文索引搜索
        let search_results = self.search_fulltext_index(&*storage)?;

        // 应用限制
        let limited_results: Vec<(Value, f32)> = if let Some(limit) = self.limit {
            search_results.into_iter().take(limit).collect()
        } else {
            search_results
        };

        // 获取空间名称用于后续查询
        let space_name = self.get_space_name(&*storage)?;

        // 根据是否为边类型，构建不同的返回结果
        let rows: Vec<Vec<Value>> = if self.is_edge {
            // 边类型：返回边对象和分数
            limited_results
                .into_iter()
                .filter_map(|(id, score)| {
                    // 尝试解析边ID (格式: src_dst_ranking 或复杂结构)
                    // 这里简化处理，假设ID可以直接用于查找
                    if let Value::String(edge_key) = &id {
                        // 尝试解析边键 (src:dst:ranking格式)
                        let parts: Vec<&str> = edge_key.split(':').collect();
                        if parts.len() >= 2 {
                            let src = Value::String(parts[0].to_string());
                            let dst = Value::String(parts[1].to_string());
                            let edge_type = self.get_schema_name(&*storage).ok()?;

                            if let Ok(Some(edge)) = storage.get_edge(&space_name, &src, &dst, &edge_type) {
                                return Some(vec![Value::Edge(edge), Value::Float(score as f64)]);
                            }
                        }
                    }
                    // 如果无法解析为边，返回ID和分数
                    Some(vec![id, Value::Float(score as f64)])
                })
                .collect()
        } else {
            // 顶点类型：返回顶点对象和分数
            limited_results
                .into_iter()
                .filter_map(|(id, score)| {
                    if let Ok(Some(vertex)) = storage.get_vertex(&space_name, &id) {
                        Some(vec![Value::Vertex(Box::new(vertex)), Value::Float(score as f64)])
                    } else {
                        // 如果无法获取完整顶点，返回ID和分数
                        Some(vec![id, Value::Float(score as f64)])
                    }
                })
                .collect()
        };

        Ok(ExecutionResult::Values(
            rows.into_iter().flatten().collect()
        ))
    }

    fn open(&mut self) -> DBResult<()> {
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

impl<S: StorageClient + Send + 'static> FulltextIndexScanExecutor<S> {
    /// 根据schema_id获取schema名称
    fn get_schema_name(&self, _storage: &S) -> DBResult<String> {
        // 在实际实现中，应该通过schema_id查询元数据获取名称
        // 这里简化处理
        Ok(format!("schema_{}", self.schema_id))
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for FulltextIndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
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
    no_loop: bool,
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
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        steps: usize,
        edge_types: Vec<String>,
        no_loop: bool,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
        start_vertex: Value,
        end_vertex: Value,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "BFSShortestExecutor".to_string(), storage),
            steps,
            max_depth,
            edge_types,
            no_loop,
            single_shortest,
            limit,
            start_vertex,
            end_vertex,
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

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn no_loop(&self) -> bool {
        self.no_loop
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
                // 根据边类型过滤 - 简化实现，不过滤
                if reverse {
                    storage.get_node_edges("default", start_vid, EdgeDirection::In)?
                } else {
                    storage.get_node_edges("default", start_vid, EdgeDirection::Out)?
                }
            };

            for edge in edges {
                self.edges_traversed += 1;
                let dst = if reverse {
                    (*edge.src).clone()
                } else {
                    (*edge.dst).clone()
                };

                // 检查是否已访问
                let already_visited = if reverse {
                    self.right_visited_vids.contains(&dst)
                } else {
                    self.left_visited_vids.contains(&dst)
                };
                
                if already_visited {
                    continue;
                }

                // 检查无环约束
                if self.no_loop {
                    let in_path = self.left_visited_vids.contains(&dst) || self.right_visited_vids.contains(&dst);
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

        let left_edges = self.all_left_edges.last().expect("Left edges should not be empty");
        
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
            let right_edges = self.all_right_edges.last().expect("Right edges should not be empty");
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
                // 检查路径是否有重复边
                if self.no_loop && path.has_duplicate_edges() {
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
        let mut reversed_steps: Vec<crate::core::vertex_edge_path::Step> = right_path.steps.into_iter().rev().collect();
        
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
                let storage_guard = storage.lock().map_err(|_| {
                    DBError::Storage(crate::core::error::StorageError::DbError(
                        "Failed to lock storage".to_string(),
                    ))
                })?;
                self.build_path(&storage_guard, &left_vids, false)?;
            }

            // 从终点方向扩展
            if right_has_vids {
                let storage = self.get_storage().clone();
                let storage_guard = storage.lock().map_err(|_| {
                    DBError::Storage(crate::core::error::StorageError::DbError(
                        "Failed to lock storage".to_string(),
                    ))
                })?;
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
            rows.into_iter().flatten().collect()
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

/// IndexScanExecutor - 索引扫描执行器
///
/// 用于执行基于索引的扫描操作
pub struct IndexScanExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    space_id: i32,
    tag_id: i32,
    index_id: i32,
    scan_type: String,
    scan_limits: Vec<super::super::planner::plan::algorithms::IndexLimit>,
    filter: Option<crate::core::Expression>,
    return_columns: Vec<String>,
    limit: Option<usize>,
    is_edge: bool,
}

impl<S: StorageClient> IndexScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_id: i32,
        tag_id: i32,
        index_id: i32,
        scan_type: &str,
        scan_limits: Vec<super::super::planner::plan::algorithms::IndexLimit>,
        filter: Option<crate::core::Expression>,
        return_columns: Vec<String>,
        limit: Option<usize>,
        is_edge: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "IndexScanExecutor".to_string(), storage),
            space_id,
            tag_id,
            index_id,
            scan_type: scan_type.to_string(),
            scan_limits,
            filter,
            return_columns,
            limit,
            is_edge,
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn index_id(&self) -> i32 {
        self.index_id
    }

    pub fn scan_type(&self) -> &str {
        &self.scan_type
    }

    pub fn scan_limits(&self) -> &[super::super::planner::plan::algorithms::IndexLimit] {
        &self.scan_limits
    }

    pub fn return_columns(&self) -> &[String] {
        &self.return_columns
    }

    pub fn is_edge(&self) -> bool {
        self.is_edge
    }

    /// 获取空间名称
    fn get_space_name(&self, _storage: &S) -> DBResult<String> {
        // 简化实现，实际应该通过space_id查询
        Ok("default".to_string())
    }

    /// 获取schema名称（tag或edge类型名称）
    fn get_schema_name(&self, _storage: &S) -> DBResult<String> {
        if self.is_edge {
            // 通过edge_type ID获取名称
            // 简化实现
            Ok(format!("edge_type_{}", self.tag_id.abs()))
        } else {
            // 通过tag ID获取名称
            // 简化实现
            Ok(format!("tag_{}", self.tag_id))
        }
    }

    /// 执行索引查找
    fn lookup_by_index(&self, storage: &S) -> DBResult<Vec<Value>> {
        let space_name = self.get_space_name(storage)?;
        let index_name = format!("index_{}", self.index_id);

        // 使用存储层的索引查找功能
        // 根据scan_type选择不同的查找策略
        match self.scan_type.as_str() {
            "UNIQUE" => {
                // 唯一索引查找
                if let Some(first_limit) = self.scan_limits.first() {
                    let value = first_limit.begin_value.as_ref()
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Null(NullType::Null));
                    storage.lookup_index(&space_name, &index_name, &value)
                        .map_err(|e| DBError::Storage(e))
                } else {
                    Ok(Vec::new())
                }
            }
            "PREFIX" => {
                // 前缀索引查找
                if let Some(first_limit) = self.scan_limits.first() {
                    let prefix = first_limit.begin_value.as_ref()
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Null(NullType::Null));
                    storage.lookup_index(&space_name, &index_name, &prefix)
                        .map_err(|e| DBError::Storage(e))
                } else {
                    Ok(Vec::new())
                }
            }
            "RANGE" => {
                // 范围索引查找
                // 这里简化处理，实际应该支持范围查询
                if let Some(first_limit) = self.scan_limits.first() {
                    let start_value = first_limit.begin_value.as_ref()
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Null(NullType::Null));
                    storage.lookup_index(&space_name, &index_name, &start_value)
                        .map_err(|e| DBError::Storage(e))
                } else {
                    Ok(Vec::new())
                }
            }
            _ => {
                // 默认扫描所有
                Ok(Vec::new())
            }
        }
    }

    /// 根据ID列表获取完整顶点或边
    fn fetch_entities(&self, storage: &S, ids: Vec<Value>) -> DBResult<Vec<Value>> {
        let space_name = self.get_space_name(storage)?;
        let schema_name = self.get_schema_name(storage)?;

        let mut results = Vec::new();

        for id in ids {
            if self.is_edge {
                // 边类型：ID格式应该是 src_dst_ranking
                if let Value::String(edge_key) = &id {
                    let parts: Vec<&str> = edge_key.split(':').collect();
                    if parts.len() >= 2 {
                        let src = Value::String(parts[0].to_string());
                        let dst = Value::String(parts[1].to_string());
                        if let Some(edge) = storage.get_edge(&space_name, &src, &dst, &schema_name)
                            .map_err(|e| DBError::Storage(e))? {
                            results.push(Value::Edge(edge));
                        }
                    }
                }
            } else {
                // 顶点类型
                if let Some(vertex) = storage.get_vertex(&space_name, &id)
                    .map_err(|e| DBError::Storage(e))? {
                    results.push(Value::Vertex(Box::new(vertex)));
                }
            }
        }

        Ok(results)
    }

    /// 应用过滤器
    fn apply_filter(&self, entities: Vec<Value>) -> Vec<Value> {
        if let Some(ref filter_expr) = self.filter {
            let mut context = crate::expression::DefaultExpressionContext::new();
            entities
                .into_iter()
                .filter(|entity| {
                    VariableContext::set_variable(&mut context, "entity".to_string(), entity.clone());
                    match crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expr, &mut context) {
                        Ok(value) => match &value {
                            Value::Bool(true) => true,
                            Value::Int(i) => *i != 0,
                            Value::Float(fl) => *fl != 0.0,
                            _ => false,
                        },
                        Err(_) => true,
                    }
                })
                .collect()
        } else {
            entities
        }
    }

    /// 投影返回列
    fn project_columns(&self, entities: Vec<Value>) -> Vec<Value> {
        if self.return_columns.is_empty() {
            return entities;
        }

        // 简化实现：实际应该根据return_columns过滤属性
        // 这里直接返回原实体
        entities
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for IndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("IndexScanExecutor storage lock should not be poisoned");

        // 1. 使用索引查找获取ID列表
        let index_results = self.lookup_by_index(&*storage)?;

        // 2. 根据ID获取完整实体
        let entities = self.fetch_entities(&*storage, index_results)?;

        // 3. 应用过滤器
        let filtered = self.apply_filter(entities);

        // 4. 投影返回列
        let projected = self.project_columns(filtered);

        // 5. 应用限制
        let limited: Vec<Value> = if let Some(limit) = self.limit {
            projected.into_iter().take(limit).collect()
        } else {
            projected
        };

        // 6. 构建返回结果
        let rows: Vec<Vec<Value>> = limited
            .into_iter()
            .map(|v| vec![v])
            .collect();

        Ok(ExecutionResult::Values(rows.into_iter().flatten().collect()))
    }

    fn open(&mut self) -> DBResult<()> {
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
        "Index scan executor - scans vertices using index"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for IndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}
