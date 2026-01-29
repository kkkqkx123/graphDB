use crate::core::{StorageError, Value};
use crate::storage::SchemaManager;
use std::collections::HashMap;
use std::sync::Arc;

pub mod nodes;
pub mod executors;

pub use nodes::*;
pub use executors::*;

pub struct ExecutionContext {
    pub space: String,
    pub schema_manager: Arc<dyn SchemaManager>,
    pub runtime: HashMap<String, Box<dyn std::any::Any>>,
}

impl ExecutionContext {
    pub fn new(space: String, schema_manager: Arc<dyn SchemaManager>) -> Self {
        Self {
            space,
            schema_manager,
            runtime: HashMap::new(),
        }
    }

    pub fn set_runtime<T: 'static>(&mut self, key: &str, value: T) {
        self.runtime.insert(key.to_string(), Box::new(value));
    }

    pub fn get_runtime<T: 'static>(&self, key: &str) -> Option<&T> {
        self.runtime.get(key).and_then(|v| v.downcast_ref())
    }
}

pub trait DataSet: Send {
    fn schema(&self) -> &ResultSetSchema;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn into_rows(self) -> Vec<ResultRow>;
    fn rows(&self) -> &[ResultRow];
    fn rows_mut(&mut self) -> &mut [ResultRow];
}

#[derive(Clone, Debug)]
pub struct ResultSetSchema {
    pub columns: Vec<ColumnSchema>,
}

#[derive(Clone, Debug)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: crate::core::DataType,
    pub nullable: bool,
}

#[derive(Clone, Debug)]
pub struct ResultRow {
    pub values: Vec<Value>,
}

impl ResultRow {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn get_by_name(&self, name: &str, schema: &ResultSetSchema) -> Option<&Value> {
        schema.columns.iter().enumerate()
            .find(|(_, col)| col.name == name)
            .and_then(|(idx, _)| self.values.get(idx))
    }
}

#[derive(Clone)]
pub struct VecDataSet {
    schema: ResultSetSchema,
    rows: Vec<ResultRow>,
}

impl VecDataSet {
    pub fn new(schema: ResultSetSchema, rows: Vec<ResultRow>) -> Self {
        Self { schema, rows }
    }

    pub fn empty(schema: ResultSetSchema) -> Self {
        Self {
            schema,
            rows: Vec::new(),
        }
    }
}

impl DataSet for VecDataSet {
    fn schema(&self) -> &ResultSetSchema {
        &self.schema
    }

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn into_rows(self) -> Vec<ResultRow> {
        self.rows
    }

    fn rows(&self) -> &[ResultRow] {
        &self.rows
    }

    fn rows_mut(&mut self) -> &mut [ResultRow] {
        &mut self.rows
    }
}

#[derive(Debug)]
pub struct ExecutionError {
    pub message: String,
    pub cause: Option<StorageError>,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Execution error: {}", self.message)
    }
}

impl std::error::Error for ExecutionError {}

impl From<StorageError> for ExecutionError {
    fn from(err: StorageError) -> Self {
        Self {
            message: err.to_string(),
            cause: Some(err),
        }
    }
}

pub trait Plan: Send + Sync {
    fn execute(&self, ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError>;
    fn schema(&self) -> &ResultSetSchema;
}
