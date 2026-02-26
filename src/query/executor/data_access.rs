use std::sync::Arc;
use std::time::Instant;

use super::base::{BaseExecutor, ExecutorStats};
use crate::core::{Value, vertex_edge_path};
use crate::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

pub struct GetVerticesExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    tag_filter: Option<crate::core::Expression>,
    vertex_filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageClient + 'static> GetVerticesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        tag_filter: Option<crate::core::Expression>,
        vertex_filter: Option<crate::core::Expression>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetVerticesExecutor".to_string(), storage),
            vertex_ids,
            tag_filter,
            vertex_filter,
            limit,
        }
    }
}

impl<S: StorageClient + 'static> Executor<S> for GetVerticesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();

        let result = self.do_execute();

        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);

        match result {
            Ok(vertices) => Ok(ExecutionResult::Vertices(vertices)),
            Err(e) => Err(e),
        }
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
        "GetVerticesExecutor"
    }

    fn description(&self) -> &str {
        "Get vertices executor - retrieves vertices from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for GetVerticesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + 'static> GetVerticesExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<vertex_edge_path::Vertex>> {
        match &self.vertex_ids {
            Some(ids) if ids.len() > 1 => {
                let storage = self.get_storage().lock();
                let mut result_vertices: Vec<vertex_edge_path::Vertex> = Vec::new();
                let mut failed_count = 0;

                for id in ids {
                    match storage.get_vertex("default", id) {
                        Ok(Some(vertex)) => {
                            let include_vertex = if let Some(ref tag_filter_expression) = self.tag_filter {
                                crate::query::executor::tag_filter::TagFilterProcessor
                                    ::process_tag_filter(tag_filter_expression, &vertex)
                            } else {
                                true
                            };

                            if include_vertex {
                                result_vertices.push(vertex);
                            }
                        }
                        Ok(None) => {
                            failed_count += 1;
                        }
                        Err(_) => {
                            failed_count += 1;
                        }
                    }

                    if let Some(limit) = self.limit {
                        if result_vertices.len() >= limit {
                            break;
                        }
                    }
                }

                if failed_count > 0 {
                    log::warn!("获取顶点失败: {} 个", failed_count);
                }

                Ok(result_vertices)
            }
            Some(ids) if ids.len() == 1 => {
                let storage = self.get_storage().lock();

                if let Some(vertex) = storage.get_vertex("default", &ids[0])? {
                    Ok(vec![vertex])
                } else {
                    Ok(Vec::new())
                }
            }
            Some(_) => Ok(Vec::new()),
            None => {
                let storage = self.get_storage().lock();

                let vertices = storage.scan_vertices("default")?
                    .into_iter()
                    .filter(|vertex| {
                        if let Some(ref tag_filter_expression) = self.tag_filter {
                            crate::query::executor::tag_filter::TagFilterProcessor
                                ::process_tag_filter(tag_filter_expression, vertex)
                        } else {
                            true
                        }
                    })
                    .filter(|vertex| {
                        if let Some(ref filter_expression) = self.vertex_filter {
                            let mut context =
                                crate::expression::DefaultExpressionContext::new();
                            context.set_variable(
                                "vertex".to_string(),
                                crate::core::Value::Vertex(Box::new(vertex.clone())),
                            );

                            match crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expression, &mut context) {
                                Ok(value) => {
                                    match value {
                                        crate::core::Value::Bool(b) => b,
                                        crate::core::Value::Int(i) => i != 0,
                                        crate::core::Value::Float(f) => f != 0.0,
                                        crate::core::Value::String(s) => !s.is_empty(),
                                        crate::core::Value::List(l) => !l.is_empty(),
                                        crate::core::Value::Map(m) => !m.is_empty(),
                                        crate::core::Value::Set(s) => !s.is_empty(),
                                        crate::core::Value::Vertex(_) => true,
                                        crate::core::Value::Edge(_) => true,
                                        crate::core::Value::Path(_) => true,
                                        crate::core::Value::Null(_) => false,
                                        crate::core::Value::Empty => false,
                                        crate::core::Value::Date(_) => true,
                                        crate::core::Value::Time(_) => true,
                                        crate::core::Value::DateTime(_) => true,
                                        crate::core::Value::Geography(_) => true,
                                        crate::core::Value::Duration(_) => true,
                                        crate::core::Value::DataSet(ds) => !ds.rows.is_empty(),
                                    }
                                }
                                Err(e) => {
                                    log::warn!("顶点过滤表达式评估失败: {}", e);
                                    false
                                }
                            }
                        } else {
                            true
                        }
                    })
                    .take(self.limit.unwrap_or(usize::MAX))
                    .collect();
                Ok(vertices)
            }
        }
    }
}

