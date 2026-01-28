use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::Value;
use crate::expression::ExpressionContext;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

// Implementation for a basic GetVertices executor
#[derive(Debug)]
pub struct GetVerticesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    tag_filter: Option<crate::core::Expression>,
    vertex_filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageEngine> GetVerticesExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let vertices = match &self.vertex_ids {
            Some(ids) => {
                let storage = safe_lock(self.get_storage())
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                let capacity = self.limit.map_or(ids.len(), |limit| limit.min(ids.len()));
                let mut result_vertices: Vec<crate::core::vertex_edge_path::Vertex> =
                    Vec::with_capacity(capacity);

                for id in ids {
                    if let Some(vertex) = storage.get_node(id)? {
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

                    if let Some(limit) = self.limit {
                        if result_vertices.len() >= limit {
                            break;
                        }
                    }
                }
                result_vertices
            }
            None => {
                let storage = safe_lock(self.get_storage())
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                storage.scan_all_vertices()?
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
                                    eprintln!("顶点过滤表达式评估失败: {}", e);
                                    false
                                }
                            }
                        } else {
                            true
                        }
                    })
                    .take(self.limit.unwrap_or(usize::MAX))
                    .collect()
            }
        };

        Ok(ExecutionResult::Values(
            vertices
                .into_iter()
                .map(|v| crate::core::Value::Vertex(Box::new(v)))
                .collect(),
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
        "Get vertices executor - retrieves vertices from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for GetVerticesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("GetVerticesExecutor storage should be set")
    }
}

// Implementation for a basic GetEdges executor
#[derive(Debug)]
pub struct GetEdgesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,

    edge_type: Option<String>,
}

impl<S: StorageEngine> GetEdgesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, edge_type: Option<String>) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetEdgesExecutor".to_string(), storage),
            edge_type,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetEdgesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("GetEdgesExecutor storage lock should not be poisoned");

        let edges = if let Some(ref edge_type) = self.edge_type {
            storage.scan_edges_by_type(edge_type)?
        } else {
            storage.scan_all_edges()?
        };

        let values: Vec<crate::core::Value> = edges
            .into_iter()
            .map(|e| crate::core::Value::Edge(e))
            .collect();

        Ok(ExecutionResult::Values(values))
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
        "Get edges executor - retrieves edges from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for GetEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("GetEdgesExecutor storage should be set")
    }
}

// Implementation for ScanEdges executor
#[derive(Debug)]
pub struct ScanEdgesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    edge_type: Option<String>,
    filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageEngine> ScanEdgesExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ScanEdgesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("ScanEdgesExecutor storage lock should not be poisoned");

        let mut edges: Vec<crate::core::vertex_edge_path::Edge> = if let Some(ref edge_type) = self.edge_type {
            storage.scan_edges_by_type(edge_type)?
        } else {
            storage.scan_all_edges()?
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

        let values: Vec<crate::core::Value> = edges
            .into_iter()
            .map(|e| crate::core::Value::Edge(e))
            .collect();

        Ok(ExecutionResult::Values(values))
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
        "Scan edges executor - scans all edges from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for ScanEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ScanEdgesExecutor storage should be set")
    }
}

// Implementation for a basic GetNeighbors executor
#[derive(Debug)]
pub struct GetNeighborsExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,

    vertex_ids: Vec<Value>,

    edge_direction: super::base::EdgeDirection, // Direction: In, Out, or Both

    edge_types: Option<Vec<String>>,
}

impl<S: StorageEngine> GetNeighborsExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetNeighborsExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("GetNeighborsExecutor storage lock should not be poisoned");

        let mut neighbors = Vec::new();

        let edge_types_filter = self.edge_types.as_ref();

        for vertex_id in &self.vertex_ids {
            let direction = self.edge_direction;

            let edges = storage.get_node_edges(vertex_id, direction)?;

            for edge in edges {
                if let Some(ref filter_types) = edge_types_filter {
                    if !filter_types.contains(&edge.edge_type) {
                        continue;
                    }
                }

                let neighbor_id = if *edge.src == *vertex_id {
                    &edge.dst
                } else {
                    &edge.src
                };

                if let Some(vertex) = storage.get_node(neighbor_id)? {
                    neighbors.push(crate::core::Value::Vertex(Box::new(vertex)));
                }
            }
        }

        Ok(ExecutionResult::Values(neighbors))
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
        "Get neighbors executor - retrieves neighboring vertices"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for GetNeighborsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("GetNeighborsExecutor storage should be set")
    }
}

// Implementation for GetPropExecutor
#[derive(Debug)]
pub struct GetPropExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,

    vertex_ids: Option<Vec<Value>>,

    edge_ids: Option<Vec<Value>>,

    prop_names: Vec<String>, // List of property names to retrieve
}

impl<S: StorageEngine> GetPropExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetPropExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("GetPropExecutor storage lock should not be poisoned");

        let mut props = Vec::new();

        if let Some(ref vertex_ids) = self.vertex_ids {
            let total_props = vertex_ids.len() * self.prop_names.len();
            props.reserve(total_props);

            for vertex_id in vertex_ids {
                if let Some(vertex) = storage.get_node(vertex_id)? {
                    for prop_name in &self.prop_names {
                        if let Some(value) = vertex.get_property_any(prop_name) {
                            props.push(value.clone());
                        } else {
                            props
                                .push(crate::core::Value::Null(crate::core::value::NullType::Null));
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
                            props
                                .push(crate::core::Value::Null(crate::core::value::NullType::Null));
                        }
                    }
                }
            }
        }

        Ok(ExecutionResult::Values(props))
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
        "Get property executor - retrieves properties from vertices or edges"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for GetPropExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("GetPropExecutor storage should be set")
    }
}

use crate::core::vertex_edge_path::{Path, Step};

use super::base::EdgeDirection;

#[derive(Debug)]
pub struct IndexScanExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    index_name: String,
    index_condition: Option<(String, Value)>,
    scan_forward: bool,
    limit: Option<usize>,
}

impl<S: StorageEngine> IndexScanExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for IndexScanExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("IndexScanExecutor storage lock should not be poisoned");

        let mut results = Vec::new();

        if let Some((prop_name, prop_value)) = &self.index_condition {
            let scan_results = storage.scan_vertices_by_prop(&self.index_name, prop_name, prop_value)?;

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
                storage.scan_vertices_by_tag(&self.index_name)?
            } else {
                storage.scan_all_vertices()?
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

        Ok(ExecutionResult::Values(results))
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
        "Index scan executor - retrieves vertices using index"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for IndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("IndexScanExecutor storage should be set")
    }
}

#[derive(Debug)]
pub struct AllPathsExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    start_vertex: Value,
    end_vertex: Option<Value>,
    max_hops: usize,
    edge_types: Option<Vec<String>>,
    direction: EdgeDirection,
}

impl<S: StorageEngine> AllPathsExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AllPathsExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("AllPathsExecutor storage lock should not be poisoned");

        let mut all_paths: Vec<Path> = Vec::new();

        let start_vertex_obj = if let Some(vertex) = storage.get_node(&self.start_vertex)? {
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

                let edges = storage.get_node_edges(&self.start_vertex, direction)?;

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

                    if let Some(neighbor) = storage.get_node(&neighbor_id)? {
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

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for AllPathsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("AllPathsExecutor storage should be set")
    }
}
