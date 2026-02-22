use std::sync::Arc;
use parking_lot::Mutex;

use crate::api::session::ClientSession;
use crate::api::service::StatsManager;
use crate::query::QueryPipelineManager;
use crate::storage::StorageClient;
use crate::transaction::TransactionId;

#[derive(Debug)]
pub struct RequestContext {
    pub session_id: i64,
    pub statement: String,
    pub parameters: std::collections::HashMap<String, String>,
    pub client_session: Option<Arc<ClientSession>>,
    pub transaction_id: Option<TransactionId>,
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

pub struct QueryEngine<S: StorageClient + 'static> {
    storage: Arc<Mutex<S>>,
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryEngine<S> {
    pub fn new(storage: Arc<S>) -> Self {
        let storage_mutex = Arc::new(Mutex::new((*storage).clone()));
        let stats_manager = Arc::new(StatsManager::new());
        Self {
            storage: Arc::clone(&storage_mutex),
            pipeline_manager: QueryPipelineManager::new(storage_mutex, stats_manager),
        }
    }

    pub async fn execute(&mut self, rctx: RequestContext) -> ExecutionResponse {
        let start_time = std::time::Instant::now();

        // 从客户端会话中提取空间信息
        let space_info = rctx.client_session.as_ref().and_then(|session| {
            session.space().map(|s| {
                crate::core::types::SpaceInfo {
                    space_name: s.name.clone(),
                    space_id: s.id as u64,
                    vid_type: crate::core::types::DataType::String,
                    tags: Vec::new(),
                    edge_types: Vec::new(),
                    version: crate::core::types::MetadataVersion::default(),
                    comment: None,
                }
            })
        });

        match self.pipeline_manager.execute_query_with_space(&rctx.statement, space_info).await {
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

    pub fn get_storage(&self) -> Arc<Mutex<S>> {
        Arc::clone(&self.storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::session::client_session::{ClientSession, Session};
    use crate::storage::test_mock::MockStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_query_engine_creation() {
        let storage = Arc::new(MockStorage::new().expect("Failed to create Mock storage"));
        let _query_engine = QueryEngine::new(storage);

        assert!(true);
    }

    #[tokio::test]
    async fn test_query_engine_execute() {
        let storage = Arc::new(MockStorage::new().expect("Failed to create Mock storage"));
        let mut query_engine = QueryEngine::new(storage);

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
            transaction_id: None,
        };

        let _response = query_engine.execute(request_context).await;
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
