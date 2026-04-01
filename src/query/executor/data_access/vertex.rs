use std::sync::Arc;
use std::time::Instant;

use super::super::base::{BaseExecutor, ExecutorStats};
use crate::core::{vertex_edge_path, Value};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// Parameters for creating GetVerticesExecutor
pub struct GetVerticesParams {
    pub space_name: String,
    pub vertex_ids: Option<Vec<Value>>,
    pub tag_filter: Option<crate::core::Expression>,
    pub vertex_filter: Option<crate::core::Expression>,
    pub limit: Option<usize>,
}

impl GetVerticesParams {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            vertex_ids: None,
            tag_filter: None,
            vertex_filter: None,
            limit: None,
        }
    }
}

pub struct GetVerticesExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    space_name: String,
    vertex_ids: Option<Vec<Value>>,
    tag_filter: Option<crate::core::Expression>,
    vertex_filter: Option<crate::core::Expression>,
    limit: Option<usize>,
}

impl<S: StorageClient + 'static> GetVerticesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        params: GetVerticesParams,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetVerticesExecutor".to_string(), storage, expr_context),
            space_name: params.space_name,
            vertex_ids: params.vertex_ids,
            tag_filter: params.tag_filter,
            vertex_filter: params.vertex_filter,
            limit: params.limit,
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
                    match storage.get_vertex(&self.space_name, id) {
                        Ok(Some(vertex)) => {
                            let include_vertex =
                                if let Some(ref tag_filter_expression) = self.tag_filter {
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
                        Err(e) => {
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

                match storage.get_vertex(&self.space_name, &ids[0]) {
                    Ok(Some(vertex)) => {
                        Ok(vec![vertex])
                    }
                    Ok(None) => {
                        Ok(vec![])
                    }
                    Err(e) => {
                        Err(crate::core::error::DBError::Storage(e))
                    }
                }
            }
            Some(_) => Ok(Vec::new()),
            None => {
                let storage = self.get_storage().lock();

                let vertices = storage.scan_vertices(&self.space_name)?
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
                                crate::query::executor::expression::DefaultExpressionContext::new();
                            context.set_variable(
                                "vertex".to_string(),
                                crate::core::Value::Vertex(Box::new(vertex.clone())),
                            );

                            match crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expression, &mut context) {
                                Ok(value) => {
                                    match value {
                                        crate::core::Value::Bool(b) => b,
                                        crate::core::Value::Int(i) => i != 0,
                                        crate::core::Value::Int8(i) => i != 0,
                                        crate::core::Value::Int16(i) => i != 0,
                                        crate::core::Value::Int32(i) => i != 0,
                                        crate::core::Value::Int64(i) => i != 0,
                                        crate::core::Value::UInt8(i) => i != 0,
                                        crate::core::Value::UInt16(i) => i != 0,
                                        crate::core::Value::UInt32(i) => i != 0,
                                        crate::core::Value::UInt64(i) => i != 0,
                                        crate::core::Value::Float(f) => f != 0.0,
                                        crate::core::Value::Decimal128(d) => !d.is_zero(),
                                        crate::core::Value::String(s) => !s.is_empty(),
                                        crate::core::Value::FixedString { data, .. } => !data.is_empty(),
                                        crate::core::Value::Blob(b) => !b.is_empty(),
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
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "ScanVerticesExecutor".to_string(),
                storage,
                expr_context,
            ),
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
                    let mut context = crate::query::executor::expression::DefaultExpressionContext::new();
                    context.set_variable(
                        "vertex".to_string(),
                        crate::core::Value::Vertex(Box::new(vertex.clone())),
                    );

                    match crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expression, &mut context) {
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
