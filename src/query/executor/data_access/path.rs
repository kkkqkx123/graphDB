use std::sync::Arc;

use super::super::base::{BaseExecutor, EdgeDirection, ExecutorStats};
use crate::core::{Path, Step, Value};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

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
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AllPathsExecutor".to_string(), storage, expr_context),
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

        let start_vertex_obj =
            if let Some(vertex) = storage.get_vertex("default", &self.start_vertex)? {
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
        self.base.storage.as_ref().expect("存储未初始化")
    }
}
