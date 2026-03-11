//! 流式结果 HTTP 处理器

use axum::{
    extract::{Json, State},
    response::{sse::Event, Sse},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::wrappers::ReceiverStream;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::query::executor::ExecutionResult;
use crate::storage::StorageClient;

/// 流式查询请求
#[derive(Debug, Clone, Deserialize)]
pub struct StreamQueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    100
}

/// 流式结果数据项
#[derive(Debug, Serialize)]
struct StreamDataItem {
    pub row: serde_json::Value,
    pub index: usize,
}

/// 流式结果元数据
#[derive(Debug, Serialize)]
struct StreamMetadata {
    pub rows_returned: usize,
    pub execution_time_ms: u64,
    pub columns: Vec<String>,
}

/// 执行查询并流式返回结果
pub async fn execute_stream<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<StreamQueryRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, HttpError>> + Send + 'static>, HttpError> {
    let batch_size = request.batch_size.max(1).min(1000);
    let server = state.server.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, HttpError>>(batch_size);

    tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let graph_service = server.get_graph_service();
        let request = request.clone();

        // 执行查询
        let exec_result = match tokio::task::spawn_blocking({
            let graph_service = graph_service.clone();
            move || {
                graph_service.execute(request.session_id, &request.query)
            }
        }).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let error_msg = json!({
                    "error": true,
                    "message": e,
                    "code": "QUERY_ERROR"
                });
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(error_msg.to_string()))).await;
                let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
                return;
            }
            Err(e) => {
                let error_msg = json!({
                    "error": true,
                    "message": format!("Task execution failed: {}", e),
                    "code": "INTERNAL_ERROR"
                });
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(error_msg.to_string()))).await;
                let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
                return;
            }
        };

        // 将执行结果转换为流式数据
        let (rows, columns) = execution_result_to_stream_data(exec_result);
        let total_rows = rows.len();

        // 分批发送数据
        for (index, row) in rows.into_iter().enumerate() {
            let item = StreamDataItem {
                row,
                index,
            };

            if let Ok(data) = serde_json::to_string(&item) {
                if tx.send(Ok(Event::default().data(data))).await.is_err() {
                    // 客户端断开连接
                    return;
                }
            }

            // 每批次发送后短暂休眠，避免阻塞
            if (index + 1) % batch_size == 0 {
                tokio::task::yield_now().await;
            }
        }

        // 发送元数据
        let metadata = StreamMetadata {
            rows_returned: total_rows,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            columns,
        };

        if let Ok(meta_str) = serde_json::to_string(&metadata) {
            let _ = tx.send(Ok(Event::default()
                .event("metadata")
                .data(meta_str))).await;
        }

        // 发送完成事件
        let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keepalive"),
    ))
}

/// 将 ExecutionResult 转换为流式数据
fn execution_result_to_stream_data(
    result: ExecutionResult,
) -> (Vec<serde_json::Value>, Vec<String>) {
    match result {
        ExecutionResult::Values(values) => {
            let rows: Vec<serde_json::Value> = values
                .into_iter()
                .map(|v| json!({"value": value_to_json(v)}))
                .collect();
            (rows, vec!["value".to_string()])
        }
        ExecutionResult::Vertices(vertices) => {
            let rows: Vec<serde_json::Value> = vertices
                .into_iter()
                .map(|v| json!({"vertex": v}))
                .collect();
            (rows, vec!["vertex".to_string()])
        }
        ExecutionResult::Edges(edges) => {
            let rows: Vec<serde_json::Value> = edges
                .into_iter()
                .map(|e| json!({"edge": e}))
                .collect();
            (rows, vec!["edge".to_string()])
        }
        ExecutionResult::DataSet(dataset) => {
            let columns = dataset.col_names.clone();
            let rows: Vec<serde_json::Value> = dataset
                .rows
                .into_iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = row
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let col_name = columns.get(i).cloned().unwrap_or_default();
                            (col_name, value_to_json(v))
                        })
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            (rows, columns)
        }
        ExecutionResult::Paths(paths) => {
            let rows: Vec<serde_json::Value> = paths
                .into_iter()
                .map(|p| json!({"path": p}))
                .collect();
            (rows, vec!["path".to_string()])
        }
        ExecutionResult::Count(count) => {
            (vec![json!({"count": count})], vec!["count".to_string()])
        }
        ExecutionResult::Result(core_result) => {
            (vec![json!({"result": core_result.row_count()})], vec!["result".to_string()])
        }
        ExecutionResult::Empty | ExecutionResult::Success => {
            (vec![], vec![])
        }
        ExecutionResult::Error(msg) => {
            (vec![json!({"error": msg})], vec!["error".to_string()])
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
