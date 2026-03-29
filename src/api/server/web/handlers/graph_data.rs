//! Graph Data Handlers
//!
//! Provides graph data query APIs for visualization:
//! - Vertex details
//! - Edge details
//! - Neighbor queries

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::api::server::web::{
    error::{WebError, WebResult},
    models::ApiResponse,
    WebState,
};
use crate::storage::StorageClient;

/// Create graph data router
pub fn create_router<S: StorageClient + 'static>(_web_state: WebState<S>) -> Router {
    Router::new()
        .route("/vertices/:vid", get(get_vertex::<S>))
        .route("/edges", get(get_edge::<S>))
        .route("/vertices/:vid/neighbors", get(get_neighbors::<S>))
}

/// Get vertex details
#[derive(Debug, Deserialize)]
pub struct GetVertexParams {
    pub space: String,
}

async fn get_vertex<S: StorageClient>(
    Path(vid): Path<String>,
    Query(params): Query<GetVertexParams>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual vertex retrieval with core API
    Err(WebError::NotFound(format!(
        "Vertex '{}' not found in space '{}'",
        vid, params.space
    )))
}

/// Get edge details
#[derive(Debug, Deserialize)]
pub struct GetEdgeParams {
    pub space: String,
    pub src: String,
    pub dst: String,
    pub edge_type: String,
    #[serde(default)]
    pub rank: i64,
}

async fn get_edge<S: StorageClient>(
    Query(params): Query<GetEdgeParams>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual edge retrieval with core API
    Err(WebError::NotFound(format!(
        "Edge from '{}' to '{}' with type '{}' not found in space '{}'",
        params.src, params.dst, params.edge_type, params.space
    )))
}

/// Get neighbors of a vertex
#[derive(Debug, Deserialize)]
pub struct GetNeighborsParams {
    pub space: String,
    /// Direction: OUT, IN, or BOTH
    #[serde(default = "default_direction")]
    pub direction: String,
    /// Edge type filter
    pub edge_type: Option<String>,
}

fn default_direction() -> String {
    "BOTH".to_string()
}

async fn get_neighbors<S: StorageClient>(
    Path(vid): Path<String>,
    Query(params): Query<GetNeighborsParams>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual neighbor query with core API
    Ok(Json(ApiResponse::success(serde_json::json!({
        "vid": vid,
        "space": params.space,
        "direction": params.direction,
        "edge_type": params.edge_type,
        "neighbors": [],
        "note": "Neighbor query to be implemented"
    }))))
}
