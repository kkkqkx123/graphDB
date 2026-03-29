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
use tokio::task;

use crate::api::server::web::{
    error::{WebError, WebResult},
    models::{ApiResponse, PaginatedResponse, PaginationParams},
    WebState,
};
use crate::storage::StorageClient;

/// Create data browser routes (without state)
pub fn create_routes<S: StorageClient + Clone + Send + Sync + 'static>() -> Router<WebState<S>> {
    Router::new()
        .route(
            "/spaces/{name}/tags/{tag_name}/vertices",
            get(list_vertices_by_tag),
        )
        .route(
            "/spaces/{name}/edge-types/{edge_name}/edges",
            get(list_edges_by_type),
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
async fn list_vertices_by_tag<S: StorageClient + Clone + Send + Sync + 'static>(
    State(web_state): State<WebState<S>>,
    Path((space_name, tag_name)): Path<(String, String)>,
    Query(params): Query<DataFilterParams>,
) -> WebResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    let result: Result<PaginatedResponse<serde_json::Value>, WebError> =
        task::spawn_blocking(move || {
            let graph_service = web_state.core_state.server.get_graph_service();

            // Build query with filter and pagination
            let filter_clause = params
                .filter
                .map(|f| format!(" WHERE {}", f))
                .unwrap_or_default();
            let sort_clause = params
                .sort_by
                .map(|s| {
                    let order = params.sort_order.unwrap_or_else(|| "ASC".to_string());
                    format!(" ORDER BY {} {}", s, order)
                })
                .unwrap_or_default();

            let query = format!(
                "USE {}; MATCH (v:{}) RETURN v{}{} SKIP {} LIMIT {}",
                space_name,
                tag_name,
                filter_clause,
                sort_clause,
                params.pagination.offset,
                params.pagination.limit
            );

            // Execute query - use session_id 0 for now (TODO: use actual session)
            match graph_service.execute(0, &query) {
                Ok(exec_result) => {
                    // Convert ExecutionResult to JSON values
                    let rows: Vec<serde_json::Value> = match exec_result {
                        crate::query::executor::ExecutionResult::Vertices(vertices) => vertices
                            .into_iter()
                            .map(|v| serde_json::json!({"vertex": v}))
                            .collect(),
                        crate::query::executor::ExecutionResult::Values(values) => values
                            .into_iter()
                            .map(|v| serde_json::json!({"value": v}))
                            .collect(),
                        _ => vec![],
                    };

                    // TODO: Get total count from core API
                    let total = rows.len() as i64;

                    Ok::<_, WebError>(PaginatedResponse::new(
                        rows,
                        total,
                        params.pagination.limit,
                        params.pagination.offset,
                    ))
                }
                Err(e) => Err(WebError::Query(format!("Failed to list vertices: {}", e))),
            }
        })
        .await
        .map_err(|e| WebError::Internal(format!("Task execution failed: {}", e)))?;

    Ok(Json(ApiResponse::success(result?)))
}

/// List edges by type
async fn list_edges_by_type<S: StorageClient + Clone + Send + Sync + 'static>(
    State(web_state): State<WebState<S>>,
    Path((space_name, edge_name)): Path<(String, String)>,
    Query(params): Query<DataFilterParams>,
) -> WebResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    let result: Result<PaginatedResponse<serde_json::Value>, WebError> =
        task::spawn_blocking(move || {
            let graph_service = web_state.core_state.server.get_graph_service();

            // Build query with filter and pagination
            let filter_clause = params
                .filter
                .map(|f| format!(" WHERE {}", f))
                .unwrap_or_default();
            let sort_clause = params
                .sort_by
                .map(|s| {
                    let order = params.sort_order.unwrap_or_else(|| "ASC".to_string());
                    format!(" ORDER BY {} {}", s, order)
                })
                .unwrap_or_default();

            let query = format!(
                "USE {}; MATCH ()-[e:{}]->() RETURN e{}{} SKIP {} LIMIT {}",
                space_name,
                edge_name,
                filter_clause,
                sort_clause,
                params.pagination.offset,
                params.pagination.limit
            );

            // Execute query - use session_id 0 for now (TODO: use actual session)
            match graph_service.execute(0, &query) {
                Ok(exec_result) => {
                    // Convert ExecutionResult to JSON values
                    let rows: Vec<serde_json::Value> = match exec_result {
                        crate::query::executor::ExecutionResult::Edges(edges) => edges
                            .into_iter()
                            .map(|e| serde_json::json!({"edge": e}))
                            .collect(),
                        crate::query::executor::ExecutionResult::Values(values) => values
                            .into_iter()
                            .map(|v| serde_json::json!({"value": v}))
                            .collect(),
                        _ => vec![],
                    };

                    // TODO: Get total count from core API
                    let total = rows.len() as i64;

                    Ok::<_, WebError>(PaginatedResponse::new(
                        rows,
                        total,
                        params.pagination.limit,
                        params.pagination.offset,
                    ))
                }
                Err(e) => Err(WebError::Query(format!("Failed to list edges: {}", e))),
            }
        })
        .await
        .map_err(|e| WebError::Internal(format!("Task execution failed: {}", e)))?;

    Ok(Json(ApiResponse::success(result?)))
}