pub struct GetEdgesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    edge_type: Option<String>,
}

impl<S: StorageClient> GetEdgesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, edge_type: Option<String>) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetEdgesExecutor".to_string(), storage),
            edge_type,
        }
    }
}

impl<S: StorageClient> Executor<S> for GetEdgesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(edges) => Ok(ExecutionResult::Edges(edges)),
            Err(e) => Err(e),
        }
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
        "GetEdgesExecutor"
    }

    fn description(&self) -> &str {
        "Get edges executor - retrieves edges from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for GetEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> GetEdgesExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<vertex_edge_path::Edge>> {
        let storage = self.get_storage().lock();

        let edges = if let Some(ref edge_type) = self.edge_type {
            storage.scan_edges_by_type("default", edge_type)?
        } else {
            storage.scan_all_edges("default")?
        };

        Ok(edges)
    }
}

pub struct ScanEdgesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    edge_type: Option<String>,
    filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageClient> ScanEdgesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_type: Option<String>,
        filter: Option<crate::core::Expression>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ScanEdgesExecutor".to_string(), storage),
            edge_type,
            filter,
            limit,
        }
    }
}

impl<S: StorageClient> Executor<S> for ScanEdgesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(edges) => Ok(ExecutionResult::Edges(edges)),
            Err(e) => Err(e),
        }
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
        "ScanEdgesExecutor"
    }

    fn description(&self) -> &str {
        "Scan edges executor - scans all edges from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for ScanEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> ScanEdgesExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<vertex_edge_path::Edge>> {
        let storage = self.get_storage().lock();

        let mut edges: Vec<vertex_edge_path::Edge> = if let Some(ref edge_type) = self.edge_type {
            storage.scan_edges_by_type("default", edge_type)?
        } else {
            storage.scan_all_edges("default")?
        };

        if let Some(ref filter_expr) = self.filter {
            let mut context = crate::expression::DefaultExpressionContext::new();
            edges.retain(|edge| {
                context.set_variable("edge".to_string(), crate::core::Value::Edge(edge.clone()));
                match crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expr, &mut context) {
                    Ok(value) => match value {
                        crate::core::Value::Bool(b) => b,
                        crate::core::Value::Int(i) => i != 0,
                        crate::core::Value::Float(f) => f != 0.0,
                        _ => true,
                    },
                    Err(_) => true,
                }
            });
        }

        if let Some(limit) = self.limit {
            edges.truncate(limit);
        }

        Ok(edges)
    }
}

pub struct GetNeighborsExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    vertex_ids: Vec<Value>,
    edge_direction: super::base::EdgeDirection,
    edge_types: Option<Vec<String>>,
}

impl<S: StorageClient> GetNeighborsExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Vec<Value>,
        edge_direction: super::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetNeighborsExecutor".to_string(), storage),
            vertex_ids,
            edge_direction,
            edge_types,
        }
    }
}

