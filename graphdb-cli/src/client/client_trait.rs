//! GraphDbClient trait - HTTP client interface for database connections
//!
//! This trait provides an interface for HTTP remote connections to GraphDB server.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::client::http::{EdgeTypeInfo, QueryResult, SpaceInfo, TagInfo};
use crate::utils::error::Result;

/// Session information returned after successful connection
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: i64,
    pub username: String,
    pub host: String,
    pub port: u16,
}

/// Configuration for client connections
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub timeout_seconds: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            username: "root".to_string(),
            password: String::new(),
            timeout_seconds: 30,
        }
    }
}

impl ClientConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = username.into();
        self.password = password.into();
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Transaction options for beginning a transaction
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub read_only: bool,
    pub timeout_seconds: Option<u64>,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            read_only: false,
            timeout_seconds: None,
        }
    }
}

impl TransactionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
}

/// Transaction information returned after beginning a transaction
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub transaction_id: u64,
    pub status: String,
}

/// Property definition for schema creation
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl PropertyDef {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }
}

/// Data types supported for properties
#[derive(Debug, Clone)]
pub enum DataType {
    Bool,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    String,
    Date,
    Time,
    DateTime,
    Timestamp,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Bool => write!(f, "BOOL"),
            DataType::SmallInt => write!(f, "SMALLINT"),
            DataType::Int => write!(f, "INT"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::String => write!(f, "STRING"),
            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
        }
    }
}

/// Statistics for a session
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_queries: u64,
    pub total_changes: u64,
    pub avg_execution_time_ms: f64,
}

/// Query type statistics
#[derive(Debug, Clone)]
pub struct QueryTypeStatistics {
    pub match_queries: u64,
    pub create_queries: u64,
    pub update_queries: u64,
    pub delete_queries: u64,
    pub insert_queries: u64,
    pub go_queries: u64,
    pub fetch_queries: u64,
    pub lookup_queries: u64,
    pub show_queries: u64,
}

/// Query statistics
#[derive(Debug, Clone)]
pub struct QueryStatistics {
    pub total_queries: u64,
    pub slow_queries: Vec<SlowQueryInfo>,
    pub query_types: QueryTypeStatistics,
}

/// Information about a slow query
#[derive(Debug, Clone)]
pub struct SlowQueryInfo {
    pub trace_id: String,
    pub session_id: i64,
    pub query: String,
    pub duration_ms: f64,
    pub status: String,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub space_count: i64,
    pub total_vertices: i64,
    pub total_edges: i64,
    pub total_queries: u64,
    pub active_queries: u64,
    pub queries_per_second: f64,
    pub avg_latency_ms: f64,
}

/// Batch operation types
#[derive(Debug, Clone)]
pub enum BatchType {
    Vertex,
    Edge,
    Mixed,
}

/// Batch item for bulk operations
#[derive(Debug, Clone)]
pub enum BatchItem {
    Vertex(VertexData),
    Edge(EdgeData),
}

/// Vertex data for batch insertion
#[derive(Debug, Clone)]
pub struct VertexData {
    pub vid: serde_json::Value,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Edge data for batch insertion
#[derive(Debug, Clone)]
pub struct EdgeData {
    pub edge_type: String,
    pub src_vid: serde_json::Value,
    pub dst_vid: serde_json::Value,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Batch operation result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_id: String,
    pub status: String,
    pub vertices_inserted: usize,
    pub edges_inserted: usize,
    pub errors: Vec<BatchError>,
}

/// Batch error information
#[derive(Debug, Clone)]
pub struct BatchError {
    pub index: usize,
    pub item_type: String,
    pub error: String,
}

/// Batch status information
#[derive(Debug, Clone)]
pub struct BatchStatus {
    pub batch_id: String,
    pub status: String,
    pub total: usize,
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// Core trait for GraphDB client operations
///
/// Implementation: `HttpClient` connects to remote GraphDB server via HTTP API
#[async_trait]
pub trait GraphDbClient: Send + Sync {
    /// Check if client is currently connected
    fn is_connected(&self) -> bool;

    /// Connect to the database
    ///
    /// Authenticates with HTTP server
    async fn connect(&mut self) -> Result<SessionInfo>;

    /// Disconnect from the database
    ///
    /// This will properly logout from the server and clean up resources
    async fn disconnect(&mut self) -> Result<()>;

    /// Execute a query and return results
    async fn execute_query(&self, query: &str, session_id: i64) -> Result<QueryResult>;

    /// Execute a query without variable substitution
    async fn execute_query_raw(&self, query: &str, session_id: i64) -> Result<QueryResult>;

    /// List all available spaces
    async fn list_spaces(&self) -> Result<Vec<SpaceInfo>>;

    /// Switch to a specific space
    async fn switch_space(&self, space: &str) -> Result<()>;

    /// List all tags in current space
    async fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>>;

    /// List all edge types in current space
    async fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>>;

    /// Check server/database health
    async fn health_check(&self) -> Result<bool>;

    /// Get base URL
    fn connection_string(&self) -> String;

    // Transaction management methods

    /// Begin a new transaction
    ///
    /// Returns the transaction ID that can be used for commit or rollback
    async fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionInfo>;

    /// Commit a transaction
    ///
    /// # Arguments
    /// * `txn_id` - The transaction ID returned by `begin_transaction`
    async fn commit_transaction(&self, txn_id: u64) -> Result<()>;

    /// Rollback a transaction
    ///
    /// # Arguments
    /// * `txn_id` - The transaction ID returned by `begin_transaction`
    async fn rollback_transaction(&self, txn_id: u64) -> Result<()>;

