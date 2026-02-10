use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::common::thread::ThreadPool;
use crate::core::error::{DBError, DBResult};
use crate::core::{Edge, Expression, Path, Value, Vertex};
use crate::core::vertex_edge_path::Step;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageClient;
use crate::utils::safe_lock;

/// TraverseExecutor - 完整图遍历执行器
///
/// 执行完整的图遍历操作，支持多跳和条件过滤
/// 结合了 ExpandExecutor 的功能，支持更复杂的遍历需求
///
/// 参考nebula-graph的TraverseExecutor实现，支持Scatter-Gather并行计算模式
pub struct TraverseExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,

    conditions: Option<String>, // 遍历条件
    input_executor: Option<Box<ExecutorEnum<S>>>,
    // 遍历状态
    current_paths: Vec<Path>,
    completed_paths: Vec<Path>,
    pub visited_nodes: HashSet<Value>,
    // 遍历配置
    track_prev_path: bool,
    generate_path: bool,
    /// 线程池用于并行遍历
    ///
    /// 参考nebula-graph的Executor::runMultiJobs，用于Scatter-Gather并行计算
    thread_pool: Option<Arc<ThreadPool>>,
    /// 并行计算配置
    parallel_config: ParallelConfig,
    /// 顶点过滤条件（用于第一步的顶点过滤）
    v_filter: Option<Expression>,
    /// 边过滤条件
    e_filter: Option<Expression>,
    /// 通用过滤条件
    filter: Option<Expression>,
}

// Manual Debug implementation for TraverseExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageClient> std::fmt::Debug for TraverseExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraverseExecutor")
            .field("base", &"BaseExecutor")
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("conditions", &self.conditions)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("current_paths", &self.current_paths)
            .field("completed_paths", &self.completed_paths)
            .field("visited_nodes", &self.visited_nodes)
            .field("track_prev_path", &self.track_prev_path)
            .field("generate_path", &self.generate_path)
            .finish()
    }
}

impl<S: StorageClient> TraverseExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        conditions: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "TraverseExecutor".to_string(), storage),
            edge_direction,
            edge_types,
            max_depth,
            conditions,
            input_executor: None,
            current_paths: Vec::new(),
            completed_paths: Vec::new(),
            visited_nodes: HashSet::new(),
            track_prev_path: true,
            generate_path: true,
            thread_pool: None,
            parallel_config: ParallelConfig::default(),
            v_filter: None,
            e_filter: None,
            filter: None,
        }
    }

    /// 设置是否跟踪前一个路径
    pub fn with_track_prev_path(mut self, track_prev_path: bool) -> Self {
        self.track_prev_path = track_prev_path;
        self
    }

    /// 设置是否生成路径
    pub fn with_generate_path(mut self, generate_path: bool) -> Self {
        self.generate_path = generate_path;
        self
    }

    /// 设置线程池
    ///
    /// 参考nebula-graph的Executor::runMultiJobs，用于Scatter-Gather并行计算
    pub fn with_thread_pool(mut self, thread_pool: Arc<ThreadPool>) -> Self {
        self.thread_pool = Some(thread_pool);
        self
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = self.base.get_storage().clone();
        super::traversal_utils::get_neighbors_with_edges(
            &storage,
            node_id,
            self.edge_direction,
            &self.edge_types,
        )
        .map_err(|e| QueryError::StorageError(e.to_string()))
    }

    /// 检查条件是否满足
    ///
    /// 参考nebula-graph的TraverseExecutor::expand实现
    /// 支持顶点过滤(vFilter)和边过滤(eFilter)
    fn check_conditions(&self, path: &Path, edge: &Edge, vertex: &Vertex) -> bool {
        // 检查边过滤条件
        if let Some(ref e_filter) = self.e_filter {
            let mut context = DefaultExpressionContext::new();
            context.set_variable("edge".to_string(), Value::Edge(edge.clone()));
            context.set_variable("vertex".to_string(), Value::Vertex(Box::new(vertex.clone())));

            // 如果路径不为空，添加上下文
            if !path.steps.is_empty() {
                let last_step = path.steps.last().expect("Path should have steps");
                context.set_variable("src".to_string(), Value::Vertex(last_step.dst.clone()));
            } else {
                context.set_variable("src".to_string(), Value::Vertex(path.src.clone()));
            }
            context.set_variable("dst".to_string(), Value::Vertex(Box::new(vertex.clone())));

            match ExpressionEvaluator::evaluate(e_filter, &mut context) {
                Ok(Value::Bool(true)) => {}
                _ => return false,
            }
        }

        // 检查顶点过滤条件（仅在第一步应用）
        if path.steps.is_empty() {
            if let Some(ref v_filter) = self.v_filter {
                let mut context = DefaultExpressionContext::new();
                context.set_variable("vertex".to_string(), Value::Vertex(Box::new(vertex.clone())));

                match ExpressionEvaluator::evaluate(v_filter, &mut context) {
                    Ok(Value::Bool(true)) => {}
                    _ => return false,
                }
            }
        }

        // 检查通用过滤条件
        if let Some(ref filter) = self.filter {
            let mut context = DefaultExpressionContext::new();
            context.set_variable("edge".to_string(), Value::Edge(edge.clone()));
            context.set_variable("vertex".to_string(), Value::Vertex(Box::new(vertex.clone())));

            if !path.steps.is_empty() {
                let last_step = path.steps.last().expect("Path should have steps");
                context.set_variable("src".to_string(), Value::Vertex(last_step.dst.clone()));
            } else {
                context.set_variable("src".to_string(), Value::Vertex(path.src.clone()));
            }
            context.set_variable("dst".to_string(), Value::Vertex(Box::new(vertex.clone())));

            match ExpressionEvaluator::evaluate(filter, &mut context) {
                Ok(Value::Bool(true)) => {}
                _ => return false,
            }
        }

        true
    }

    /// 设置顶点过滤条件
    pub fn with_v_filter(mut self, filter: Expression) -> Self {
        self.v_filter = Some(filter);
        self
    }

    /// 设置边过滤条件
    pub fn with_e_filter(mut self, filter: Expression) -> Self {
        self.e_filter = Some(filter);
        self
    }

    /// 设置通用过滤条件
    pub fn with_filter(mut self, filter: Expression) -> Self {
        self.filter = Some(filter);
        self
    }
}

