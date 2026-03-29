//! Metadata Handlers (Query History & Favorites)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::api::server::web::{
    error::WebResult,
    models::{
        metadata::{
            AddFavoriteRequest, AddHistoryRequest, FavoriteListResponse, HistoryListResponse,
            UpdateFavoriteRequest,
        },
        ApiResponse, PaginationParams,
    },
    services::metadata_service::MetadataService,
    WebState,
};
use crate::storage::StorageClient;

/// Create metadata router
pub fn create_router<S: StorageClient + 'static>(web_state: WebState<S>) -> Router {
    Router::new()
        .route("/history", get(list_history::<S>).post(add_history::<S>))
        .route("/history/:id", delete(delete_history::<S>))
        .route("/history/clear", delete(clear_history::<S>))
        .route("/favorites", get(list_favorites::<S>).post(add_favorite::<S>))
        .route("/favorites/:id", get(get_favorite::<S>).put(update_favorite::<S>).delete(delete_favorite::<S>))
        .route("/favorites/clear", delete(clear_favorites::<S>))
        .with_state(web_state)
}

/// Add a query history item
async fn add_history<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Json(request): Json<AddHistoryRequest>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    // Get session ID from app state (this is a simplified version)
    // In real implementation, extract from auth middleware
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let item = service.add_history(&session_id, request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(serde_json::json!({
            "id": item.id,
            "query": item.query,
            "executed_at": item.executed_at,
            "execution_time_ms": item.execution_time_ms,
            "rows_returned": item.rows_returned,
            "success": item.success,
        }))),
    ))
}

/// List query history
async fn list_history<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Query(params): Query<PaginationParams>,
) -> WebResult<Json<ApiResponse<HistoryListResponse>>> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let (items, total) = service.get_history(&session_id, params.limit, params.offset).await?;

    Ok(Json(ApiResponse::success(HistoryListResponse { items, total })))
}

/// Delete a history item
async fn delete_history<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Path(id): Path<String>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    service.delete_history(&id, &session_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({"deleted": true}))),
    ))
}

/// Clear all history
async fn clear_history<S: StorageClient>(
    State(web_state): State<WebState<S>>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    service.clear_history(&session_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({"cleared": true}))),
    ))
}

/// Add a favorite
async fn add_favorite<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Json(request): Json<AddFavoriteRequest>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let item = service.add_favorite(&session_id, request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(serde_json::json!({
            "id": item.id,
            "name": item.name,
            "query": item.query,
            "description": item.description,
            "created_at": item.created_at,
        }))),
    ))
}

/// List all favorites
async fn list_favorites<S: StorageClient>(
    State(web_state): State<WebState<S>>,
) -> WebResult<Json<ApiResponse<FavoriteListResponse>>> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let items = service.get_favorites(&session_id).await?;

    Ok(Json(ApiResponse::success(FavoriteListResponse { items })))
}

/// Get a favorite by ID
async fn get_favorite<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Path(id): Path<String>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let item = service.get_favorite(&id, &session_id).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "id": item.id,
        "name": item.name,
        "query": item.query,
        "description": item.description,
        "created_at": item.created_at,
    }))))
}

/// Update a favorite
async fn update_favorite<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateFavoriteRequest>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    let item = service.update_favorite(&id, &session_id, request).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "id": item.id,
        "name": item.name,
        "query": item.query,
        "description": item.description,
        "created_at": item.created_at,
    }))))
}

/// Delete a favorite
async fn delete_favorite<S: StorageClient>(
    State(web_state): State<WebState<S>>,
    Path(id): Path<String>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    service.delete_favorite(&id, &session_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({"deleted": true}))),
    ))
}

/// Clear all favorites
async fn clear_favorites<S: StorageClient>(
    State(web_state): State<WebState<S>>,
) -> WebResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let session_id = "default_session".to_string();

    let service = MetadataService::new(web_state.metadata_storage.clone());
    service.delete_all_favorites(&session_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({"cleared": true}))),
    ))
}
