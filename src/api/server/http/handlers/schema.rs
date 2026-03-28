use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use tokio::task;

use crate::api::core::{PropertyDef, SpaceConfig};
use crate::api::server::http::{error::HttpError, state::AppState};
use crate::core::DataType;
use crate::storage::StorageClient;

// ==================== Space related ====================

#[derive(Debug, Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    #[serde(default)]
    pub vid_type: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

/// Creating a graph space
pub async fn create_space<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateSpaceRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        let config = SpaceConfig {
            vid_type: parse_data_type(&request.vid_type.unwrap_or_else(|| "STRING".to_string())),
            comment: request.comment,
            partition_num: 100,
            replica_factor: 1,
        };

        match schema_api.create_space(&request.name, config) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "Space created successfully",
                "space_name": request.name,
            })),
            Err(e) => Err(HttpError::InternalError(format!(
                "Failed to create space: {}",
                e
            ))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// Getting the graph space
pub async fn get_space<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        match schema_api.use_space(&name) {
            Ok(space_id) => Ok::<_, HttpError>(serde_json::json!({
                "space": {
                    "name": name,
                    "id": space_id,
                }
            })),
            Err(_e) => Err(HttpError::NotFound(format!("Space '{}' not found", name))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// Deletion of map space
pub async fn drop_space<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        match schema_api.drop_space(&name) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "Space deleted successfully",
                "space_name": name,
            })),
            Err(e) => Err(HttpError::InternalError(format!(
                "Failed to delete space: {}",
                e
            ))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// List all graph spaces
pub async fn list_spaces<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // Returns an empty list for now, since SchemaApi doesn't have a list_spaces method.
    Ok(JsonResponse(serde_json::json!({
        "spaces": [],
        "note": "This feature is pending implementation",
    })))
}

// ==================== Tag related ====================

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub properties: Vec<PropertyDefInput>,
}

#[derive(Debug, Deserialize)]
pub struct PropertyDefInput {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub nullable: bool,
}

/// Creating Tags
pub async fn create_tag<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(space_name): Path<String>,
    Json(request): Json<CreateTagRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        // Get Space ID
        let space_id = match schema_api.use_space(&space_name) {
            Ok(id) => id,
            Err(_) => {
                return Err(HttpError::NotFound(format!(
                    "The graph space '{}' does not exist.",
                    space_name
                )))
            }
        };

        // Conversion Attribute Definition
        let properties: Vec<PropertyDef> = request
            .properties
            .into_iter()
            .map(|p| PropertyDef {
                name: p.name,
                data_type: parse_data_type(&p.data_type),
                nullable: p.nullable,
                default_value: None,
                comment: None,
            })
            .collect();

        match schema_api.create_tag(space_id, &request.name, properties) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "Tag created successfully",
                "tag_name": request.name,
                "space_name": space_name,
            })),
            Err(e) => Err(HttpError::InternalError(format!(
                "Failed to create tag: {}",
                e
            ))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// List all tags
pub async fn list_tags<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(space_name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // Returns an empty list for now, since SchemaApi doesn't have a list_tags method.
    Ok(JsonResponse(serde_json::json!({
        "tags": [],
        "space_name": space_name,
        "note": "This feature is pending implementation",
    })))
}

// ==================== Edge Type related ====================

#[derive(Debug, Deserialize)]
pub struct CreateEdgeTypeRequest {
    pub name: String,
    pub properties: Vec<PropertyDefInput>,
}

/// Creating Edge Types
pub async fn create_edge_type<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(space_name): Path<String>,
    Json(request): Json<CreateEdgeTypeRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        // Get Space ID
        let space_id = match schema_api.use_space(&space_name) {
            Ok(id) => id,
            Err(_) => {
                return Err(HttpError::NotFound(format!(
                    "The graph space '{}' does not exist.",
                    space_name
                )))
            }
        };

        // Conversion Attribute Definition
        let properties: Vec<PropertyDef> = request
            .properties
            .into_iter()
            .map(|p| PropertyDef {
                name: p.name,
                data_type: parse_data_type(&p.data_type),
                nullable: p.nullable,
                default_value: None,
                comment: None,
            })
            .collect();

        match schema_api.create_edge_type(space_id, &request.name, properties) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "Edge type created successfully",
                "edge_type_name": request.name,
                "space_name": space_name,
            })),
            Err(e) => Err(HttpError::InternalError(format!(
                "Failed to create edge type: {}",
                e
            ))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// List all edge types
pub async fn list_edge_types<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(space_name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // Returns an empty list for now, since SchemaApi doesn't have a list_edge_types method.
    Ok(JsonResponse(serde_json::json!({
        "edge_types": [],
        "space_name": space_name,
        "note": "This feature is pending implementation",
    })))
}

// ==================== Auxiliary Functions ====================

fn parse_data_type(type_str: &str) -> DataType {
    match type_str.to_uppercase().as_str() {
        "INT" | "INTEGER" => DataType::Int,
        "FLOAT" | "DOUBLE" => DataType::Float,
        "STRING" | "STR" => DataType::String,
        "BOOL" | "BOOLEAN" => DataType::Bool,
        _ => DataType::String, // String types are used by default
    }
}
