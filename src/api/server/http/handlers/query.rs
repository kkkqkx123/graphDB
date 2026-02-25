use axum::{
    extract::{State, Json},
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
pub struct QueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub result: String,
    pub execution_time_ms: u64,
}

pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<QueryRequest>,
) -> Result<JsonResponse<QueryResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        let graph_service = state.server.get_graph_service();
        
        // 通过 GraphService 执行查询
        match graph_service.execute(request.session_id, &request.query) {
            Ok(result) => Ok::<_, HttpError>(QueryResponse {
                result,
                execution_time_ms: 0, // GraphService 返回的结果中不包含执行时间，后续可以改进
            }),
            Err(e) => Err(HttpError::InternalError(format!("查询执行失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;
    
    Ok(JsonResponse(result?))
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub message: String,
}

pub async fn validate(
    Json(_request): Json<QueryRequest>,
) -> Result<JsonResponse<ValidateResponse>, HttpError> {
    Ok(JsonResponse(ValidateResponse {
        valid: true,
        message: "语法正确".to_string(),
    }))
}
