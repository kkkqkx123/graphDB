use crate::core::error::DBError;
use crate::core::{Edge, Path, Value, Vertex};
use crate::storage::StorageClient;
use crate::utils::safe_lock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

/// Global context for the graph database
pub struct GraphContext<S: StorageClient> {
    pub config: crate::config::Config,
    pub storage: Arc<Mutex<S>>,
    /// Session-specific variables
    pub session_vars: Arc<Mutex<HashMap<String, Value>>>,
    /// Statistics and metrics
    pub metrics: Arc<Mutex<Metrics>>,
    /// Execution context for current operations
    pub execution_context: Arc<Mutex<ExecutionContext>>,
}

/// Metrics and statistics about database operations
#[derive(Debug, Clone)]
pub struct Metrics {
    pub vertices_created: u64,
    pub edges_created: u64,
    pub vertices_read: u64,
    pub edges_read: u64,
    pub vertices_updated: u64,
    pub edges_updated: u64,
    pub vertices_deleted: u64,
    pub edges_deleted: u64,
    pub queries_executed: u64,
    pub errors_occurred: u64,
    pub total_execution_time_ms: u64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            vertices_created: 0,
            edges_created: 0,
            vertices_read: 0,
            edges_read: 0,
            vertices_updated: 0,
            edges_updated: 0,
            vertices_deleted: 0,
            edges_deleted: 0,
            queries_executed: 0,
            errors_occurred: 0,
            total_execution_time_ms: 0,
        }
    }

    pub fn increment_vertices_created(&mut self) {
        self.vertices_created += 1;
    }

    pub fn increment_edges_created(&mut self) {
        self.edges_created += 1;
    }

    pub fn increment_vertices_read(&mut self) {
        self.vertices_read += 1;
    }

    pub fn increment_edges_read(&mut self) {
        self.edges_read += 1;
    }

    pub fn increment_queries_executed(&mut self) {
        self.queries_executed += 1;
    }

    pub fn increment_errors_occurred(&mut self) {
        self.errors_occurred += 1;
    }

    pub fn add_execution_time(&mut self, time_ms: u64) {
        self.total_execution_time_ms += time_ms;
    }
}

/// Execution context for current operation
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub query_id: String,
    pub start_time: std::time::SystemTime,
    pub timeout: Duration,
    pub current_vertex: Option<Vertex>,
    pub current_edge: Option<Edge>,
    pub current_path: Option<Path>,
    pub variables: HashMap<String, Value>,
}

impl ExecutionContext {
    pub fn new(query_id: String) -> Self {
        Self {
            query_id,
            start_time: std::time::SystemTime::now(),
            timeout: Duration::from_secs(30), // Default 30 second timeout
            current_vertex: None,
            current_edge: None,
            current_path: None,
            variables: HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn check_timeout(&self) -> Result<(), String> {
        match self.start_time.elapsed() {
            Ok(elapsed) => {
                if elapsed > self.timeout {
                    Err("Query execution timed out".to_string())
                } else {
                    Ok(())
                }
            }
            Err(_) => Err("Error checking elapsed time".to_string()),
        }
    }
}

impl<S: StorageClient> GraphContext<S> {
    pub fn new(config: crate::config::Config, storage: S) -> Self {
        Self {
            config,
            storage: Arc::new(Mutex::new(storage)),
            session_vars: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Metrics::new())),
            execution_context: Arc::new(Mutex::new(ExecutionContext::new("default".to_string()))),
        }
    }

    pub fn with_execution_context(&self, execution_context: ExecutionContext) -> Self {
        Self {
            config: self.config.clone(),
            storage: Arc::clone(&self.storage),
            session_vars: Arc::clone(&self.session_vars),
            metrics: Arc::clone(&self.metrics),
            execution_context: Arc::new(Mutex::new(execution_context)),
        }
    }

    pub fn increment_vertices_created(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_vertices_created();
        Ok(())
    }

    pub fn increment_edges_created(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_edges_created();
        Ok(())
    }

    pub fn increment_vertices_read(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_vertices_read();
        Ok(())
    }

    pub fn increment_edges_read(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_edges_read();
        Ok(())
    }

    pub fn increment_queries_executed(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_queries_executed();
        Ok(())
    }

    pub fn increment_errors_occurred(&self) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.increment_errors_occurred();
        Ok(())
    }

    pub fn add_execution_time(&self, time_ms: u64) -> Result<(), DBError> {
        let mut metrics = safe_lock(&self.metrics)?;
        metrics.add_execution_time(time_ms);
        Ok(())
    }

    /// Get a session variable
    pub fn get_session_var(&self, key: &str) -> Result<Option<Value>, DBError> {
        let vars = safe_lock(&self.session_vars)?;
        Ok(vars.get(key).cloned())
    }

    /// Set a session variable
    pub fn set_session_var(&self, key: String, value: Value) -> Result<(), DBError> {
        let mut vars = safe_lock(&self.session_vars)?;
        vars.insert(key, value);
        Ok(())
    }

    /// Remove a session variable
    pub fn remove_session_var(&self, key: &str) -> Result<Option<Value>, DBError> {
        let mut vars = safe_lock(&self.session_vars)?;
        Ok(vars.remove(key))
    }

    /// Wait for the context to be ready or timeout
    pub async fn wait_ready(&self, timeout: Duration) -> Result<(), DBError> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            // Check if our storage engine is ready
            {
                let _storage = safe_lock(&self.storage)?;
                // Since NativeStorage doesn't have an is_operational method, we'll just try a basic operation
                // to verify that the storage is accessible
                // Just drop the lock without doing anything
            }

            time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MockStorage;

    #[test]
    fn test_metrics() {
        let mut metrics = Metrics::new();
        metrics.increment_vertices_created();
        metrics.increment_edges_created();

        assert_eq!(metrics.vertices_created, 1);
        assert_eq!(metrics.edges_created, 1);
    }

    #[tokio::test]
    async fn test_graph_context() {
        let config = crate::config::Config::default();
        let storage = MockStorage;
        let ctx = GraphContext::new(config, storage);

        ctx.increment_vertices_created()
            .expect("Failed to increment vertices created");
        ctx.increment_edges_created()
            .expect("Failed to increment edges created");

        {
            let metrics = safe_lock(&ctx.metrics).expect("Failed to lock metrics for test");
            assert_eq!(metrics.vertices_created, 1);
            assert_eq!(metrics.edges_created, 1);
        }

        // Test session variables
        ctx.set_session_var("test_key".to_string(), Value::Int(42))
            .expect("Failed to set session variable");
        let value = ctx
            .get_session_var("test_key")
            .expect("Failed to get session variable");
        assert_eq!(value, Some(Value::Int(42)));
    }
}
