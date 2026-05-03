use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::api::core::TransactionHandle;
use crate::api::server::http::{error::HttpError, state::AppState};
use crate::storage::StorageClient;
use crate::transaction::{DurabilityLevel, IsolationLevel, TransactionOptions};

#[derive(Debug, Deserialize)]
pub struct BeginTransactionRequest {
    pub session_id: i64,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub query_timeout_seconds: Option<u64>,
    #[serde(default)]
    pub statement_timeout_seconds: Option<u64>,
    #[serde(default)]
    pub idle_timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transaction_id: u64,
    pub status: String,
}

/// Start a transaction
pub async fn begin<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<BeginTransactionRequest>,
) -> Result<JsonResponse<TransactionResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        let txn_api = state.server.get_txn_api();

        let options = TransactionOptions {
            read_only: request.read_only,
            timeout: request.timeout_seconds.map(std::time::Duration::from_secs),
            durability: DurabilityLevel::Immediate,
            isolation_level: IsolationLevel::default(),
            query_timeout: request
                .query_timeout_seconds
                .map(std::time::Duration::from_secs),
            statement_timeout: request
                .statement_timeout_seconds
                .map(std::time::Duration::from_secs),
            idle_timeout: request
                .idle_timeout_seconds
                .map(std::time::Duration::from_secs),
            two_phase_commit: false,
        };

        match txn_api.begin(options) {
            Ok(handle) => Ok::<_, HttpError>(TransactionResponse {
                transaction_id: handle.0,
                status: "Active".to_string(),
            }),
            Err(e) => Err(HttpError::InternalError(format!(
                "Failed to begin transaction: {}",
                e
            ))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

#[derive(Debug, Deserialize)]
pub struct TransactionActionRequest {
    pub session_id: i64,
}

/// Submit the transaction
pub async fn commit<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(txn_id): Path<u64>,
    Json(_request): Json<TransactionActionRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let txn_api = state.server.get_txn_api();
    let handle = TransactionHandle(txn_id);

    match txn_api.commit(handle) {
        Ok(()) => Ok(JsonResponse(serde_json::json!({
            "message": "Transaction committed successfully",
            "transaction_id": txn_id,
        }))),
        Err(e) => Err(HttpError::InternalError(format!(
            "Failed to commit transaction: {}",
            e
        ))),
    }
}

/// Roll back a transaction
pub async fn rollback<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(txn_id): Path<u64>,
    Json(_request): Json<TransactionActionRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let txn_api = state.server.get_txn_api();
    let handle = TransactionHandle(txn_id);

    match txn_api.rollback(handle) {
        Ok(()) => Ok(JsonResponse(serde_json::json!({
            "message": "Transaction rolled back successfully",
            "transaction_id": txn_id,
        }))),
        Err(e) => Err(HttpError::InternalError(format!(
            "Failed to rollback transaction: {}",
            e
        ))),
    }
}
