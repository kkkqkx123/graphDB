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
        session_queries
            .iter()
            .map(|q| q.total_duration_ms)
            .sum::<u64>() as f64
            / total_queries as f64
    } else {
        0.0
    };

    // 获取会话级变更统计
    let session_stats = session.statistics();
    let total_changes = session_stats.total_changes();
    let last_insert_vertex_id = session_stats.last_insert_vertex_id();
    let last_insert_edge_id = session_stats.last_insert_edge_id();

    Ok(JsonResponse(serde_json::json!({
        "session_id": session_id,
        "username": session.user(),
        "statistics": {
            "total_queries": total_queries,
            "total_changes": total_changes,
            "last_insert_vertex_id": last_insert_vertex_id,
            "last_insert_edge_id": last_insert_edge_id,
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

    // 获取各类型查询统计
    let match_queries = stats_manager
        .get_value(MetricType::NumMatchQueries)
        .unwrap_or(0);
    let create_queries = stats_manager
        .get_value(MetricType::NumCreateQueries)
        .unwrap_or(0);
    let update_queries = stats_manager
        .get_value(MetricType::NumUpdateQueries)
        .unwrap_or(0);
    let delete_queries = stats_manager
        .get_value(MetricType::NumDeleteQueries)
        .unwrap_or(0);
    let insert_queries = stats_manager
        .get_value(MetricType::NumInsertQueries)
        .unwrap_or(0);
    let go_queries = stats_manager
        .get_value(MetricType::NumGoQueries)
        .unwrap_or(0);
    let fetch_queries = stats_manager
        .get_value(MetricType::NumFetchQueries)
        .unwrap_or(0);
    let lookup_queries = stats_manager
        .get_value(MetricType::NumLookupQueries)
        .unwrap_or(0);
    let show_queries = stats_manager
        .get_value(MetricType::NumShowQueries)
        .unwrap_or(0);

    Ok(JsonResponse(serde_json::json!({
        "total_queries": total_queries,
        "slow_queries": slow_queries,
        "query_types": {
            "MATCH": match_queries,
            "CREATE": create_queries,
            "UPDATE": update_queries,
            "DELETE": delete_queries,
            "INSERT": insert_queries,
            "GO": go_queries,
            "FETCH": fetch_queries,
            "LOOKUP": lookup_queries,
            "SHOW": show_queries,
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
    let storage = state.server.get_storage();

    // 在异步上下文中使用 spawn_blocking 获取存储锁
    let storage_stats = {
        let storage = storage.clone();
        tokio::task::spawn_blocking(move || {
            let storage = storage.lock();
            storage.get_storage_stats()
        })
        .await
        .map_err(|e| HttpError::internal(format!("获取存储统计失败: {:?}", e)))?
    };

    // 获取查询相关统计
    let total_queries = stats_manager.get_value(MetricType::NumQueries).unwrap_or(0);
    let active_queries = stats_manager
        .get_value(MetricType::NumActiveQueries)
        .unwrap_or(0);

    // 获取缓存大小
    let cache_size = stats_manager.query_cache_size();

    // 计算性能指标
    let recent_queries = stats_manager.get_recent_queries(100);
    let avg_latency_ms = if recent_queries.is_empty() {
        0.0
    } else {
        recent_queries
            .iter()
            .map(|q| q.total_duration_ms)
            .sum::<u64>() as f64
            / recent_queries.len() as f64
    };

    // 计算 QPS（基于最近100个查询的时间跨度）
    let qps = if recent_queries.len() >= 2 {
        let first = recent_queries.first().map(|q| q.start_time);
        let last = recent_queries.last().map(|q| q.start_time);
        if let (Some(first), Some(last)) = (first, last) {
            // 计算时间差，如果 last < first 则返回 0
            let duration = last.saturating_duration_since(first);
            let duration_secs = duration.as_secs() as f64;
            if duration_secs > 0.0 {
                recent_queries.len() as f64 / duration_secs
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(JsonResponse(serde_json::json!({
        "spaces": {
            "count": storage_stats.total_spaces,
            "total_vertices": storage_stats.total_vertices,
            "total_edges": storage_stats.total_edges,
        },
        "storage": {
            "total_size_bytes": 0, // 需要扩展 StorageStats
            "index_size_bytes": 0,
            "data_size_bytes": 0,
        },
        "performance": {
            "total_queries": total_queries,
            "active_queries": active_queries,
            "query_cache_size": cache_size,
            "queries_per_second": qps,
            "avg_latency_ms": avg_latency_ms,
            "cache_hit_rate": 0.0, // 需要查询缓存层支持
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

    // 获取系统资源使用情况（使用 sysinfo）
    let (memory_used, memory_total) = get_memory_info();
    let cpu_usage = get_cpu_usage();

    Ok(JsonResponse(serde_json::json!({
        "cpu_usage_percent": cpu_usage,
        "memory_usage": {
            "used_bytes": memory_used,
            "total_bytes": memory_total,
        },
        "connections": {
            "active": active_connections,
            "total": active_connections,
            "max": max_connections,
        },
    })))
}

/// 获取内存信息（已使用字节数和总字节数）
/// 使用 sysinfo crate 实现跨平台支持
fn get_memory_info() -> (u64, u64) {
    use sysinfo::System;

    // 创建系统信息实例并刷新内存信息
    let mut sys = System::new();
    sys.refresh_memory();

    // 获取系统总内存和已使用内存（转换为字节）
    let total_memory = sys.total_memory() * 1024;
    let used_memory = sys.used_memory() * 1024;

    (used_memory, total_memory)
}

/// 获取 CPU 使用率百分比
/// 使用 sysinfo crate 实现跨平台支持
fn get_cpu_usage() -> f64 {
    use sysinfo::System;

    // 创建系统信息实例
    let mut sys = System::new();

    // 刷新 CPU 使用率信息
    sys.refresh_cpu_usage();

    // 计算平均 CPU 使用率
    let cpus = sys.cpus();
    if cpus.is_empty() {
        0.0
    } else {
        let avg_usage: f32 =
            cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        avg_usage as f64
    }
}

/// 查询统计参数
#[derive(Debug, Deserialize)]
pub struct QueryStatsParams {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}
