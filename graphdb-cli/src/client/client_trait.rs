//! GraphDbClient trait - Abstract interface for database connections
//!
//! This trait provides a unified interface for both HTTP remote connections
//! and embedded local database access.

use async_trait::async_trait;

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

/// Connection mode - HTTP remote or Embedded local
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    /// Connect to remote server via HTTP
    Http,
    /// Use embedded database (direct file access)
    Embedded,
}

impl std::str::FromStr for ConnectionMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(ConnectionMode::Http),
            "embedded" => Ok(ConnectionMode::Embedded),
            _ => Err(format!("Unknown connection mode: {}", s)),
        }
    }
}

impl std::fmt::Display for ConnectionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionMode::Http => write!(f, "http"),
            ConnectionMode::Embedded => write!(f, "embedded"),
        }
    }
}

/// Configuration for client connections
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub mode: ConnectionMode,
    pub host: String,
    pub port: u16,
    pub database_path: Option<String>,
    pub username: String,
    pub password: String,
    pub timeout_seconds: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            mode: ConnectionMode::Http,
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_path: None,
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

    pub fn with_mode(mut self, mode: ConnectionMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_database_path(mut self, path: impl Into<String>) -> Self {
        self.database_path = Some(path.into());
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

/// Core trait for GraphDB client operations
///
/// Implementations:
/// - `HttpClient`: Connects to remote GraphDB server via HTTP API
/// - `EmbeddedClient`: Direct access to local database file
#[async_trait]
pub trait GraphDbClient: Send + Sync {
    /// Get the connection mode
    fn connection_mode(&self) -> ConnectionMode;

    /// Check if client is currently connected
    fn is_connected(&self) -> bool;

    /// Connect to the database
    ///
    /// For HTTP mode: authenticates with server
    /// For Embedded mode: opens database file
    async fn connect(&mut self) -> Result<SessionInfo>;

    /// Disconnect from the database
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

    /// Get base URL or database path
    fn connection_string(&self) -> String;
}

/// Factory for creating clients
pub struct ClientFactory;

impl ClientFactory {
    /// Create a client based on configuration
    pub fn create(config: ClientConfig) -> Result<Box<dyn GraphDbClient>> {
        match config.mode {
            ConnectionMode::Http => {
                let client = super::http::HttpClient::with_config(config)?;
                Ok(Box::new(client))
            }
            ConnectionMode::Embedded => {
                let client = super::embedded::EmbeddedClient::with_config(config)?;
                Ok(Box::new(client))
            }
        }
    }

    /// Create HTTP client with default settings
    pub fn create_http(host: &str, port: u16) -> Result<Box<dyn GraphDbClient>> {
        let config = ClientConfig::new()
            .with_mode(ConnectionMode::Http)
            .with_host(host)
            .with_port(port);
        Self::create(config)
    }

    /// Create embedded client with database path
    pub fn create_embedded(db_path: &str) -> Result<Box<dyn GraphDbClient>> {
        let config = ClientConfig::new()
            .with_mode(ConnectionMode::Embedded)
            .with_database_path(db_path);
        Self::create(config)
    }
}
