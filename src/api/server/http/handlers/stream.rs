//! 流式结果 HTTP 处理器

use axum::{
    extract::{Json, State},
    response::{sse::Event, Sse},
};
use serde::Deserialize;
use tokio_stream::wrappers::ReceiverStream;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::api::server::http::handlers::query_types::QueryRequest;
use crate::storage::StorageClient;

/// 执行查询并流式返回结果
pub async fn execute_stream<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Json(_request): Json<QueryRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, HttpError>> + Send + 'static>, HttpError> {
    // TODO: 实现实际的流式查询
    // 目前返回模拟流
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, HttpError>>(100);

    tokio::spawn(async move {
        // 模拟发送数据
        let _ = tx.send(Ok(Event::default().data(r#"{"n": {"id": 1, "name": "Alice"}}"#))).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let _ = tx.send(Ok(Event::default().data(r#"{"n": {"id": 2, "name": "Bob"}}"#))).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let _ = tx.send(Ok(Event::default().event("metadata").data(r#"{"rows_returned": 2, "execution_time_ms": 100}"#))).await;
        let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keepalive"),
    ))
}

/// 流式查询请求
#[derive(Debug, Deserialize)]
pub struct StreamQueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default)]
    pub batch_size: usize,
}
