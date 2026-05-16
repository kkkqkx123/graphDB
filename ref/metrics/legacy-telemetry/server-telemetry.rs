//! Telemetry HTTP Server
//!
//! Provides an HTTP endpoint for exposing metrics data in JSON or Plain Text format.
//! This module is only available when the `server` feature is enabled.
//!
//! ## Endpoints
//!
//! - GET /metrics - Returns metrics in JSON or Plain Text format
//! - GET /health - Health check endpoint
//! - GET /metrics/slow_queries - Returns slow query details
//! - GET /metrics/slow_queries/stats - Returns slow query statistics
//! - GET /metrics/errors - Returns error statistics
//! - GET /metrics/errors/summary - Returns error summary with recent errors
//!
//! ## Query Parameters
//!
//! - format: "json" (default) or "text"/"prometheus"
//! - filter: prefix filter for metric names
//!
//! ## Example
//!
//! ```rust,no_run
//! use graphdb::api::core::telemetry::init_global_recorder;
//! use graphdb::api::server::telemetry_server::TelemetryServer;
//!
//! # async fn example() {
//! let recorder = init_global_recorder();
//! let server = TelemetryServer::with_default_config(recorder.into());
//! server.spawn();
//! # }
//! ```

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::core::telemetry::{MetricsSnapshot, TelemetryRecorder};
use crate::core::stats::StatsManager;

/// Query parameters for metrics endpoint
#[derive(Debug, Deserialize, Default)]
pub struct MetricsQuery {
    /// Output format: "json" (default) or "text"/"prometheus"
    pub format: Option<String>,
    /// Filter prefix for metric names
    pub filter: Option<String>,
}

/// Telemetry server state
#[derive(Clone)]
pub struct TelemetryState {
    recorder: Arc<TelemetryRecorder>,
    stats_manager: Option<Arc<StatsManager>>,
}

impl TelemetryState {
    /// Create new telemetry state with only recorder
    pub fn new(recorder: Arc<TelemetryRecorder>) -> Self {
        Self {
            recorder,
            stats_manager: None,
        }
    }

    /// Create new telemetry state with both recorder and stats manager
    pub fn with_stats_manager(
        recorder: Arc<TelemetryRecorder>,
        stats_manager: Arc<StatsManager>,
    ) -> Self {
        Self {
            recorder,
            stats_manager: Some(stats_manager),
        }
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.recorder.get_snapshot()
    }

    /// Get stats manager if available
    pub fn get_stats_manager(&self) -> Option<&Arc<StatsManager>> {
        self.stats_manager.as_ref()
    }
}

/// Create telemetry router with only recorder
pub fn create_telemetry_router(recorder: Arc<TelemetryRecorder>) -> Router {
    let state = TelemetryState::new(recorder);
    create_telemetry_router_with_stats_manager(state, None)
}

/// Create telemetry router with stats manager integration
pub fn create_telemetry_router_with_stats_manager(
    state: TelemetryState,
    _stats_manager: Option<Arc<StatsManager>>,
) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/metrics/slow_queries", get(slow_queries_handler))
        .route("/metrics/slow_queries/stats", get(slow_query_stats_handler))
        .route("/metrics/errors", get(errors_handler))
        .route("/metrics/errors/summary", get(error_summary_handler))
        .with_state(state)
}

/// Metrics endpoint handler
async fn metrics_handler(
    Query(query): Query<MetricsQuery>,
    axum::extract::State(state): axum::extract::State<TelemetryState>,
) -> Response {
    let mut snapshot = state.get_metrics();

    // Apply filter if provided
    if let Some(filter) = query.filter {
        snapshot = snapshot.filter_by_prefix(&filter);
    }

    // Determine output format
    let format = query.format.unwrap_or_else(|| "json".to_string());

    match format.as_str() {
        "json" => Json(snapshot).into_response(),
        "text" | "prometheus" => {
            let text = snapshot.to_text_format();
            ([("content-type", "text/plain; charset=utf-8")], text).into_response()
        }
        _ => (
            StatusCode::BAD_REQUEST,
            "Unsupported format. Use 'json' or 'text'",
        )
            .into_response(),
    }
}

/// Health check endpoint handler
async fn health_handler() -> Response {
    let health = serde_json::json!({
        "status": "ok",
        "service": "graphdb-telemetry",
    });
    Json(health).into_response()
}

