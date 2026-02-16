use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::error::DBError;
use crate::core::{Edge, EdgeDirection, NullType, Path, Value, Vertex};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

use crate::expression::context::traits::VariableContext;

/// BFSShortestExecutor - BFS最短路径执行器
///
/// 使用双向广度优先搜索算法查找最短路径
/// 参考nebula-graph实现，支持双向BFS和路径拼接
pub struct BFSShortestExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    steps: usize,
    max_depth: Option<usize>,
    edge_types: Vec<String>,
    with_cycle: bool,  // 是否允许回路（路径中重复访问顶点）
    with_loop: bool,   // 是否允许自环边
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
        with_cycle: bool,
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
            with_cycle,
            with_loop: false,
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
                all_edges.into_iter()
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
    fn get_space_name(&self, storage: &S) -> DBResult<String> {
        if let Ok(Some(space_info)) = storage.get_space_by_id(self.space_id) {
            Ok(space_info.space_name)
        } else {
            Ok("default".to_string())
        }
    }

    /// 获取schema名称（tag或edge类型名称）
    fn get_schema_name(&self, storage: &S) -> DBResult<String> {
        let space_name = self.get_space_name(storage)?;

        if self.is_edge {
            let edge_types = storage.list_edge_types(&space_name)
                .map_err(|e| DBError::Storage(e))?;
            if let Some(edge_type_info) = edge_types.iter().find(|e| e.edge_type_id == self.tag_id) {
                Ok(edge_type_info.edge_type_name.clone())
            } else {
                Ok(format!("edge_type_{}", self.tag_id.abs()))
            }
        } else {
            let tags = storage.list_tags(&space_name)
                .map_err(|e| DBError::Storage(e))?;
            if let Some(tag_info) = tags.iter().find(|t| t.tag_id == self.tag_id) {
                Ok(tag_info.tag_name.clone())
            } else {
                Ok(format!("tag_{}", self.tag_id))
            }
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
                // 参考 nebula-graph 的 RangePath 实现：
                // 1. 使用 begin_value 作为前缀进行初步查找
                // 2. 使用 end_value 进行范围过滤
                // 3. 支持包含/不包含边界控制 (include_begin, include_end)
                if let Some(first_limit) = self.scan_limits.first() {
                    let column_name = &first_limit.column;
                    let include_begin = first_limit.include_begin;
                    let include_end = first_limit.include_end;
                    
                    // 获取起始值和结束值
                    let start_value = first_limit.begin_value.as_ref()
                        .map(|v| Value::String(v.clone()));
                    let end_value = first_limit.end_value.as_ref()
                        .map(|v| Value::String(v.clone()));
                    
                    // 如果没有起始值，返回空结果
                    let start_val = match start_value {
                        Some(v) => v,
                        None => return Ok(Vec::new()),
                    };
                    
                    // 使用起始值进行前缀查找获取候选结果
                    let candidates = storage.lookup_index(&space_name, &index_name, &start_val)
                        .map_err(|e| DBError::Storage(e))?;
                    
                    // 如果有结束值，进行范围过滤
                    if let Some(end_val) = end_value {
                        let filtered: Vec<Value> = candidates
                            .into_iter()
                            .filter(|id| {
                                // 获取实体的属性值进行比较
                                match self.get_entity_property_for_filter(storage, id, column_name) {
                                    Some(prop_value) => {
                                        // 比较属性值是否在范围内，考虑边界包含控制
                                        Self::value_in_range(
                                            &prop_value,
                                            &start_val,
                                            &end_val,
                                            include_begin,
                                            include_end,
                                        )
                                    }
                                    None => false,
                                }
                            })
                            .collect();
                        Ok(filtered)
                    } else {
                        // 没有结束值，返回所有候选结果（从起始值到无穷大）
                        // 但仍需要检查起始边界
                        if include_begin {
                            Ok(candidates)
                        } else {
                            // 不包含起始值，需要过滤掉等于起始值的
                            let filtered: Vec<Value> = candidates
                                .into_iter()
                                .filter(|id| {
                                    match self.get_entity_property_for_filter(storage, id, column_name) {
                                        Some(prop_value) => !Self::values_equal(&prop_value, &start_val),
                                        None => false,
                                    }
                                })
                                .collect();
                            Ok(filtered)
                        }
                    }
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

    /// 获取实体的属性值用于范围过滤
    /// 根据ID获取实体的指定属性值
    fn get_entity_property_for_filter(&self, storage: &S, id: &Value, column_name: &str) -> Option<Value> {
        let space_name = match self.get_space_name(storage) {
            Ok(name) => name,
            Err(_) => return None,
        };
        
        if self.is_edge {
            // 边类型：ID格式应该是 src:dst:ranking
            if let Value::String(edge_key) = id {
                let parts: Vec<&str> = edge_key.split(':').collect();
                if parts.len() >= 2 {
                    let src = Value::String(parts[0].to_string());
                    let dst = Value::String(parts[1].to_string());
                    let schema_name = match self.get_schema_name(storage) {
                        Ok(name) => name,
                        Err(_) => return None,
                    };
                    
                    if let Ok(Some(edge)) = storage.get_edge(&space_name, &src, &dst, &schema_name) {
                        // 从边的属性中查找
                        if let Some(value) = edge.props.get(column_name) {
                            return Some(value.clone());
                        }
                        // 特殊字段
                        match column_name {
                            "src" => return Some((*edge.src).clone()),
                            "dst" => return Some((*edge.dst).clone()),
                            "edge_type" => return Some(Value::String(edge.edge_type.clone())),
                            "ranking" => return Some(Value::Int(edge.ranking)),
                            _ => return None,
                        }
                    }
                }
            }
        } else {
            // 顶点类型
            if let Ok(Some(vertex)) = storage.get_vertex(&space_name, id) {
                // 从顶点的属性中查找
                if let Some(value) = vertex.properties.get(column_name) {
                    return Some(value.clone());
                }
                // 从tag的属性中查找
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(column_name) {
                        return Some(value.clone());
                    }
                }
                // 特殊字段
                match column_name {
                    "vid" => return Some((*vertex.vid).clone()),
                    "id" => return Some(Value::Int(vertex.id)),
                    _ => return None,
                }
            }
        }
        
        None
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

        entities
            .into_iter()
            .map(|entity| {
                match entity {
                    Value::Vertex(vertex) => {
                        let mut props = std::collections::HashMap::new();
                        for col in &self.return_columns {
                            match col.as_str() {
                                "vid" => {
                                    props.insert(col.clone(), (*vertex.vid).clone());
                                }
                                "id" => {
                                    props.insert(col.clone(), Value::Int(vertex.id));
                                }
                                "*" => {
                                    for (k, v) in &vertex.properties {
                                        props.insert(k.clone(), v.clone());
                                    }
                                }
                                _ => {
                                    if let Some(v) = vertex.properties.get(col) {
                                        props.insert(col.clone(), v.clone());
                                    }
                                }
                            }
                        }
                        Value::Map(props)
                    }
                    Value::Edge(edge) => {
                        let mut props = std::collections::HashMap::new();
                        for col in &self.return_columns {
                            match col.as_str() {
                                "src" => {
                                    props.insert(col.clone(), (*edge.src).clone());
                                }
                                "dst" => {
                                    props.insert(col.clone(), (*edge.dst).clone());
                                }
                                "edge_type" => {
                                    props.insert(col.clone(), Value::String(edge.edge_type.clone()));
                                }
                                "ranking" => {
                                    props.insert(col.clone(), Value::Int(edge.ranking));
                                }
                                "*" => {
                                    for (k, v) in &edge.props {
                                        props.insert(k.clone(), v.clone());
                                    }
                                }
                                _ => {
                                    if let Some(v) = edge.props.get(col) {
                                        props.insert(col.clone(), v.clone());
                                    }
                                }
                            }
                        }
                        Value::Map(props)
                    }
                    _ => entity,
                }
            })
            .collect()
    }

    /// 检查值是否在指定范围内
    /// 支持边界包含控制 (include_begin, include_end)
    fn value_in_range(
        value: &Value,
        start: &Value,
        end: &Value,
        include_begin: bool,
        include_end: bool,
    ) -> bool {
        use std::cmp::Ordering;

        // 比较起始边界
        let pass_start = match Self::compare_values(value, start) {
            Some(Ordering::Greater) => true,
            Some(Ordering::Equal) => include_begin,
            Some(Ordering::Less) => false,
            None => false,
        };

        if !pass_start {
            return false;
        }

        // 比较结束边界
        match Self::compare_values(value, end) {
            Some(Ordering::Less) => true,
            Some(Ordering::Equal) => include_end,
            Some(Ordering::Greater) => false,
            None => false,
        }
    }

    /// 比较两个值
    fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
        match (a, b) {
            (Value::Int(a_i), Value::Int(b_i)) => Some(a_i.cmp(b_i)),
            (Value::Float(a_f), Value::Float(b_f)) => a_f.partial_cmp(b_f),
            (Value::Int(a_i), Value::Float(b_f)) => (*a_i as f64).partial_cmp(b_f),
            (Value::Float(a_f), Value::Int(b_i)) => a_f.partial_cmp(&(*b_i as f64)),
            (Value::String(a_s), Value::String(b_s)) => Some(a_s.cmp(b_s)),
            _ => None,
        }
    }

    /// 检查两个值是否相等
    fn values_equal(a: &Value, b: &Value) -> bool {
        matches!(Self::compare_values(a, b), Some(std::cmp::Ordering::Equal))
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for IndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.get_storage().lock();

        // 1. 使用索引查找获取ID列表
        let index_results = self.lookup_by_index(&storage)?;

        // 2. 根据ID获取完整实体
        let entities = self.fetch_entities(&storage, index_results)?;

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