impl<S: StorageClient + 'static> Executor<S> for GetNeighborsExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(values) => Ok(ExecutionResult::Values(values)),
            Err(e) => Err(e),
        }
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
        "GetNeighborsExecutor"
    }

    fn description(&self) -> &str {
        "Get neighbors executor - retrieves neighboring vertices"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for GetNeighborsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + 'static> GetNeighborsExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<Value>> {
        if self.vertex_ids.is_empty() {
            return Ok(Vec::new());
        }

        let storage = self.get_storage().lock();
        let mut neighbor_ids: Vec<Value> = Vec::new();
        let edge_types_filter = self.edge_types.as_ref();
        let direction = self.edge_direction;

        for vertex_id in &self.vertex_ids {
            let edges = storage.get_node_edges("default", vertex_id, direction)?;

            for edge in edges {
                if let Some(ref filter_types) = edge_types_filter {
                    if !filter_types.contains(&edge.edge_type) {
                        continue;
                    }
                }

                let neighbor_id = if edge.src.as_ref() == vertex_id {
                    (*edge.dst).clone()
                } else {
                    (*edge.src).clone()
                };

                neighbor_ids.push(neighbor_id);
            }
        }

        neighbor_ids.sort();
        neighbor_ids.dedup();

        if neighbor_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut neighbors: Vec<Value> = Vec::new();
        let mut failed_count = 0;

        for neighbor_id in &neighbor_ids {
            match storage.get_vertex("default", neighbor_id) {
                Ok(Some(vertex)) => {
                    neighbors.push(crate::core::Value::Vertex(Box::new(vertex)));
                }
                Ok(None) => {
                    failed_count += 1;
                }
                Err(_) => {
                    failed_count += 1;
                }
            }
        }

        if failed_count > 0 {
            log::warn!("获取邻居顶点失败: {} 个", failed_count);
        }

        Ok(neighbors)
    }
}

#[derive(Debug)]
pub struct GetPropExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    edge_ids: Option<Vec<Value>>,
    prop_names: Vec<String>,
}

impl<S: StorageClient> GetPropExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<Value>>,
        prop_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetPropExecutor".to_string(), storage),
            vertex_ids,
            edge_ids,
            prop_names,
        }
    }
}

impl<S: StorageClient> Executor<S> for GetPropExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(values) => Ok(ExecutionResult::Values(values)),
            Err(e) => Err(e),
        }
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
        "GetPropExecutor"
    }

    fn description(&self) -> &str {
        "Get property executor - retrieves properties from vertices or edges"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for GetPropExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> GetPropExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<Value>> {
        let storage = self.get_storage().lock();

        let mut props = Vec::new();

        if let Some(ref vertex_ids) = self.vertex_ids {
            let total_props = vertex_ids.len() * self.prop_names.len();
            props.reserve(total_props);

            for vertex_id in vertex_ids {
                if let Some(vertex) = storage.get_vertex("default", vertex_id)? {
                    for prop_name in &self.prop_names {
                        if let Some(value) = vertex.get_property_any(prop_name) {
                            props.push(value.clone());
                        } else {
                            props.push(crate::core::Value::Null(crate::core::value::NullType::Null));
                        }
                    }
                }
            }
        }

        if let Some(ref edge_ids) = self.edge_ids {
            let total_props = edge_ids.len() * self.prop_names.len();
            props.reserve(total_props);

            for edge_id in edge_ids {
                if let crate::core::Value::Edge(edge) = edge_id {
                    for prop_name in &self.prop_names {
                        if let Some(value) = edge.get_property(prop_name) {
                            props.push(value.clone());
                        } else {
                            props.push(crate::core::Value::Null(crate::core::value::NullType::Null));
                        }
                    }
                }
            }
        }

        Ok(props)
    }
}

use crate::core::vertex_edge_path::{Path, Step};

use super::base::EdgeDirection;

#[derive(Debug)]
pub struct IndexScanExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    index_condition: Option<(String, Value)>,
    scan_forward: bool,
    limit: Option<usize>,
}

impl<S: StorageClient> IndexScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        index_condition: Option<(String, Value)>,
        scan_forward: bool,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "IndexScanExecutor".to_string(), storage),
            index_name,
            index_condition,
            scan_forward,
            limit,
        }
    }
}

impl<S: StorageClient> Executor<S> for IndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(values) => Ok(ExecutionResult::Values(values)),
            Err(e) => Err(e),
        }
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
        "IndexScanExecutor"
    }

    fn description(&self) -> &str {
        "Index scan executor - retrieves vertices using index"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for IndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> IndexScanExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<Value>> {
        let storage = self.get_storage().lock();

        let mut results = Vec::new();

        if let Some((prop_name, prop_value)) = &self.index_condition {
            let scan_results = storage.scan_vertices_by_prop("default", &self.index_name, prop_name, prop_value)?;

            for vertex in scan_results {
                results.push(crate::core::Value::Vertex(Box::new(vertex)));

                if let Some(limit) = self.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        } else {
            let scan_results = if self.scan_forward {
                storage.scan_vertices_by_tag("default", &self.index_name)?
            } else {
                storage.scan_vertices("default")?
            };

            for vertex in scan_results {
                results.push(crate::core::Value::Vertex(Box::new(vertex)));

                if let Some(limit) = self.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug)]
pub struct AllPathsExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    start_vertex: Value,
    end_vertex: Option<Value>,
    max_hops: usize,
    edge_types: Option<Vec<String>>,
    direction: EdgeDirection,
}

impl<S: StorageClient> AllPathsExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vertex: Value,
        end_vertex: Option<Value>,
        max_hops: usize,
        edge_types: Option<Vec<String>>,
        direction: EdgeDirection,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AllPathsExecutor".to_string(), storage),
            start_vertex,
            end_vertex,
            max_hops,
            edge_types,
            direction,
        }
    }
}

