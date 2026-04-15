//! Telemetry HTTP Server
//!
//! Provides an HTTP endpoint for exposing metrics data in JSON or Plain Text format.
//! This module is only available when the `server` feature is enabled.
//!
//! Endpoints:
//! - GET /metrics - Returns metrics in JSON or Plain Text format
//! - GET /health - Health check endpoint
//!
//! Query Parameters:
//! - format: "json" (default) or "text"/"prometheus"
//! - filter: prefix filter for metric names

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::telemetry::{MetricsSnapshot, TelemetryRecorder};

/// Query parameters for metrics endpoint
#[derive(Debug, Deserialize, Default)]
pub struct MetricsQuery {
    /// Output format: "json" (default) or "text"/"prometheus"
    pub format: Option<String>,
    /// Filter prefix for metric names
    pub filter: Option<String>,
}

/// Telemetry server state
#[derive(Debug, Clone)]
pub struct TelemetryState {
    recorder: Arc<TelemetryRecorder>,
}

impl TelemetryState {
    /// Create new telemetry state
    pub fn new(recorder: Arc<TelemetryRecorder>) -> Self {
        Self { recorder }
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.recorder.get_snapshot()
    }
}

/// Create telemetry router
pub fn create_telemetry_router(recorder: Arc<TelemetryRecorder>) -> Router {
    let state = TelemetryState::new(recorder);

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
        _ => (StatusCode::BAD_REQUEST, "Unsupported format. Use 'json' or 'text'").into_response(),
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
) -> Response {
    let limit = query.limit.unwrap_or(100);
    
    // Get stats manager from GraphService
    // Note: This requires GraphService to expose a method to get recent slow queries
    // For now, return empty response
    let queries = serde_json::json!({
        "message": "Slow queries endpoint - implementation requires GraphService integration",
        "limit": limit,
        "queries": Vec::<String>::new()
    });
    
    Json(queries).into_response()
}

/// Slow query stats endpoint handler
async fn slow_query_stats_handler() -> Response {
    // Return basic stats
    let stats = serde_json::json!({
        "slow_query_total": 0,
        "slow_query_duration_seconds": {
            "note": "Available via /metrics endpoint with filter=graphdb_slow_query"
        },
        "message": "Use /metrics?filter=graphdb_slow_query for detailed metrics"
    });
    
    Json(stats).into_response()
}

/// Query parameters for slow queries endpoint
#[derive(Debug, Deserialize, Default)]
pub struct SlowQueriesQuery {
    /// Maximum number of slow queries to return (default: 100)
    pub limit: Option<usize>,
}

/// Error statistics endpoint handler
async fn errors_handler() -> Response {
    // Get stats manager from GraphService
    // For now, return placeholder response
    let errors = serde_json::json!({
        "message": "Error statistics endpoint - requires GraphService integration",
        "by_type": {},
        "by_phase": {}
    });
    
    Json(errors).into_response()
}

/// Error summary endpoint handler
async fn error_summary_handler() -> Response {
    // Get stats manager from GraphService
    // For now, return placeholder response
    let summary = serde_json::json!({
        "message": "Error summary endpoint - requires GraphService integration",
        "total_errors": 0,
        "errors_by_type": {},
        "errors_by_phase": {}
    });
    
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
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
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
    async fn test_metrics_json_endpoint() {
        let router = create_test_router();

        let response = router
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_text_endpoint() {
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

    #[tokio::test]
    async fn test_metrics_filter() {
        let router = create_test_router();

        let response = router
            .oneshot(
                Request::get("/metrics?filter=graphdb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