/// Slow queries endpoint handler
async fn slow_queries_handler(
    Query(query): Query<SlowQueriesQuery>,
    axum::extract::State(state): axum::extract::State<TelemetryState>,
) -> Response {
    let limit = query.limit.unwrap_or(100);

    // Try to get slow queries from stats manager
    if let Some(stats_manager) = state.get_stats_manager() {
        let slow_queries = stats_manager.get_slow_queries(limit);
        let queries: Vec<serde_json::Value> = slow_queries
            .iter()
            .map(|profile| {
                serde_json::json!({
                    "trace_id": profile.trace_id.clone(),
                    "session_id": profile.session_id,
                    "query": profile.query_text.clone(),
                    "duration_ms": profile.total_duration_us as f64 / 1000.0,
                    "stages": {
                        "parse_ms": profile.stages.parse_ms(),
                        "validate_ms": profile.stages.validate_ms(),
                        "plan_ms": profile.stages.plan_ms(),
                        "optimize_ms": profile.stages.optimize_ms(),
                        "execute_ms": profile.stages.execute_ms(),
                    },
                    "status": match profile.status {
                        crate::core::stats::QueryStatus::Success => "success",
                        crate::core::stats::QueryStatus::Failed => "failed",
                    },
                    "error": profile.error_info.as_ref().map(|e| e.error_message.clone()),
                    "result_count": profile.result_count,
                })
            })
            .collect();

        Json(serde_json::json!({
            "count": queries.len(),
            "limit": limit,
            "queries": queries
        }))
        .into_response()
    } else {
        // Fallback to telemetry recorder metrics
        let snapshot = state.get_metrics().filter_by_prefix("graphdb_slow_query");
        Json(serde_json::json!({
            "message": "StatsManager not available, showing telemetry metrics only",
            "limit": limit,
            "metrics": snapshot,
            "note": "Use /metrics?filter=graphdb_slow_query for detailed telemetry data"
        }))
        .into_response()
    }
}

/// Slow query stats endpoint handler
async fn slow_query_stats_handler(
    axum::extract::State(state): axum::extract::State<TelemetryState>,
) -> Response {
    let mut stats = serde_json::json!({
        "slow_query_total": 0,
        "slow_query_active": 0,
        "slow_query_errors": 0,
    });

    // Get stats from telemetry recorder
    let snapshot = state.get_metrics().filter_by_prefix("graphdb_slow_query");
    for (name, value) in &snapshot.counters {
        if name == "graphdb_slow_query_total" {
            stats["slow_query_total"] = serde_json::json!(*value);
        }
        if name.contains("error") {
            stats["slow_query_errors"] =
                serde_json::json!(stats["slow_query_errors"].as_u64().unwrap_or(0) + value);
        }
    }

    for (name, value) in &snapshot.gauges {
        if name == "graphdb_slow_query_active" {
            stats["slow_query_active"] = serde_json::json!(*value);
        }
    }

    // Add histogram data if available
    if let Some((_, hist_data)) = snapshot
        .histograms
        .iter()
        .find(|(name, _)| name == "graphdb_slow_query_duration_seconds")
    {
        stats["duration_seconds"] = serde_json::json!({
            "count": hist_data.count,
            "sum": hist_data.sum,
            "min": hist_data.min,
            "max": hist_data.max,
            "p50": hist_data.p50,
            "p95": hist_data.p95,
            "p99": hist_data.p99,
        });
    }

    // Add stats from StatsManager if available
    if let Some(stats_manager) = state.get_stats_manager() {
        let slow_queries = stats_manager.get_slow_queries(1);
        stats["recent_slow_query_count"] = serde_json::json!(slow_queries.len());
        stats["from_stats_manager"] = serde_json::json!(true);
    }

    Json(stats).into_response()
}

/// Query parameters for slow queries endpoint
#[derive(Debug, Deserialize, Default)]
pub struct SlowQueriesQuery {
    /// Maximum number of slow queries to return (default: 100)
    pub limit: Option<usize>,
}

/// Error statistics endpoint handler
async fn errors_handler(
    axum::extract::State(state): axum::extract::State<TelemetryState>,
) -> Response {
    let mut errors = serde_json::json!({
        "by_type": {},
        "by_phase": {},
        "total": 0,
    });

    // Try to get error stats from StatsManager
    if let Some(stats_manager) = state.get_stats_manager() {
        let error_summary = stats_manager.get_error_summary();
        errors["by_type"] = serde_json::json!(error_summary.errors_by_type);
        errors["by_phase"] = serde_json::json!(error_summary.errors_by_phase);
        errors["total"] = serde_json::json!(error_summary.total_errors);
    } else {
        // Fallback to telemetry recorder metrics
        let snapshot = state.get_metrics().filter_by_prefix("graphdb_error");
        let mut by_type = serde_json::Map::new();
        let mut total = 0u64;

        for (name, value) in &snapshot.counters {
            total += value;
            if name.contains("by_type") {
                if let Some(error_type) = name.split("type=").last() {
                    by_type.insert(error_type.to_string(), serde_json::json!(value));
                }
            }
        }

        errors["by_type"] = serde_json::json!(by_type);
        errors["total"] = serde_json::json!(total);
    }

    Json(errors).into_response()
}

