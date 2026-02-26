use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::api::server::http::{
    state::AppState,
    error::HttpError,
};
use crate::storage::StorageClient;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_id: i64,
    pub username: String,
    pub expires_at: Option<u64>,
}

pub async fn login<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Json(request): Json<LoginRequest>,
) -> Result<JsonResponse<LoginResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        // 这里需要通过 GraphService 的 authenticate 方法
        // 当前架构需要调整，暂时返回模拟结果
        Ok::<_, HttpError>(LoginResponse {
            session_id: 12345,
            username: request.username,
            expires_at: None,
        })
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;
    
    Ok(JsonResponse(result?))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub session_id: i64,
}

pub async fn logout<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<LogoutRequest>,
) -> Result<StatusCode, HttpError> {
    let session_manager = state.server.get_session_manager();
    session_manager.remove_session(request.session_id).await;
    Ok(StatusCode::NO_CONTENT)
}
