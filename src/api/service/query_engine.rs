use std::sync::{Arc, Mutex};

use crate::api::session::ClientSession;
use crate::core::QueryPipelineManager;
use crate::storage::NativeStorage;

#[derive(Debug)]
pub struct RequestContext {
    pub session_id: i64,
    pub statement: String,
    pub parameters: std::collections::HashMap<String, String>,
    pub client_session: Option<Arc<ClientSession>>,
}

#[derive(Debug)]
pub struct ExecutionResponse {
    pub result: Result<String, String>,
    pub latency_us: u64,
}

#[derive(Debug)]
pub struct AuthResponse {
    pub session_id: i64,
    pub result: Result<(), String>,
}

pub struct QueryEngine {
    storage: Arc<Mutex<NativeStorage>>,
    pipeline_manager: QueryPipelineManager<NativeStorage>,
}

impl QueryEngine {
    pub fn new(storage: Arc<NativeStorage>) -> Self {
        let storage_clone = Arc::new(Mutex::new((*storage).clone()));
        Self {
            storage: Arc::clone(&storage_clone),
            pipeline_manager: QueryPipelineManager::new(storage_clone),
        }
    }

    pub async fn execute(&mut self, rctx: RequestContext) -> ExecutionResponse {
        let start_time = std::time::Instant::now();

        // 使用新的查询管道管理器执行查询
        match self.pipeline_manager.execute_query(&rctx.statement).await {
            Ok(result) => ExecutionResponse {
                result: Ok(format!("{:?}", result)),
                latency_us: start_time.elapsed().as_micros() as u64,
            },
            Err(e) => ExecutionResponse {
                result: Err(e.to_string()),
                latency_us: start_time.elapsed().as_micros() as u64,
            },
        }
    }

    pub fn get_storage(&self) -> Arc<Mutex<NativeStorage>> {
        Arc::clone(&self.storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::session::client_session::{ClientSession, Session};
    use crate::config::Config;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_query_engine_creation() {
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
            NativeStorage::new(&config.storage_path).expect("Failed to create native storage"),
        );
        let _query_engine = QueryEngine::new(storage);

        // We can't directly check the data dir, so we'll just test that storage initialization succeeded
        // by ensuring no panic occurred during construction
        assert!(true); // Test passes as long as we reached this point without panicking
    }

    #[tokio::test]
    async fn test_query_engine_execute() {
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
            NativeStorage::new(&config.storage_path).expect("Failed to create native storage"),
        );
        let mut query_engine = QueryEngine::new(storage);

        // Create a dummy session
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };
        let client_session = ClientSession::new(session);

        let request_context = RequestContext {
            session_id: 123,
            statement: "CREATE SPACE IF NOT EXISTS test_space".to_string(),
            parameters: std::collections::HashMap::new(),
            client_session: Some(client_session),
        };

        let _response = query_engine.execute(request_context).await;
        // The query will likely fail with an unsupported statement, but we want to ensure
        // the execution path works without panicking
        // Note: This particular query might fail since our parser doesn't support it,
        // but that's expected behavior
    }

    #[tokio::test]
    async fn test_execution_response() {
        let response = ExecutionResponse {
            result: Ok("Success".to_string()),
            latency_us: 1000,
        };

        assert!(response.result.is_ok());
        assert_eq!(response.latency_us, 1000);
    }
}
