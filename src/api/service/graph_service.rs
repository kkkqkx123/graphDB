use crate::api::service::QueryEngine;
use crate::api::session::{ClientSession, GraphSessionManager};
use crate::config::Config;
use crate::storage::RocksDBStorage;
use std::sync::{Arc, Mutex};

pub struct GraphService {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine>>,

    config: Config,
}

impl GraphService {
    pub fn new(config: Config, storage: Arc<RocksDBStorage>) -> Arc<Self> {
        let session_manager = GraphSessionManager::new(format!("{}:{}", config.host, config.port));
        let query_engine = Arc::new(Mutex::new(QueryEngine::new(storage)));

        Arc::new(Self {
            session_manager,
            query_engine,
            config,
        })
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Arc<ClientSession>, String> {
        // In a real implementation, you would verify the username and password
        // For now, we'll just create a session if the credentials are non-empty
        if username.is_empty() || password.is_empty() {
            return Err("Invalid username or password".to_string());
        }

        self.session_manager
            .create_session(username.to_string(), "127.0.0.1".to_string())
    }

    pub async fn execute(&self, session_id: i64, stmt: &str) -> Result<String, String> {
        // Find the session
        let session = self.session_manager.find_session(session_id);
        if session.is_none() {
            return Err("Invalid session ID".to_string());
        }

        // Create request context
        let request_context = crate::api::service::query_engine::RequestContext {
            session_id,
            statement: stmt.to_string(),
            parameters: std::collections::HashMap::new(),
            client_session: session,
        };

        // Execute the query
        let mut query_engine = self
            .query_engine
            .lock()
            .expect("Query engine lock was poisoned");
        let response = query_engine.execute(request_context).await;

        match response.result {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    pub fn get_session_manager(&self) -> &GraphSessionManager {
        &self.session_manager
    }

    pub fn get_query_engine(&self) -> &Mutex<QueryEngine> {
        &self.query_engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::storage::RocksDBStorage;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_graph_service_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: temp_dir
                .path()
                .to_str()
                .expect("Failed to convert temp path to string")
                .to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(
            RocksDBStorage::new(&config.storage_path).expect("Failed to create RocksDB storage"),
        );
        let graph_service = GraphService::new(config, storage);

        assert_eq!(graph_service.config.host, "127.0.0.1");
        assert_eq!(graph_service.config.port, 9669);
    }

    #[tokio::test]
    async fn test_authentication() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: temp_dir
                .path()
                .to_str()
                .expect("Failed to convert temp path to string")
                .to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(
            RocksDBStorage::new(&config.storage_path).expect("Failed to create RocksDB storage"),
        );
        let graph_service = GraphService::new(config, storage);

        // Test valid credentials
        let session = graph_service.authenticate("testuser", "password").await;
        assert!(session.is_ok());

        // Test invalid credentials
        let session = graph_service.authenticate("", "password").await;
        assert!(session.is_err());

        let session = graph_service.authenticate("testuser", "").await;
        assert!(session.is_err());
    }

    #[tokio::test]
    async fn test_execute_query() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: temp_dir
                .path()
                .to_str()
                .expect("Failed to convert temp path to string")
                .to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(
            RocksDBStorage::new(&config.storage_path).expect("Failed to create RocksDB storage"),
        );
        let graph_service = GraphService::new(config, storage);

        // First authenticate to get a session
        let session = graph_service
            .authenticate("testuser", "password")
            .await
            .expect("Failed to authenticate session");
        let session_id = session.id();

        // Try to execute a query (this will likely fail due to unsupported query, but should not panic)
        let _result = graph_service.execute(session_id, "SHOW SPACES").await;
        // The result could be either success or failure depending on whether the query is supported,
        // but we're checking that it doesn't panic or fail in an unexpected way
    }

    #[tokio::test]
    async fn test_invalid_session_execute() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 9669,
            storage_path: temp_dir
                .path()
                .to_str()
                .expect("Failed to convert temp path to string")
                .to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        };

        let storage = Arc::new(
            RocksDBStorage::new(&config.storage_path).expect("Failed to create RocksDB storage"),
        );
        let graph_service = GraphService::new(config, storage);

        // Try to execute a query with an invalid session
        let result = graph_service.execute(999999, "SHOW SPACES").await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Expected error for invalid session"),
            "Invalid session ID"
        );
    }
}
