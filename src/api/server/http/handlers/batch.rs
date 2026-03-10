//! 批量操作 HTTP 处理器

use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde_json;

use crate::api::server::batch::{
    AddBatchItemsRequest, AddBatchItemsResponse, BatchManager, BatchProgress, BatchStatus,
    BatchStatusResponse, CreateBatchRequest, CreateBatchResponse, ExecuteBatchResponse,
};
use crate::api::server::http::{error::HttpError, state::AppState};
use crate::storage::StorageClient;

/// 创建批量任务
pub async fn create<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateBatchRequest>,
) -> Result<JsonResponse<CreateBatchResponse>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    match batch_manager.create_task(request.space_id, request.batch_type, request.batch_size) {
        Ok(task) => Ok(JsonResponse(CreateBatchResponse {
            batch_id: task.id,
            status: task.status,
            created_at: task.created_at.to_rfc3339(),
        })),
        Err(e) => Err(HttpError::InternalError(format!("创建批量任务失败: {}", e))),
    }
}

/// 获取批量任务状态
pub async fn status<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(batch_id): Path<String>,
) -> Result<JsonResponse<BatchStatusResponse>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    match batch_manager.get_task(&batch_id) {
        Some(task) => Ok(JsonResponse(BatchStatusResponse {
            batch_id: task.id,
            status: task.status,
            progress: task.progress,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        })),
        None => Err(HttpError::NotFound(format!("批量任务不存在: {}", batch_id))),
    }
}

/// 添加批量项
pub async fn add_items<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(batch_id): Path<String>,
    Json(request): Json<AddBatchItemsRequest>,
) -> Result<JsonResponse<AddBatchItemsResponse>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    match batch_manager.add_items(&batch_id, request.items) {
        Ok(accepted) => {
            let task = batch_manager
                .get_task(&batch_id)
                .ok_or_else(|| HttpError::NotFound(format!("批量任务不存在: {}", batch_id)))?;

            Ok(JsonResponse(AddBatchItemsResponse {
                accepted,
                buffered: accepted,
                total_buffered: task.progress.buffered,
            }))
        }
        Err(e) => Err(HttpError::BadRequest(format!("添加批量项失败: {}", e))),
    }
}

/// 执行批量任务
pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(batch_id): Path<String>,
) -> Result<JsonResponse<ExecuteBatchResponse>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    // 获取任务信息以获取 space_id
    let task = batch_manager
        .get_task(&batch_id)
        .ok_or_else(|| HttpError::NotFound(format!("批量任务不存在: {}", batch_id)))?;

    // 获取 space_name（这里简化处理，实际应该从 space_id 查询）
    let space_name = format!("space_{}", task.space_id);

    match batch_manager.execute_task(&batch_id, &space_name).await {
        Ok(result) => {
            let task = batch_manager
                .get_task(&batch_id)
                .ok_or_else(|| HttpError::NotFound(format!("批量任务不存在: {}", batch_id)))?;

            Ok(JsonResponse(ExecuteBatchResponse {
                batch_id: batch_id.clone(),
                status: task.status,
                result,
                completed_at: Some(task.updated_at.to_rfc3339()),
            }))
        }
        Err(e) => Err(HttpError::InternalError(format!("执行批量任务失败: {}", e))),
    }
}

/// 取消批量任务
pub async fn cancel<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(batch_id): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    match batch_manager.cancel_task(&batch_id) {
        Ok(()) => Ok(JsonResponse(serde_json::json!({
            "message": "批量任务已取消",
            "batch_id": batch_id,
        }))),
        Err(e) => Err(HttpError::BadRequest(format!("取消批量任务失败: {}", e))),
    }
}

/// 删除批量任务
pub async fn delete<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(batch_id): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let batch_manager = state.server.get_batch_manager();

    match batch_manager.remove_task(&batch_id) {
        Ok(()) => Ok(JsonResponse(serde_json::json!({
            "message": "批量任务已删除",
            "batch_id": batch_id,
        }))),
        Err(e) => Err(HttpError::NotFound(format!("删除批量任务失败: {}", e))),
    }
}
