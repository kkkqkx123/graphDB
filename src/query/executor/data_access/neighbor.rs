use std::sync::Arc;
use std::time::Instant;

use super::super::base::{BaseExecutor, EdgeDirection, ExecutorStats};
use crate::core::Value;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::DataSet;
use crate::storage::StorageClient;
use parking_lot::Mutex;

pub struct GetNeighborsExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    vertex_ids: Vec<Value>,
    edge_direction: EdgeDirection,
    edge_types: Option<Vec<String>>,
}

impl<S: StorageClient> GetNeighborsExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "GetNeighborsExecutor".to_string(),
                storage,
                expr_context,
            ),
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
            Ok(values) => {
                let dataset = DataSet::from_rows(
                    values.into_iter().map(|v| vec![v]).collect(),
                    vec!["value".to_string()],
                );
                Ok(ExecutionResult::DataSet(dataset))
            }
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
                if let Some(filter_types) = edge_types_filter {
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
            log::warn!("Failed to get neighbor vertices: {} ", failed_count);
        }

        Ok(neighbors)
    }
}
