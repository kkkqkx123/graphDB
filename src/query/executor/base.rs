use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::core::{Edge, Value, Vertex};
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage};
use crate::storage::StorageEngine;

pub use crate::core::types::EdgeDirection;

// Context for execution - holds intermediate results only
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub results: HashMap<String, ExecutionResult>,
    pub variables: HashMap<String, Value>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

// Base executor with common functionality
#[derive(Clone, Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub storage: Option<Arc<Mutex<S>>>,
    pub context: ExecutionContext,
    is_open: bool,
    stats: ExecutorStats,
}

impl<S: StorageEngine> BaseExecutor<S> {
    pub fn new(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn without_storage(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: None,
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn with_context(
        id: i64,
        name: String,
        storage: Arc<Mutex<S>>,
        context: ExecutionContext,
    ) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn with_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            storage: Some(storage),
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn with_context_and_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
        context: ExecutionContext,
    ) -> Self {
        Self {
            id,
            name,
            description,
            storage: Some(storage),
            context,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 获取执行统计信息
    pub fn get_stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// 获取可变的执行统计信息
    pub fn get_stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageEngine> HasStorage<S> for BaseExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.storage
            .as_ref()
            .expect("BaseExecutor storage should be set")
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for BaseExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = Ok(ExecutionResult::Success);
        self.stats_mut().add_total_time(start.elapsed());
        result
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

// Trait for executors that process input from other executors
pub trait InputExecutor<S: StorageEngine> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>);
    fn get_input(&self) -> Option<&Box<dyn Executor<S>>>;
}

// Trait for executors that can be chained together
pub trait ChainableExecutor<S: StorageEngine + Send + 'static>:
    Executor<S> + InputExecutor<S>
{
    fn chain(mut self, next: Box<dyn Executor<S>>) -> Box<dyn Executor<S>>
    where
        Self: Sized + 'static,
    {
        self.set_input(next);
        Box::new(self)
    }
}

// Implementation for StartExecutor
#[derive(Debug)]
pub struct StartExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
}

impl<S: StorageEngine> StartExecutor<S> {
    pub fn new(id: i64) -> Self {
        Self {
            base: BaseExecutor::without_storage(id, "StartExecutor".to_string()),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for StartExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = Ok(ExecutionResult::Success);
        self.base.get_stats_mut().add_total_time(start.elapsed());
        result
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

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
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