/// Error summary endpoint handler
async fn error_summary_handler(
    axum::extract::State(state): axum::extract::State<TelemetryState>,
) -> Response {
    let mut summary = serde_json::json!({
        "total_errors": 0,
        "errors_by_type": {},
        "errors_by_phase": {},
        "recent_errors": [],
    });

    // Try to get error stats from StatsManager
    if let Some(stats_manager) = state.get_stats_manager() {
        let error_summary = stats_manager.get_error_summary();
        summary["total_errors"] = serde_json::json!(error_summary.total_errors);
        summary["errors_by_type"] = serde_json::json!(error_summary.errors_by_type);
        summary["errors_by_phase"] = serde_json::json!(error_summary.errors_by_phase);

        // Get recent errors if available
        let recent_errors = stats_manager.get_recent_errors(10);
        if !recent_errors.is_empty() {
            summary["recent_errors"] = serde_json::json!(recent_errors
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "timestamp": e.timestamp,
                        "error_type": e.error_type.to_string(),
                        "error_phase": e.error_phase.to_string(),
                        "message": e.message,
                        "query": e.query_text,
                    })
                })
                .collect::<Vec<_>>());
        }
    } else {
        // Fallback to telemetry recorder metrics
        let snapshot = state.get_metrics().filter_by_prefix("graphdb_error");
        let mut total = 0u64;
        for (_, value) in &snapshot.counters {
            total += value;
        }
        summary["total_errors"] = serde_json::json!(total);
        summary["note"] = serde_json::json!("Use StatsManager for detailed error information");
    }

    Json(summary).into_response()
}

/// Telemetry server configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Bind address
    pub bind_address: String,
    /// Port number
    pub port: u16,
    /// Maximum histogram entries before cleanup
    pub max_histogram_entries: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 9090,
            max_histogram_entries: 10000,
            cleanup_interval_secs: 60,
        }
    }
}

/// Telemetry server handle
pub struct TelemetryServer {
    config: TelemetryConfig,
    recorder: Arc<TelemetryRecorder>,
}

impl TelemetryServer {
    /// Create a new telemetry server
    pub fn new(config: TelemetryConfig, recorder: Arc<TelemetryRecorder>) -> Self {
        Self { config, recorder }
    }

    /// Create with default configuration
    pub fn with_default_config(recorder: Arc<TelemetryRecorder>) -> Self {
        Self::new(TelemetryConfig::default(), recorder)
    }

    /// Start the telemetry server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        use axum::serve;
        use tokio::net::TcpListener;

        let router = create_telemetry_router(self.recorder.clone());
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);

        log::info!("Starting telemetry server on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        let server = serve(listener, router);

        // Spawn cleanup task if configured
        if self.config.cleanup_interval_secs > 0 {
            let recorder = self.recorder.clone();
            let interval = self.config.cleanup_interval_secs;
            let max_entries = self.config.max_histogram_entries;

            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(tokio::time::Duration::from_secs(interval));
                loop {
                    interval.tick().await;
                    recorder.cleanup_histograms(max_entries);
                }
            });
        }

        server.await?;
        Ok(())
    }

    /// Start the telemetry server in a separate task
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            if let Err(e) = self.start().await {
                log::error!("Telemetry server error: {}", e);
            }
        })
    }
}

/// Initialize and start telemetry server with default configuration
pub async fn start_telemetry_server(
    recorder: Arc<TelemetryRecorder>,
) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error>> {
    let server = TelemetryServer::with_default_config(recorder);
    Ok(server.spawn())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_test_router() -> Router {
        let recorder = Arc::new(TelemetryRecorder::new());
        create_telemetry_router(recorder)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let router = create_test_router();

        let response = router
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint_json() {
        let router = create_test_router();

        let response = router
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint_text() {
        let router = create_test_router();

        let response = router
            .oneshot(
                Request::get("/metrics?format=text")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
