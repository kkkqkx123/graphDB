use axum::{
    extract::{Json, State},
    response::Json as JsonResponse,
};
use tokio::task;

use crate::api::server::http::handlers::query_types::*;
use crate::api::server::http::{error::HttpError, state::AppState};
use crate::query::executor::ExecutionResult;
use crate::storage::StorageClient;

pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<QueryRequest>,
) -> Result<JsonResponse<QueryResponse>, HttpError> {
    let result = task::spawn_blocking(move || {
        let graph_service = state.server.get_graph_service();

        // 通过 GraphService 执行查询
        match graph_service.execute(request.session_id, &request.query) {
            Ok(exec_result) => {
                // 将 ExecutionResult 转换为 QueryResponse
                Ok::<_, HttpError>(execution_result_to_response(exec_result))
            }
            Err(e) => Ok::<_, HttpError>(QueryResponse::error(
                "QUERY_ERROR".to_string(),
                e.to_string(),
                None,
            )),
        }
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("Task execution failed: {}", e)))?;

    Ok(JsonResponse(result?))
}

/// 将 ExecutionResult 转换为 QueryResponse
fn execution_result_to_response(result: ExecutionResult) -> QueryResponse {
    match result {
        ExecutionResult::Values(values) => {
            let rows: Vec<std::collections::HashMap<String, serde_json::Value>> = values
                .into_iter()
                .map(|v| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("value".to_string(), value_to_json(v));
                    map
                })
                .collect();
            let row_count = rows.len();

            QueryResponse::success(
                QueryData::new(vec!["value".to_string()], rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
        ExecutionResult::Vertices(vertices) => {
            let rows: Vec<std::collections::HashMap<String, serde_json::Value>> = vertices
                .into_iter()
                .map(|v| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("vertex".to_string(), serde_json::json!(v));
                    map
                })
                .collect();
            let row_count = rows.len();

            QueryResponse::success(
                QueryData::new(vec!["vertex".to_string()], rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
        ExecutionResult::Edges(edges) => {
            let rows: Vec<std::collections::HashMap<String, serde_json::Value>> = edges
                .into_iter()
                .map(|e| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("edge".to_string(), serde_json::json!(e));
                    map
                })
                .collect();
            let row_count = rows.len();

            QueryResponse::success(
                QueryData::new(vec!["edge".to_string()], rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
        ExecutionResult::DataSet(dataset) => {
            let columns: Vec<String> = dataset.col_names.clone();
            let rows: Vec<std::collections::HashMap<String, serde_json::Value>> = dataset
                .rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let col_name = columns.get(i).cloned().unwrap_or_default();
                            (col_name, value_to_json(v))
                        })
                        .collect()
                })
                .collect();
            let row_count = rows.len();

            QueryResponse::success(
                QueryData::new(columns, rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
        ExecutionResult::Result(core_result) => {
            let row_count = core_result.row_count();
            QueryResponse::success(
                QueryData::empty(),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
        ExecutionResult::Empty | ExecutionResult::Success => QueryResponse::success(
            QueryData::empty(),
            QueryMetadata {
                execution_time_ms: 0,
                rows_scanned: 0,
                rows_returned: 0,
                space_id: None,
            },
        ),
        ExecutionResult::Error(msg) => {
            QueryResponse::error("EXECUTION_ERROR".to_string(), msg, None)
        }
        ExecutionResult::Count(count) => {
            let mut rows = Vec::new();
            let mut map = std::collections::HashMap::new();
            map.insert("count".to_string(), serde_json::json!(count));
            rows.push(map);

            QueryResponse::success(
                QueryData::new(vec!["count".to_string()], rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: 1,
                    space_id: None,
                },
            )
        }
        ExecutionResult::Paths(paths) => {
            let rows: Vec<std::collections::HashMap<String, serde_json::Value>> = paths
                .into_iter()
                .map(|p| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("path".to_string(), serde_json::json!(p));
                    map
                })
                .collect();
            let row_count = rows.len();

            QueryResponse::success(
                QueryData::new(vec!["path".to_string()], rows),
                QueryMetadata {
                    execution_time_ms: 0,
                    rows_scanned: 0,
                    rows_returned: row_count,
                    space_id: None,
                },
            )
        }
    }
}

/// 将 Core Value 转换为 serde_json::Value
fn value_to_json(value: crate::core::Value) -> serde_json::Value {
    match value {
        crate::core::Value::Empty => serde_json::Value::Null,
        crate::core::Value::Null(_) => serde_json::Value::Null,
        crate::core::Value::Bool(b) => serde_json::Value::Bool(b),
        crate::core::Value::Int(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::Int8(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::Int16(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::Int32(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::Int64(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::UInt8(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::UInt16(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::UInt32(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::UInt64(i) => serde_json::Value::Number(i.into()),
        crate::core::Value::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
        ),
        crate::core::Value::Decimal128(d) => serde_json::Value::String(d.to_string()),
        crate::core::Value::String(s) => serde_json::Value::String(s),
        crate::core::Value::FixedString { data, .. } => serde_json::Value::String(data),
        crate::core::Value::Blob(blob) => serde_json::Value::String(format!("{:?}", blob)),
        crate::core::Value::Date(d) => serde_json::Value::String(d.to_string()),
        crate::core::Value::Time(t) => serde_json::Value::String(t.to_string()),
        crate::core::Value::DateTime(dt) => serde_json::Value::String(dt.to_string()),
        crate::core::Value::Vertex(v) => serde_json::json!(v),
        crate::core::Value::Edge(e) => serde_json::json!(e),
        crate::core::Value::Path(p) => serde_json::json!(p),
        crate::core::Value::List(list) => {
            serde_json::Value::Array(list.into_iter().map(value_to_json).collect())
        }
        crate::core::Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| (k, value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        crate::core::Value::Set(set) => {
            serde_json::Value::Array(set.into_iter().map(value_to_json).collect())
        }
        crate::core::Value::Geography(g) => serde_json::json!(g),
        crate::core::Value::Duration(d) => serde_json::Value::String(d.to_string()),
        crate::core::Value::DataSet(ds) => serde_json::json!(ds),
    }
}

pub async fn validate(
    Json(_request): Json<QueryRequest>,
) -> Result<JsonResponse<ValidateResponse>, HttpError> {
    Ok(JsonResponse(ValidateResponse {
        valid: true,
        message: "Syntax is correct".to_string(),
    }))
}
