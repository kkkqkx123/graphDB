use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, Vertex, Edge};
use crate::storage::StorageEngine;
use crate::query::QueryError;

// Base executor trait that all executors should implement
#[async_trait]
pub trait Executor<S: StorageEngine + Send + 'static>: Send + Sync {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError>;

    // Prepare for execution
    fn open(&mut self) -> Result<(), QueryError> {
        Ok(())
    }

    // Clean up after execution
    fn close(&mut self) -> Result<(), QueryError> {
        Ok(())
    }

    // Get the ID of this executor
    fn id(&self) -> usize;

    // Get the name of this executor
    fn name(&self) -> &str;
}

// Result of executor execution
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Vertices(Vec<Vertex>),
    Edges(Vec<Edge>),
    Values(Vec<Value>),
    Count(usize),
    Success,
}

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
pub struct BaseExecutor<S: StorageEngine> {
    pub id: usize,
    pub name: String,
    pub storage: Arc<Mutex<S>>,
    pub context: ExecutionContext,
}

impl<S: StorageEngine> BaseExecutor<S> {
    pub fn new(id: usize, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            storage,
            context: ExecutionContext::new(),
        }
    }

    pub fn with_context(id: usize, name: String, storage: Arc<Mutex<S>>, context: ExecutionContext) -> Self {
        Self {
            id,
            name,
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

// Helper functions for working with ExecutionResult
impl ExecutionResult {
    pub fn is_empty(&self) -> bool {
        match self {
            ExecutionResult::Vertices(v) => v.is_empty(),
            ExecutionResult::Edges(e) => e.is_empty(),
            ExecutionResult::Values(v) => v.is_empty(),
            ExecutionResult::Count(c) => *c == 0,
            ExecutionResult::Success => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ExecutionResult::Vertices(v) => v.len(),
            ExecutionResult::Edges(e) => e.len(),
            ExecutionResult::Values(v) => v.len(),
            ExecutionResult::Count(c) => *c,
            ExecutionResult::Success => 0,
        }
    }

    pub fn count(&self) -> usize {
        self.len()
    }
}