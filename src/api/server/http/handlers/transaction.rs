use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::storage::StorageClient;
use crate::transaction::{DurabilityLevel, TransactionOptions};

#[derive(Debug, Deserialize)]
pub struct BeginTransactionRequest {
    pub session_id: i64,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transaction_id: u64,
    pub status: String,
}

/// 开始事务
pub async fn begin<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<BeginTransactionRequest>,
) -> Result<JsonResponse<TransactionResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        let txn_manager = state.server.get_txn_manager();

        let options = TransactionOptions {
            read_only: request.read_only,
            timeout: request.timeout_seconds.map(std::time::Duration::from_secs),
            durability: DurabilityLevel::Immediate,
            two_phase_commit: false,
        };

        match txn_manager.begin_transaction(options) {
            Ok(txn_id) => Ok::<_, HttpError>(TransactionResponse {
                transaction_id: txn_id,
                status: "Active".to_string(),
            }),
            Err(e) => Err(HttpError::InternalError(format!("开始事务失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

#[derive(Debug, Deserialize)]
pub struct TransactionActionRequest {
    pub session_id: i64,
}

/// 提交事务
pub async fn commit<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(txn_id): Path<u64>,
    Json(_request): Json<TransactionActionRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let txn_manager = state.server.get_txn_manager();

        match txn_manager.commit_transaction(txn_id) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "事务提交成功",
                "transaction_id": txn_id,
            })),
            Err(e) => Err(HttpError::InternalError(format!("提交事务失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 回滚事务
pub async fn rollback<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(txn_id): Path<u64>,
    Json(_request): Json<TransactionActionRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let txn_manager = state.server.get_txn_manager();

        match txn_manager.abort_transaction(txn_id) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "事务回滚成功",
                "transaction_id": txn_id,
            })),
            Err(e) => Err(HttpError::InternalError(format!("回滚事务失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}
