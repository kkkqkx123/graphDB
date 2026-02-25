use axum::{
    extract::{Path, State, Json},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use tokio::task;

use crate::api::server::http::{
    state::AppState,
    error::HttpError,
};
use crate::storage::StorageClient;
use crate::api::core::{SpaceConfig, PropertyDef};
use crate::core::DataType;

// ==================== Space 相关 ====================

#[derive(Debug, Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    #[serde(default)]
    pub vid_type: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

/// 创建图空间
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
                "message": "图空间创建成功",
                "space_name": request.name,
            })),
            Err(e) => Err(HttpError::InternalError(format!("创建图空间失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 获取图空间
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
            Err(_e) => Err(HttpError::NotFound(format!("图空间 '{}' 不存在", name))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 删除图空间
pub async fn drop_space<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        match schema_api.drop_space(&name) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "图空间删除成功",
                "space_name": name,
            })),
            Err(e) => Err(HttpError::InternalError(format!("删除图空间失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 列出所有图空间
pub async fn list_spaces<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // 暂时返回空列表，因为 SchemaApi 没有 list_spaces 方法
    Ok(JsonResponse(serde_json::json!({
        "spaces": [],
        "note": "此功能待实现",
    })))
}

// ==================== Tag 相关 ====================

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

/// 创建标签
pub async fn create_tag<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(space_name): Path<String>,
    Json(request): Json<CreateTagRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        // 获取空间 ID
        let space_id = match schema_api.use_space(&space_name) {
            Ok(id) => id,
            Err(_) => return Err(HttpError::NotFound(format!("图空间 '{}' 不存在", space_name))),
        };

        // 转换属性定义
        let properties: Vec<PropertyDef> = request.properties.into_iter().map(|p| {
            PropertyDef {
                name: p.name,
                data_type: parse_data_type(&p.data_type),
                nullable: p.nullable,
                default_value: None,
                comment: None,
            }
        }).collect();

        match schema_api.create_tag(space_id, &request.name, properties) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "标签创建成功",
                "tag_name": request.name,
                "space_name": space_name,
            })),
            Err(e) => Err(HttpError::InternalError(format!("创建标签失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 列出所有标签
pub async fn list_tags<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(space_name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // 暂时返回空列表，因为 SchemaApi 没有 list_tags 方法
    Ok(JsonResponse(serde_json::json!({
        "tags": [],
        "space_name": space_name,
        "note": "此功能待实现",
    })))
}

// ==================== Edge Type 相关 ====================

#[derive(Debug, Deserialize)]
pub struct CreateEdgeTypeRequest {
    pub name: String,
    pub properties: Vec<PropertyDefInput>,
}

/// 创建边类型
pub async fn create_edge_type<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(space_name): Path<String>,
    Json(request): Json<CreateEdgeTypeRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let result = task::spawn_blocking(move || {
        let schema_api = state.server.get_schema_api();

        // 获取空间 ID
        let space_id = match schema_api.use_space(&space_name) {
            Ok(id) => id,
            Err(_) => return Err(HttpError::NotFound(format!("图空间 '{}' 不存在", space_name))),
        };

        // 转换属性定义
        let properties: Vec<PropertyDef> = request.properties.into_iter().map(|p| {
            PropertyDef {
                name: p.name,
                data_type: parse_data_type(&p.data_type),
                nullable: p.nullable,
                default_value: None,
                comment: None,
            }
        }).collect();

        match schema_api.create_edge_type(space_id, &request.name, properties) {
            Ok(()) => Ok::<_, HttpError>(serde_json::json!({
                "message": "边类型创建成功",
                "edge_type_name": request.name,
                "space_name": space_name,
            })),
            Err(e) => Err(HttpError::InternalError(format!("创建边类型失败: {}", e))),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 列出所有边类型
pub async fn list_edge_types<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path(space_name): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // 暂时返回空列表，因为 SchemaApi 没有 list_edge_types 方法
    Ok(JsonResponse(serde_json::json!({
        "edge_types": [],
        "space_name": space_name,
        "note": "此功能待实现",
    })))
}

// ==================== 辅助函数 ====================

fn parse_data_type(type_str: &str) -> DataType {
    match type_str.to_uppercase().as_str() {
        "INT" | "INTEGER" => DataType::Int,
        "FLOAT" | "DOUBLE" => DataType::Float,
        "STRING" | "STR" => DataType::String,
        "BOOL" | "BOOLEAN" => DataType::Bool,
        _ => DataType::String, // 默认使用字符串类型
    }
}
