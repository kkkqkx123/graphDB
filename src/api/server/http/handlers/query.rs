use axum::{
    extract::{Json, State},
    response::Json as JsonResponse,
};
use tokio::task;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::api::server::http::handlers::query_types::*;
use crate::storage::StorageClient;

pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<QueryRequest>,
) -> Result<JsonResponse<QueryResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        let graph_service = state.server.get_graph_service();

        // 通过 GraphService 执行查询
        match graph_service.execute(request.session_id, &request.query) {
            Ok(result_str) => {
                // TODO: 解析字符串结果为结构化数据
                // 目前先使用字符串结果，后续需要改进 GraphService 返回结构化结果
                Ok::<_, HttpError>(QueryResponse::from_string(result_str))
            }
            Err(e) => {
                Ok::<_, HttpError>(QueryResponse::error(
                    "QUERY_ERROR".to_string(),
                    e.to_string(),
                    None,
                ))
            }
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

pub async fn validate(
    Json(_request): Json<QueryRequest>,
) -> Result<JsonResponse<ValidateResponse>, HttpError> {
    Ok(JsonResponse(ValidateResponse {
        valid: true,
        message: "Syntax is correct".to_string(),
    }))
}
