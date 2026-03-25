use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::api::server::http::{error::HttpError, state::AppState};
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
        // The authenticate method of the GraphService is required.
        // The current architecture needs to be adjusted to return to the simulation results for the time being
        Ok::<_, HttpError>(LoginResponse {
            session_id: 12345,
            username: request.username,
            expires_at: None,
        })
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

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
