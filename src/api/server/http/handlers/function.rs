//! 自定义函数 HTTP 处理器

use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use serde_json;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::storage::StorageClient;

/// 注册自定义函数
pub async fn register<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Json(_request): Json<RegisterFunctionRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // TODO: 实现实际的函数注册
    Ok(JsonResponse(serde_json::json!({
        "function_id": "uuid",
        "name": _request.name,
        "status": "registered",
    })))
}

/// 列出所有函数
pub async fn list<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // TODO: 实现实际的函数列表
    Ok(JsonResponse(serde_json::json!({
        "functions": []
    })))
}

/// 获取函数详情
pub async fn info<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // TODO: 实现实际的函数详情获取
    Ok(JsonResponse(serde_json::json!({
        "name": name,
        "type": "scalar",
        "parameters": [],
        "registered_at": "2024-01-01T00:00:00Z",
    })))
}

/// 注销函数
pub async fn unregister<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // TODO: 实现实际的函数注销
    Ok(JsonResponse(serde_json::json!({
        "message": "函数已注销",
        "name": name,
    })))
}

/// 注册函数请求
#[derive(Debug, Deserialize)]
pub struct RegisterFunctionRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub function_type: String,
    pub parameters: Vec<String>,
    #[serde(rename = "return_type")]
    pub return_type: String,
    pub implementation: serde_json::Value,
}
