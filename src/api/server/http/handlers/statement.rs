//! 预编译语句 HTTP 处理器

use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde_json;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::api::server::statement::types::*;
use crate::storage::StorageClient;

/// 创建预编译语句
pub async fn create<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateStatementRequest>,
) -> Result<JsonResponse<CreateStatementResponse>, HttpError> {
    let statement_manager = state.server.get_statement_manager();

    match statement_manager.create_statement(request.query, request.space_id) {
        Ok(info) => Ok(JsonResponse(CreateStatementResponse {
            statement_id: info.id.clone(),
            parameters: info.parameters.clone(),
            created_at: info.created_at.to_rfc3339(),
        })),
        Err(e) => Err(HttpError::InternalError(format!(
            "创建预编译语句失败: {}",
            e
        ))),
    }
}

/// 获取预编译语句信息
pub async fn info<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(statement_id): Path<String>,
) -> Result<JsonResponse<StatementInfoResponse>, HttpError> {
    let statement_manager = state.server.get_statement_manager();

    match statement_manager.get_statement(&statement_id) {
        Some(info) => Ok(JsonResponse(info.to_response())),
        None => Err(HttpError::NotFound(format!(
            "预编译语句不存在: {}",
            statement_id
        ))),
    }
}

/// 执行预编译语句
pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(statement_id): Path<String>,
    Json(request): Json<ExecuteStatementRequest>,
) -> Result<JsonResponse<ExecuteStatementResponse>, HttpError> {
    let statement_manager = state.server.get_statement_manager();

    match statement_manager.execute_statement(&statement_id, &request.parameters) {
        Ok(response) => Ok(JsonResponse(response)),
        Err(e) => Err(HttpError::InternalError(format!(
            "执行预编译语句失败: {}",
            e
        ))),
    }
}

/// 批量执行预编译语句
pub async fn batch_execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(statement_id): Path<String>,
    Json(request): Json<BatchExecuteStatementRequest>,
) -> Result<JsonResponse<BatchExecuteStatementResponse>, HttpError> {
    let statement_manager = state.server.get_statement_manager();

    match statement_manager.batch_execute_statement(&statement_id, request.batch_parameters) {
        Ok(response) => Ok(JsonResponse(response)),
        Err(e) => Err(HttpError::InternalError(format!(
            "批量执行预编译语句失败: {}",
            e
        ))),
    }
}

/// 删除预编译语句
pub async fn drop<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(statement_id): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let statement_manager = state.server.get_statement_manager();

    match statement_manager.remove_statement(&statement_id) {
        Ok(()) => Ok(JsonResponse(serde_json::json!({
            "message": "预编译语句已删除",
            "statement_id": statement_id,
        }))),
        Err(e) => Err(HttpError::NotFound(format!("删除预编译语句失败: {}", e))),
    }
}
