use super::{ColumnSchema, DataSet, ExecutionContext, ExecutionError, Plan, ResultRow, ResultSetSchema, VecDataSet};
use crate::core::{StorageError, Value, Vertex, EdgeDirection};
use crate::storage::{MemoryStorage, VertexReader, EdgeReader, ScanResult};
use std::sync::Arc;

pub trait StorageReader: VertexReader + EdgeReader {}

impl<T: VertexReader + EdgeReader> StorageReader for T {}

pub trait Executor: Send {
    fn execute(
        &self,
        storage: &Arc<dyn StorageReader>,
        input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError>;
}

pub struct ScanExecutor {
    space: String,
    target: super::nodes::ScanTarget,
    schema: ResultSetSchema,
}

impl ScanExecutor {
    pub fn new(space: String, target: super::nodes::ScanTarget, schema: ResultSetSchema) -> Self {
        Self {
            space,
            target,
            schema,
        }
    }
}

impl Executor for ScanExecutor {
    fn execute(
        &self,
        storage: &Arc<dyn StorageReader>,
        _input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError> {
        match &self.target {
            super::nodes::ScanTarget::AllVertices => {
                let result = storage.scan_vertices(&self.space)?;
                let rows: Vec<ResultRow> = result
                    .into_iter()
                    .map(|v| {
                        let props = v.properties.clone();
                        ResultRow::new(vec![
                            v.vid.as_ref().clone(),
                            Value::Map(props),
                        ])
                    })
                    .collect();
                Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
            }
            super::nodes::ScanTarget::VerticesByTag(tag) => {
                let result = storage.scan_vertices_by_tag(&self.space, tag)?;
                let rows: Vec<ResultRow> = result
                    .into_iter()
                    .map(|v| {
                        let props = v.properties.clone();
                        ResultRow::new(vec![
                            v.vid.as_ref().clone(),
                            Value::Map(props),
                        ])
                    })
                    .collect();
                Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
            }
            super::nodes::ScanTarget::AllEdges => {
                let result = storage.scan_all_edges(&self.space)?;
                let rows: Vec<ResultRow> = result
                    .into_iter()
                    .map(|e| {
                        let props = e.properties().clone();
                        ResultRow::new(vec![
                            e.src().clone(),
                            e.dst().clone(),
                            Value::Int(e.ranking),
                            Value::Map(props),
                        ])
                    })
                    .collect();
                Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
            }
            super::nodes::ScanTarget::EdgesByType(edge_type) => {
                let result = storage.scan_edges_by_type(&self.space, edge_type)?;
                let rows: Vec<ResultRow> = result
                    .into_iter()
                    .map(|e| {
                        let props = e.properties().clone();
                        ResultRow::new(vec![
                            e.src().clone(),
                            e.dst().clone(),
                            Value::Int(e.ranking),
                            Value::Map(props),
                        ])
                    })
                    .collect();
                Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
            }
        }
    }
}

impl Plan for ScanExecutor {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError> {
        Err(ExecutionError {
            message: "ScanExecutor should be executed via plan executor".to_string(),
            cause: None,
        })
    }

    fn schema(&self) -> &ResultSetSchema {
        &self.schema
    }
}

pub struct GetNeighborsExecutor {
    space: String,
    src_vertex: String,
    edge_type: String,
    direction: EdgeDirection,
    schema: ResultSetSchema,
}

impl GetNeighborsExecutor {
    pub fn new(
        space: String,
        src_vertex: String,
        edge_type: String,
        direction: EdgeDirection,
        schema: ResultSetSchema,
    ) -> Self {
        Self {
            space,
            src_vertex,
            edge_type,
            direction,
            schema,
        }
    }
}

impl Executor for GetNeighborsExecutor {
    fn execute(
        &self,
        storage: &Arc<dyn StorageReader>,
        _input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError> {
        let src_value = Value::String(self.src_vertex.clone());
        let result = storage.get_node_edges(&self.space, &src_value, self.direction)?;

        let filtered: Vec<_> = if self.edge_type.is_empty() {
            result.into_iter().collect()
        } else {
            result.into_iter().filter(|e| e.edge_type == self.edge_type).collect()
        };

        let rows: Vec<ResultRow> = filtered
            .into_iter()
            .map(|e| {
                let props = e.properties().clone();
                ResultRow::new(vec![
                    e.src().clone(),
                    e.dst().clone(),
                    Value::Int(e.ranking),
                    Value::Map(props),
                ])
            })
            .collect();

        Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
    }
}

impl Plan for GetNeighborsExecutor {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError> {
        Err(ExecutionError {
            message: "GetNeighborsExecutor should be executed via plan executor".to_string(),
            cause: None,
        })
    }

    fn schema(&self) -> &ResultSetSchema {
        &self.schema
    }
}

pub struct LimitExecutor {
    offset: usize,
    count: usize,
    schema: ResultSetSchema,
}

impl LimitExecutor {
    pub fn new(offset: usize, count: usize, schema: ResultSetSchema) -> Self {
        Self {
            offset,
            count,
            schema,
        }
    }
}

impl Executor for LimitExecutor {
    fn execute(
        &self,
        _storage: &Arc<dyn StorageReader>,
        input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError> {
        let input = input.ok_or_else(|| ExecutionError {
            message: "Limit requires input".to_string(),
            cause: None,
        })?;

        let input_rows = input.rows();
        let start = std::cmp::min(self.offset, input_rows.len());
        let end = std::cmp::min(start + self.count, input_rows.len());

        let rows: Vec<ResultRow> = input_rows[start..end].to_vec();

        Ok(Box::new(VecDataSet::new(self.schema.clone(), rows)))
    }
}

impl Plan for LimitExecutor {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError> {
        Err(ExecutionError {
            message: "LimitExecutor should not be executed directly".to_string(),
            cause: None,
        })
    }

    fn schema(&self) -> &ResultSetSchema {
        &self.schema
    }
}

pub struct DummyExecutor;

impl Executor for DummyExecutor {
    fn execute(
        &self,
        _storage: &Arc<dyn StorageReader>,
        _input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError> {
        Ok(Box::new(VecDataSet::empty(ResultSetSchema {
            columns: Vec::new(),
        })))
    }
}

pub struct SimplePlanExecutor {
    plan: Box<dyn Plan>,
    storage: Arc<MemoryStorage>,
}

impl SimplePlanExecutor {
    pub fn new(plan: Box<dyn Plan>, storage: Arc<MemoryStorage>) -> Self {
        Self { plan, storage }
    }

    pub fn execute(&self) -> Result<Box<dyn DataSet>, ExecutionError> {
        let ctx = ExecutionContext::new(
            "test_space".to_string(),
            self.storage.schema_manager.clone(),
        );
        self.plan.execute(&ctx)
    }
}
