//! 统计信息 HTTP 处理器

use axum::{
    extract::{Path, Query, State},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use serde_json;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::core::stats::MetricType;
use crate::storage::StorageClient;

/// 获取会话统计
pub async fn session<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<i64>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let session_manager = state.server.get_session_manager();
    let stats_manager = state.server.get_stats_manager();

    let session = session_manager
        .find_session(session_id)
        .ok_or_else(|| HttpError::NotFound(format!("会话不存在: {}", session_id)))?;

    // 获取会话相关的查询统计
    let session_queries = stats_manager.get_session_queries(session_id, 1000);
    let total_queries = session_queries.len() as u64;

    // 计算平均执行时间
    let avg_execution_time_ms = if total_queries > 0 {
        session_queries.iter().map(|q| q.total_duration_ms).sum::<u64>() as f64
            / total_queries as f64
    } else {
        0.0
    };

    Ok(JsonResponse(serde_json::json!({
        "session_id": session_id,
        "username": session.user(),
        "statistics": {
            "total_queries": total_queries,
            "total_changes": 0, // 暂不支持
            "last_insert_vertex_id": null,
            "last_insert_edge_id": null,
            "avg_execution_time_ms": avg_execution_time_ms,
        },
    })))
}

/// 获取查询统计
pub async fn queries<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Query(params): Query<QueryStatsParams>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let stats_manager = state.server.get_stats_manager();

    // 获取总查询数
    let total_queries = stats_manager.get_value(MetricType::NumQueries).unwrap_or(0);

    // 获取慢查询列表
    let slow_queries = stats_manager
        .get_slow_queries(10)
        .into_iter()
        .map(|profile| {
            serde_json::json!({
                "trace_id": profile.trace_id,
                "session_id": profile.session_id,
                "query": profile.query_text,
                "duration_ms": profile.total_duration_ms,
                "status": match profile.status {
                    crate::core::stats::QueryStatus::Success => "success",
                    crate::core::stats::QueryStatus::Failed => "failed",
                },
            })
        })
        .collect::<Vec<_>>();

    Ok(JsonResponse(serde_json::json!({
        "total_queries": total_queries,
        "slow_queries": slow_queries,
        "query_types": {
            "MATCH": 0,
            "CREATE": 0,
            "UPDATE": 0,
            "DELETE": 0,
        },
        "from": params.from,
        "to": params.to,
    })))
}

/// 获取数据库统计
pub async fn database<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let stats_manager = state.server.get_stats_manager();

    // 获取查询相关统计
    let total_queries = stats_manager.get_value(MetricType::NumQueries).unwrap_or(0);
    let active_queries = stats_manager.get_value(MetricType::NumActiveQueries).unwrap_or(0);

    // 获取缓存大小
    let cache_size = stats_manager.query_cache_size();

    Ok(JsonResponse(serde_json::json!({
        "spaces": {
            "count": 0, // 暂不支持
            "total_vertices": 0,
            "total_edges": 0,
        },
        "storage": {
            "total_size_bytes": 0,
            "index_size_bytes": 0,
            "data_size_bytes": 0,
        },
        "performance": {
            "total_queries": total_queries,
            "active_queries": active_queries,
            "query_cache_size": cache_size,
            "queries_per_second": 0.0,
            "avg_latency_ms": 0.0,
            "cache_hit_rate": 0.0,
        },
    })))
}

/// 获取系统资源使用
pub async fn system<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let session_manager = state.server.get_session_manager();

    // 获取连接统计
    let active_connections = session_manager.active_session_count().await;
    let max_connections = session_manager.max_connections();

    Ok(JsonResponse(serde_json::json!({
        "cpu_usage_percent": 0.0, // 暂不支持
        "memory_usage": {
            "used_bytes": 0,
            "total_bytes": 0,
        },
        "connections": {
            "active": active_connections,
            "total": active_connections,
            "max": max_connections,
        },
    })))
}

/// 查询统计参数
#[derive(Debug, Deserialize)]
pub struct QueryStatsParams {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}
