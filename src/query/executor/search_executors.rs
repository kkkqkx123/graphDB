use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{Path, Value};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// FulltextIndexScanExecutor - 全文索引扫描执行器
///
/// 用于执行全文索引扫描操作
pub struct FulltextIndexScanExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    index_name: String,
    pattern: String,
    limit: Option<usize>,
}

impl<S: StorageEngine> FulltextIndexScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: &str,
        pattern: &str,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FulltextIndexScanExecutor".to_string(), storage),
            index_name: index_name.to_string(),
            pattern: pattern.to_string(),
            limit,
        }
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn limit(&self) -> Option<usize> {
        self.limit
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for FulltextIndexScanExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())
            .expect("FulltextIndexScanExecutor storage lock should not be poisoned");

        let vertices = storage.scan_all_vertices()?;

        let pattern_lower = self.pattern.to_lowercase();
        let mut matched_vertices = Vec::new();

        for vertex in vertices {
            let vertex_text = format!("{:?}", vertex).to_lowercase();
            if vertex_text.contains(&pattern_lower) {
                matched_vertices.push(vertex);
            }

            if let Some(limit) = self.limit {
                if matched_vertices.len() >= limit {
                    break;
                }
            }
        }

        let rows: Vec<Vec<Value>> = matched_vertices
            .into_iter()
            .map(|v| vec![Value::Vertex(Box::new(v))])
            .collect();

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

impl<S: StorageEngine + Send + 'static> HasStorage<S> for FulltextIndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}

/// BFSShortestExecutor - BFS最短路径执行器
///
/// 使用广度优先搜索算法查找最短路径
pub struct BFSShortestExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    start_vertex_id: Value,
    end_vertex_id: Value,
    max_depth: Option<usize>,
    shortest_paths: Vec<Path>,
    nodes_visited: usize,
    edges_traversed: usize,
    execution_time_ms: u64,
    max_depth_reached: usize,
}

impl<S: StorageEngine> BFSShortestExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vertex_id: Value,
        end_vertex_id: Value,
        max_depth: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "BFSShortestExecutor".to_string(), storage),
            start_vertex_id,
            end_vertex_id,
            max_depth,
            shortest_paths: Vec::new(),
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
            max_depth_reached: 0,
        }
    }

    pub fn start_vertex_id(&self) -> &Value {
        &self.start_vertex_id
    }

    pub fn end_vertex_id(&self) -> &Value {
        &self.end_vertex_id
    }

    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    pub fn shortest_paths(&self) -> &[Path] {
        &self.shortest_paths
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

    pub fn max_depth_reached(&self) -> usize {
        self.max_depth_reached
    }

    fn bfs_shortest_path(&mut self) -> DBResult<Path> {
        let storage_ref = self.get_storage().clone();
        let storage = safe_lock(&storage_ref)
            .expect("BFSShortestExecutor storage lock should not be poisoned");

        let mut queue: VecDeque<(Value, Path)> = VecDeque::new();
        let mut visited: HashMap<Value, bool> = HashMap::new();

        let start_vertex = storage.get_node(&self.start_vertex_id)?;
        if let Some(vertex) = start_vertex {
            queue.push_back((self.start_vertex_id.clone(), Path::new(vertex)));
            visited.insert(self.start_vertex_id.clone(), true);
        }

        while let Some((current_vertex_id, current_path)) = queue.pop_front() {
            self.nodes_visited += 1;

            if current_vertex_id == self.end_vertex_id {
                return Ok(current_path);
            }

            if let Some(max_depth) = self.max_depth {
                if current_path.len() >= max_depth {
                    continue;
                }
            }

            let edges = storage.get_node_edges(&current_vertex_id, crate::core::EdgeDirection::Both)?;
            for edge in edges {
                self.edges_traversed +=1;
                let neighbor_id = *edge.dst.clone();

                if !visited.contains_key(&neighbor_id) {
                    visited.insert(neighbor_id.clone(), true);

                    if let Some(neighbor_vertex) = storage.get_node(&neighbor_id)? {
                        let mut new_path = current_path.clone();
                        new_path.add_step(crate::core::vertex_edge_path::Step {
                            dst: Box::new(neighbor_vertex),
                            edge: Box::new(edge),
                        });
                        queue.push_back((neighbor_id, new_path));
                    }
                }
            }
        }

        Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
            "未找到路径".to_string(),
        )))
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for BFSShortestExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        let shortest_path = self.bfs_shortest_path()?;

        self.execution_time_ms = start_time.elapsed().as_millis() as u64;
        self.max_depth_reached = shortest_path.len();
        self.shortest_paths.push(shortest_path.clone());

        let rows = vec![vec![Value::Path(shortest_path)]];

        Ok(ExecutionResult::Values(
            rows.into_iter().flatten().collect()
        ))
    }

    fn open(&mut self) -> DBResult<()> {
        self.shortest_paths.clear();
        self.nodes_visited = 0;
        self.edges_traversed = 0;
        self.execution_time_ms = 0;
        self.max_depth_reached = 0;
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

impl<S: StorageEngine + Send + 'static> HasStorage<S> for BFSShortestExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}