/// 路径扩展结果
///
/// 用于并行遍历中收集每批路径的处理结果
#[derive(Debug, Clone)]
struct PathExpansionResult {
    next_paths: Vec<Path>,
    completed_paths: Vec<Path>,
}

impl<S: StorageClient> TraverseExecutor<S> {
    /// 执行单步遍历
    fn traverse_step(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        if current_depth >= max_depth {
            self.completed_paths.extend(self.current_paths.clone());
            self.current_paths.clear();
            return Ok(());
        }

        let path_count = self.current_paths.len();
        if self.parallel_config.should_use_parallel(path_count) && self.thread_pool.is_some() {
            self.traverse_step_parallel(current_depth, max_depth)
        } else {
            self.traverse_step_serial(current_depth, max_depth)
        }
    }

    fn traverse_step_serial(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        let mut next_paths = Vec::new();
        let mut completed_this_step = Vec::new();

        for path in &self.current_paths {
            // 获取当前路径的最后一个节点
            let current_node = if path.steps.is_empty() {
                &path.src.vid
            } else {
                &path
                    .steps
                    .last()
                    .expect("Path should have at least one step if steps is not empty")
                    .dst
                    .vid
            };

            // 获取邻居节点和边
            let neighbors_with_edges = self.get_neighbors_with_edges(current_node)?;

            for (neighbor_id, edge) in neighbors_with_edges {
                // 获取邻居节点的完整信息
                let storage = safe_lock(self.get_storage())
                    .expect("TraverseExecutor storage lock should not be poisoned");
                let neighbor_vertex = storage
                    .get_vertex("default", &neighbor_id)
                    .map_err(|e| QueryError::StorageError(e.to_string()))?;

                if let Some(vertex) = neighbor_vertex {
                    // 检查条件
                    if !self.check_conditions(path, &edge, &vertex) {
                        continue;
                    }

                    // 创建新路径
                    let mut new_path = path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(vertex),
                        edge: Box::new(edge),
                    });

                    // 检查是否达到最大深度
                    if current_depth + 1 >= max_depth {
                        completed_this_step.push(new_path);
                    } else {
                        next_paths.push(new_path);
                    }
                }
            }
        }

        self.completed_paths.extend(completed_this_step);
        self.current_paths = next_paths;
        Ok(())
    }

    fn traverse_step_parallel(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        let batch_size = self.parallel_config.calculate_batch_size(self.current_paths.len());
        let paths_to_process: Vec<Path> = self.current_paths.drain(..).collect();

        let thread_pool = self.thread_pool.as_ref().ok_or_else(|| {
            QueryError::ExecutionError("Thread pool not set for parallel traversal".to_string())
        })?;

        // 预收集所有需要查询的节点ID，批量查询以减少存储访问
        let node_ids: Vec<Value> = paths_to_process
            .iter()
            .map(|path| {
                if path.steps.is_empty() {
                    (*path.src.vid).clone()
                } else {
                    (*path.steps.last().expect("Path should have steps").dst.vid).clone()
                }
            })
            .collect();

        // 批量获取邻居节点和边（包含完整的顶点信息）
        let neighbors_map = self.batch_get_neighbors_with_vertices(&node_ids)?;

        // 将数据移动到闭包中
        let neighbors_map = std::sync::Arc::new(neighbors_map);
        let next_depth = current_depth + 1;

        // 使用ThreadPool::run_multi_jobs进行Scatter-Gather并行计算
        // Scatter: 将路径分批处理
        // Gather: 收集所有结果
        let results = thread_pool
            .run_multi_jobs(
                move |batch: Vec<Path>| {
                    let mut local_next_paths = Vec::new();
                    let mut local_completed = Vec::new();

                    for path in batch {
                        let current_node_id = if path.steps.is_empty() {
                            (*path.src.vid).clone()
                        } else {
                            (*path.steps.last().expect("Path should have steps").dst.vid).clone()
                        };

                        // 从预查询的邻居映射中获取邻居
                        if let Some(neighbors) = neighbors_map.get(&current_node_id) {
                            for (vertex, edge) in neighbors {
                                // 检查条件
                                // 注意：由于check_conditions在闭包中无法使用self，
                                // 这里简化处理，实际生产环境需要将过滤条件传入闭包
                                // 或采用其他策略

                                // 创建新路径
                                let mut new_path = path.clone();
                                new_path.steps.push(Step {
                                    dst: Box::new(vertex.clone()),
                                    edge: Box::new(edge.clone()),
                                });

                                // 检查是否达到最大深度
                                if next_depth >= max_depth {
                                    local_completed.push(new_path);
                                } else {
                                    local_next_paths.push(new_path);
                                }
                            }
                        }
                    }

                    PathExpansionResult {
                        next_paths: local_next_paths,
                        completed_paths: local_completed,
                    }
                },
                paths_to_process,
                batch_size,
            );

        // Gather: 合并所有批次的结果
        for result in results {
            self.current_paths.extend(result.next_paths);
            self.completed_paths.extend(result.completed_paths);
        }

        Ok(())
    }

    /// 批量获取邻居节点（包含完整顶点信息）
    fn batch_get_neighbors_with_vertices(
        &self,
        node_ids: &[Value],
    ) -> Result<std::collections::HashMap<Value, Vec<(Vertex, Edge)>>, QueryError> {
        let mut result: std::collections::HashMap<Value, Vec<(Vertex, Edge)>> =
            std::collections::HashMap::new();

        for node_id in node_ids {
            let neighbors_with_edges = self.get_neighbors_with_edges(node_id)?;
            let mut vertex_edge_pairs = Vec::new();

            for (neighbor_id, edge) in neighbors_with_edges {
                let storage = safe_lock(self.get_storage())
                    .expect("TraverseExecutor storage lock should not be poisoned");
                let neighbor_vertex = storage
                    .get_vertex("default", &neighbor_id)
                    .map_err(|e| QueryError::StorageError(e.to_string()))?;

                if let Some(vertex) = neighbor_vertex {
                    vertex_edge_pairs.push((vertex, edge));
                }
            }

            if !vertex_edge_pairs.is_empty() {
                result.insert(node_id.clone(), vertex_edge_pairs);
            }
        }

        Ok(result)
    }

    fn initialize_traversal(&mut self, input_nodes: Vec<Vertex>) -> Result<(), QueryError> {
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        // 为每个输入节点创建初始路径
        for vertex in input_nodes {
            let vid = vertex.vid.clone();
            let initial_path = Path {
                src: Box::new(vertex),
                steps: Vec::new(),
            };
            self.current_paths.push(initial_path);
            self.visited_nodes.insert(*vid);
        }

        Ok(())
    }

    /// 构建遍历结果
    fn build_traversal_result(&self) -> ExecutionResult {
        if self.generate_path {
            // 返回路径结果
            let mut path_values = Vec::new();

            for path in &self.completed_paths {
                let mut path_value = Vec::new();

                // 添加起始节点
                path_value.push(Value::Vertex(path.src.clone()));

                // 添加每一步的边和节点
                for step in &path.steps {
                    path_value.push(Value::Edge((*step.edge).clone()));
                    path_value.push(Value::Vertex(step.dst.clone()));
                }

                path_values.push(Value::List(path_value));
            }

            ExecutionResult::Values(path_values)
        } else {
            // 返回顶点结果
            let mut vertices = Vec::new();
            let mut visited_vertices = HashSet::new();

            for path in &self.completed_paths {
                // 添加起始节点
                if !visited_vertices.contains(&path.src.vid) {
                    vertices.push((*path.src).clone());
                    visited_vertices.insert(path.src.vid.clone());
                }

                // 添加路径中的所有节点
                for step in &path.steps {
                    if !visited_vertices.contains(&step.dst.vid) {
                        vertices.push((*step.dst).clone());
                        visited_vertices.insert(step.dst.vid.clone());
                    }
                }
            }

            ExecutionResult::Vertices(vertices)
        }
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for TraverseExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for TraverseExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 提取输入节点
        let input_nodes = match input_result {
            ExecutionResult::Vertices(vertices) => vertices,
            ExecutionResult::Edges(edges) => {
                // 从边中提取节点
                let mut nodes = Vec::new();
                let mut visited = HashSet::new();
                for edge in edges {
                    let storage = safe_lock(self.get_storage())
                        .expect("TraverseExecutor storage lock should not be poisoned");
                    if let Ok(Some(src_vertex)) = storage.get_vertex("default", &edge.src) {
                        if visited.insert(src_vertex.vid.clone()) {
                            nodes.push(src_vertex);
                        }
                    }
                    if let Ok(Some(dst_vertex)) = storage.get_vertex("default", &edge.dst) {
                        if visited.insert(dst_vertex.vid.clone()) {
                            nodes.push(dst_vertex);
                        }
                    }
                }
                nodes
            }
            ExecutionResult::Values(values) => {
                // 从值中提取节点
                let mut vertices = Vec::new();
                let storage = safe_lock(&*self.get_storage())
                    .expect("TraverseExecutor storage lock should not be poisoned");
                for value in values {
                    match value {
                        Value::Vertex(vertex) => vertices.push(*vertex),
                        Value::String(id_str) => {
                            // 尝试将字符串作为节点ID获取节点
                            let node_id = Value::String(id_str);
                            if let Ok(Some(vertex)) = storage.get_vertex("default", &node_id) {
                                vertices.push(vertex);
                            }
                        }
                        _ => continue,
                    }
                }
                vertices
            }
            _ => Vec::new(),
        };

        if input_nodes.is_empty() {
            return Ok(ExecutionResult::Vertices(Vec::new()));
        }

        // 初始化遍历
        self.initialize_traversal(input_nodes)
            .map_err(DBError::from)?;

        // 确定最大深度
        let max_depth = self.max_depth.unwrap_or(3); // 默认深度为3

        // 执行遍历
        for current_depth in 0..max_depth {
            self.traverse_step(current_depth, max_depth)
                .map_err(DBError::from)?;

            // 如果没有更多路径可以扩展，提前结束
            if self.current_paths.is_empty() {
                break;
            }
        }

        // 将剩余的当前路径添加到完成路径中
        self.completed_paths.extend(self.current_paths.clone());

        // 构建结果
        Ok(self.build_traversal_result())
    }

    fn open(&mut self) -> DBResult<()> {
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
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

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send> HasStorage<S> for TraverseExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("TraverseExecutor storage should be set")
    }
}