impl<S: StorageClient> Executor<S> for AllPathsExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.get_storage().lock();

        let mut all_paths: Vec<Path> = Vec::new();

        let start_vertex_obj = if let Some(vertex) = storage.get_vertex("default", &self.start_vertex)? {
            vertex
        } else {
            return Ok(ExecutionResult::Values(vec![]));
        };

        let mut current_paths: Vec<Path> = vec![Path {
            src: Box::new(start_vertex_obj.clone()),
            steps: Vec::new(),
        }];

        for _hop in 0..self.max_hops {
            let mut next_paths: Vec<Path> = Vec::new();

            for path in &current_paths {
                let direction = self.direction;

                let edges = storage.get_node_edges("default", &self.start_vertex, direction)?;

                for edge in edges {
                    let neighbor_id = edge.dst.clone();

                    if let Some(ref _end_vertex) = self.end_vertex {
                        continue;
                    }

                    if let Some(ref edge_types) = self.edge_types {
                        if !edge_types.contains(&edge.edge_type) {
                            continue;
                        }
                    }

                    if let Some(neighbor) = storage.get_vertex("default", &neighbor_id)? {
                        let mut new_path = path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(neighbor),
                            edge: Box::new(edge),
                        });

                        next_paths.push(new_path.clone());
                        all_paths.push(new_path);
                    }
                }
            }

            current_paths = next_paths;
            if current_paths.is_empty() {
                break;
            }
        }

        Ok(ExecutionResult::Paths(all_paths))
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
        "All paths executor - finds all paths between vertices"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for AllPathsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("存储未初始化")
    }
}

// Implementation for a ScanVertices executor
pub struct ScanVerticesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    tag_filter: Option<crate::core::Expression>,
    vertex_filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageClient> ScanVerticesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        tag_filter: Option<crate::core::Expression>,
        vertex_filter: Option<crate::core::Expression>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ScanVerticesExecutor".to_string(), storage),
            tag_filter,
            vertex_filter,
            limit,
        }
    }
}

impl<S: StorageClient> Executor<S> for ScanVerticesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(vertices) => Ok(ExecutionResult::Vertices(vertices)),
            Err(e) => Err(e),
        }
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
        "ScanVerticesExecutor"
    }

    fn description(&self) -> &str {
        "Scan vertices executor - scans all vertices from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for ScanVerticesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> ScanVerticesExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<vertex_edge_path::Vertex>> {
        let storage = self.get_storage().lock();

        let mut vertices: Vec<vertex_edge_path::Vertex> = storage.scan_vertices("default")?
            .into_iter()
            .filter(|vertex| {
                if let Some(ref tag_filter_expression) = self.tag_filter {
                    crate::query::executor::tag_filter::TagFilterProcessor
                        ::process_tag_filter(tag_filter_expression, vertex)
                } else {
                    true
                }
            })
            .filter(|vertex| {
                if let Some(ref filter_expression) = self.vertex_filter {
                    let mut context = crate::expression::DefaultExpressionContext::new();
                    context.set_variable(
                        "vertex".to_string(),
                        crate::core::Value::Vertex(Box::new(vertex.clone())),
                    );

                    match crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expression, &mut context) {
                        Ok(value) => {
                            match value {
                                crate::core::Value::Bool(b) => b,
                                _ => false,
                            }
                        }
                        Err(_) => false,
                    }
                } else {
                    true
                }
            })
            .collect();

        if let Some(limit) = self.limit {
            vertices.truncate(limit);
        }

        Ok(vertices)
    }
}
