//! Data Browser Handlers
//!
//! Provides data browsing APIs:
//! - Browse vertices by tag
//! - Browse edges by edge type
//! - Data filtering and pagination

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::api::server::web::{
    error::WebResult,
    models::{ApiResponse, PaginatedResponse, PaginationParams},
    WebState,
};
use crate::storage::StorageClient;

/// Create data browser router
pub fn create_router<S: StorageClient + 'static>(_web_state: WebState<S>) -> Router {
    Router::new()
        .route("/spaces/:name/tags/:tag_name/vertices", get(list_vertices_by_tag::<S>))
        .route(
            "/spaces/:name/edge-types/:edge_name/edges",
            get(list_edges_by_type::<S>),
        )
}

/// Filter parameters for data browsing
#[derive(Debug, Deserialize)]
pub struct DataFilterParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    /// Property filter (e.g., "age>18")
    pub filter: Option<String>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort order
    pub sort_order: Option<String>,
}

/// List vertices by tag
async fn list_vertices_by_tag<S: StorageClient>(
    Path((_space_name, _tag_name)): Path<(String, String)>,
    Query(params): Query<DataFilterParams>,
) -> WebResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    // TODO: Implement actual vertex listing with core API
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        vec![],
        0,
        params.pagination.limit,
        params.pagination.offset,
    ))))
}

/// List edges by type
async fn list_edges_by_type<S: StorageClient>(
    Path((_space_name, _edge_name)): Path<(String, String)>,
    Query(params): Query<DataFilterParams>,
) -> WebResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    // TODO: Implement actual edge listing with core API
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        vec![],
        0,
        params.pagination.limit,
        params.pagination.offset,
    ))))
}
