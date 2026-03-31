use std::sync::Arc;
use std::time::Instant;

use super::super::base::{BaseExecutor, ExecutorStats};
use crate::core::vertex_edge_path;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

pub struct GetEdgesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    edge_type: Option<String>,
}

impl<S: StorageClient> GetEdgesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_type: Option<String>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetEdgesExecutor".to_string(), storage, expr_context),
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
            let result = storage.scan_edges_by_type("default", edge_type)?;
            println!("[GetEdgesExecutor] scan_edges_by_type({:?}) returned {} edges", edge_type, result.len());
            for e in &result {
                println!("[GetEdgesExecutor]   edge: {} -> {} type={} rank={}", e.src(), e.dst(), e.edge_type, e.ranking());
            }
            result
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
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ScanEdgesExecutor".to_string(), storage, expr_context),
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
            let mut context = crate::query::executor::expression::DefaultExpressionContext::new();
            edges.retain(|edge| {
                context.set_variable("edge".to_string(), crate::core::Value::Edge(edge.clone()));
                match crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expr, &mut context) {
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
