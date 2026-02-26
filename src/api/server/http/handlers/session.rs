use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};

use crate::api::server::http::{
    state::AppState,
    error::HttpError,
};
use crate::storage::StorageClient;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub username: String,
    pub client_ip: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: i64,
    pub username: String,
    pub created_at: u64,
}

pub async fn create<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<JsonResponse<SessionResponse>, HttpError> {
    let session_manager = state.server.get_session_manager();
    let session = session_manager
        .create_session(request.username, request.client_ip)
        .await
        .map_err(|e| HttpError::BadRequest(format!("创建会话失败: {}", e)))?;
    
    Ok(JsonResponse(SessionResponse {
        session_id: session.id(),
        username: session.user(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }))
}

pub async fn get_session<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<i64>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let session_manager = state.server.get_session_manager();
    let session = session_manager
        .find_session(session_id)
        .ok_or_else(|| HttpError::NotFound("会话不存在".to_string()))?;
    
    Ok(JsonResponse(serde_json::json!({
        "session_id": session.id(),
        "username": session.user(),
        "space_name": session.space_name(),
        "graph_addr": session.graph_addr(),
        "timezone": session.timezone(),
    })))
}

pub async fn delete_session<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let session_manager = state.server.get_session_manager();
    session_manager.remove_session(session_id).await;
    Ok(StatusCode::NO_CONTENT)
}