    // Schema DDL operations

    /// Create a new graph space
    ///
    /// # Arguments
    /// * `name` - Space name
    /// * `vid_type` - Optional vertex ID type (default: STRING)
    /// * `comment` - Optional comment
    async fn create_space(
        &self,
        name: &str,
        vid_type: Option<&str>,
        comment: Option<&str>,
    ) -> Result<()>;

    /// Drop a graph space
    ///
    /// # Arguments
    /// * `name` - Space name
    async fn drop_space(&self, name: &str) -> Result<()>;

    /// Create a tag in a space
    ///
    /// # Arguments
    /// * `space` - Space name
    /// * `name` - Tag name
    /// * `properties` - Property definitions
    async fn create_tag(&self, space: &str, name: &str, properties: Vec<PropertyDef>)
        -> Result<()>;

    /// Create an edge type in a space
    ///
    /// # Arguments
    /// * `space` - Space name
    /// * `name` - Edge type name
    /// * `properties` - Property definitions
    async fn create_edge_type(
        &self,
        space: &str,
        name: &str,
        properties: Vec<PropertyDef>,
    ) -> Result<()>;

    // Batch operations

    /// Create a batch task
    ///
    /// # Arguments
    /// * `space_id` - Space ID
    /// * `batch_type` - Type of batch operation
    /// * `batch_size` - Batch size
    async fn create_batch(
        &self,
        space_id: u64,
        batch_type: BatchType,
        batch_size: usize,
    ) -> Result<String>;

    /// Add items to a batch
    ///
    /// # Arguments
    /// * `batch_id` - Batch ID
    /// * `items` - Items to add
    async fn add_batch_items(&self, batch_id: &str, items: Vec<BatchItem>) -> Result<usize>;

    /// Execute a batch task
    ///
    /// # Arguments
    /// * `batch_id` - Batch ID
    async fn execute_batch(&self, batch_id: &str) -> Result<BatchResult>;

    /// Get batch status
    ///
    /// # Arguments
    /// * `batch_id` - Batch ID
    async fn get_batch_status(&self, batch_id: &str) -> Result<BatchStatus>;

    /// Cancel a batch task
    ///
    /// # Arguments
    /// * `batch_id` - Batch ID
    async fn cancel_batch(&self, batch_id: &str) -> Result<()>;

    // Statistics APIs

    /// Get session statistics
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    async fn get_session_statistics(&self, session_id: i64) -> Result<SessionStatistics>;

    /// Get query statistics
    async fn get_query_statistics(&self) -> Result<QueryStatistics>;

    /// Get database statistics
    async fn get_database_statistics(&self) -> Result<DatabaseStatistics>;

    // Query validation

    /// Validate a query without executing it
    ///
    /// # Arguments
    /// * `query` - The query to validate
    async fn validate_query(&self, query: &str) -> Result<ValidationResult>;

    // Configuration management

    /// Get server configuration
    async fn get_config(&self) -> Result<ServerConfig>;

    /// Update server configuration
    ///
    /// # Arguments
    /// * `section` - Configuration section
    /// * `key` - Configuration key
    /// * `value` - New value
    async fn update_config(&self, section: &str, key: &str, value: serde_json::Value)
        -> Result<()>;

    // Vector operations

    /// Create a vector index
    ///
    /// # Arguments
    /// * `space` - Space name
    /// * `name` - Index name
    /// * `tag` - Tag to index
    /// * `field` - Field containing vector data
    /// * `dimension` - Vector dimension
    /// * `metric` - Distance metric (euclidean, cosine, etc.)
    async fn create_vector_index(
        &self,
        space: &str,
        name: &str,
        tag: &str,
        field: &str,
        dimension: usize,
        metric: &str,
    ) -> Result<()>;

    /// Drop a vector index
    ///
    /// # Arguments
    /// * `space` - Space name
    /// * `name` - Index name
    async fn drop_vector_index(&self, space: &str, name: &str) -> Result<()>;

    /// Search similar vectors
    ///
    /// # Arguments
    /// * `space` - Space name
    /// * `index_name` - Vector index name
    /// * `vector` - Query vector
    /// * `top_k` - Number of results to return
    async fn vector_search(
        &self,
        space: &str,
        index_name: &str,
        vector: Vec<f32>,
        top_k: usize,
    ) -> Result<VectorSearchResult>;
}

/// Query validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub estimated_cost: Option<u64>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub position: Option<usize>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub version: String,
    pub sections: Vec<ConfigSection>,
}

/// Configuration section
#[derive(Debug, Clone)]
pub struct ConfigSection {
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<ConfigItem>,
}

/// Configuration item
#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub mutable: bool,
}

/// Vector search result
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub total: usize,
    pub results: Vec<VectorMatch>,
}

/// Vector match
#[derive(Debug, Clone)]
pub struct VectorMatch {
    pub vid: serde_json::Value,
    pub score: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Factory for creating clients
pub struct ClientFactory;

impl ClientFactory {
    /// Create HTTP client based on configuration
    pub fn create(config: ClientConfig) -> Result<Box<dyn GraphDbClient>> {
        let client = super::http::HttpClient::with_config(config)?;
        Ok(Box::new(client))
    }

    /// Create HTTP client with default settings
    pub fn create_http(host: &str, port: u16) -> Result<Box<dyn GraphDbClient>> {
        let config = ClientConfig::new().with_host(host).with_port(port);
        Self::create(config)
    }
}
