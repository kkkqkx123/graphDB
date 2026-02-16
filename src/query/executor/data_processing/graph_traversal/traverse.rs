use std::collections::HashSet;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult};
use crate::core::{Edge, Expression, NPath, Path, Value, Vertex};
use crate::core::value::dataset::List;

use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// TraverseExecutor - 完整图遍历执行器
///
/// 执行完整的图遍历操作，支持多跳和条件过滤
/// 结合了 ExpandExecutor 的功能，支持更复杂的遍历需求
pub struct TraverseExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,

    conditions: Option<String>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 使用 NPath 存储当前遍历路径，减少内存复制
    current_npaths: Vec<Arc<NPath>>,
    /// 使用 NPath 存储已完成路径
    completed_npaths: Vec<Arc<NPath>>,
    /// 最终输出时转换为 Path
    current_paths: Vec<Path>,
    completed_paths: Vec<Path>,
    pub visited_nodes: HashSet<Value>,
    track_prev_path: bool,
    generate_path: bool,
    v_filter: Option<Expression>,
    e_filter: Option<Expression>,
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
            current_npaths: Vec::new(),
            completed_npaths: Vec::new(),
            current_paths: Vec::new(),
            completed_paths: Vec::new(),
            visited_nodes: HashSet::new(),
            track_prev_path: true,
            generate_path: true,
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
            false, // 默认不允许自环边
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

impl<S: StorageClient> TraverseExecutor<S> {
    /// 执行单步遍历
    fn traverse_step(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        if current_depth >= max_depth {
            // 将剩余的 current_npaths 移到 completed_npaths
            self.completed_npaths.extend(self.current_npaths.clone());
            self.current_npaths.clear();
            return Ok(());
        }

        self.traverse_step_serial(current_depth, max_depth)
    }

    fn traverse_step_serial(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        let mut next_npaths: Vec<Arc<NPath>> = Vec::new();
        let mut completed_this_step: Vec<Arc<NPath>> = Vec::new();

        for npath in &self.current_npaths {
            // 获取当前路径的最后一个节点
            let current_node = &npath.vertex().vid;

            // 获取邻居节点和边
            let neighbors_with_edges = self.get_neighbors_with_edges(current_node)?;

            for (neighbor_id, edge) in neighbors_with_edges {
                // 获取邻居节点的完整信息
                let storage = self.get_storage().lock();
                let neighbor_vertex = storage
                    .get_vertex("default", &neighbor_id)
                    .map_err(|e| QueryError::StorageError(e.to_string()))?;

                if let Some(vertex) = neighbor_vertex {
                    // 将 NPath 转换为 Path 用于条件检查
                    let path = npath.to_path();
                    // 检查条件
                    if !self.check_conditions(&path, &edge, &vertex) {
                        continue;
                    }

                    // 使用 NPath 扩展，O(1) 操作
                    let new_npath = Arc::new(NPath::extend(
                        npath.clone(),
                        Arc::new(edge),
                        Arc::new(vertex),
                    ));

                    // 检查是否达到最大深度
                    if current_depth + 1 >= max_depth {
                        completed_this_step.push(new_npath);
                    } else {
                        next_npaths.push(new_npath);
                    }
                }
            }
        }

        self.completed_npaths.extend(completed_this_step);
        self.current_npaths = next_npaths;
        Ok(())
    }

    fn initialize_traversal(&mut self, input_nodes: Vec<Vertex>) -> Result<(), QueryError> {
        self.current_npaths.clear();
        self.completed_npaths.clear();
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        // 为每个输入节点创建初始 NPath
        for vertex in input_nodes {
            let vid = vertex.vid.clone();
            let initial_npath = Arc::new(NPath::new(Arc::new(vertex)));
            self.current_npaths.push(initial_npath);
            self.visited_nodes.insert(*vid);
        }

        Ok(())
    }

    /// 构建遍历结果
    fn build_traversal_result(&self) -> ExecutionResult {
        // 将 NPath 转换为 Path 用于输出
        let completed_paths: Vec<Path> = self.completed_npaths.iter().map(|np| np.to_path()).collect();

        if self.generate_path {
            // 返回路径结果
            let mut path_values = Vec::new();

            for path in &completed_paths {
                let mut path_value = Vec::new();

                // 添加起始节点
                path_value.push(Value::Vertex(path.src.clone()));

                // 添加每一步的边和节点
                for step in &path.steps {
                    path_value.push(Value::Edge((*step.edge).clone()));
                    path_value.push(Value::Vertex(step.dst.clone()));
                }

                path_values.push(Value::List(List::from(path_value)));
            }

            ExecutionResult::Values(path_values)
        } else {
            // 返回顶点结果
            let mut vertices = Vec::new();
            let mut visited_vertices = HashSet::new();

            for path in &completed_paths {
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
                    let storage = self.get_storage().lock();
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
                let storage = self.get_storage().lock();
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
