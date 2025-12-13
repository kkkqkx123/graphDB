use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, Vertex, Edge};
use crate::storage::StorageEngine;
use crate::query::QueryError;
use crate::query::executor::traits::{Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata, ExecutionResult, DBResult};

// Context for execution - holds variables and intermediate results
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, Value>,
    pub results: HashMap<String, ExecutionResult>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            results: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }
}

// Base executor with common functionality
#[derive(Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub storage: Arc<Mutex<S>>,
    pub context: ExecutionContext,
}

impl<S: StorageEngine> BaseExecutor<S> {
    pub fn new(id: usize, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage,
            context: ExecutionContext::new(),
        }
    }

    pub fn with_context(id: usize, name: String, storage: Arc<Mutex<S>>, context: ExecutionContext) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage,
            context,
        }
    }

    pub fn with_description(id: usize, name: String, description: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description,
            storage,
            context: ExecutionContext::new(),
        }
    }

    pub fn with_context_and_description(id: usize, name: String, description: String, storage: Arc<Mutex<S>>, context: ExecutionContext) -> Self {
        Self {
            id,
            name,
            description,
            storage,
            context,
        }
    }
}

// Trait for executors that process input from other executors
pub trait InputExecutor<S: StorageEngine> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>);
    fn get_input(&self) -> Option<&Box<dyn Executor<S>>>;
}

// Trait for executors that can be chained together
pub trait ChainableExecutor<S: StorageEngine + Send + 'static>: Executor<S> + InputExecutor<S> {
    fn chain(mut self, next: Box<dyn Executor<S>>) -> Box<dyn Executor<S>>
    where
        Self: Sized + 'static,
    {
        self.set_input(next);
        Box::new(self)
    }
}

// Edge direction enum for neighbor queries
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    In,
    Out,
    Both,
}

// Implementation for StartExecutor
#[derive(Debug)]
pub struct StartExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
}

impl<S: StorageEngine> StartExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::with_description(id, "StartExecutor".to_string(), "Start executor - provides initial execution context".to_string(), storage),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for StartExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // StartExecutor typically produces an initial result set or provides a starting point
        // For initial implementation, we can return a simple success or empty result
        Ok(ExecutionResult::Success)
    }
}

impl<S: StorageEngine> ExecutorLifecycle for StartExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources needed for the start executor
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up any resources
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine> ExecutorMetadata for StartExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for StartExecutor<S> {
    fn storage(&self) -> &S {
        // This is a bit tricky because we have Arc<Mutex<S>>
        // For now, we'll panic if called, but this should be redesigned
        panic!("StartExecutor doesn't provide direct storage access")
    }
}

// Legacy ExecutionResult for backward compatibility
#[derive(Debug, Clone)]
pub enum OldExecutionResult {
    Vertices(Vec<Vertex>),
    Edges(Vec<Edge>),
    Values(Vec<Value>),
    Paths(Vec<crate::core::vertex_edge_path::Path>),
    DataSet(crate::core::value::DataSet),
    Count(usize),
    Success,
}

// Helper functions for working with OldExecutionResult
impl OldExecutionResult {
    pub fn is_empty(&self) -> bool {
        match self {
            OldExecutionResult::Vertices(v) => v.is_empty(),
            OldExecutionResult::Edges(e) => e.is_empty(),
            OldExecutionResult::Values(v) => v.is_empty(),
            OldExecutionResult::Paths(p) => p.is_empty(),
            OldExecutionResult::DataSet(ds) => ds.rows.is_empty(),
            OldExecutionResult::Count(c) => *c == 0,
            OldExecutionResult::Success => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            OldExecutionResult::Vertices(v) => v.len(),
            OldExecutionResult::Edges(e) => e.len(),
            OldExecutionResult::Values(v) => v.len(),
            OldExecutionResult::Paths(p) => p.len(),
            OldExecutionResult::DataSet(ds) => ds.rows.len(),
            OldExecutionResult::Count(c) => *c,
            OldExecutionResult::Success => 0,
        }
    }

    pub fn count(&self) -> usize {
        self.len()
    }
}