// Server metrics - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

use metrics::{counter, gauge, histogram};

/// Initialize server metrics
pub fn init_server_metrics() {
    // Initialize server-specific metrics
    gauge!("inversearch.server.startup_time").set(0.0);
    counter!("inversearch.server.requests_total").increment(0);
    let _ = histogram!("inversearch.server.request_duration_seconds");
}

/// Record a request
pub fn record_request(method: &str) {
    counter!("inversearch.server.requests_total", "method" => method.to_string()).increment(1);
}

/// Record request duration
pub fn record_request_duration(method: &str, duration_secs: f64) {
    histogram!(
        "inversearch.server.request_duration_seconds",
        "method" => method.to_string()
    )
    .record(duration_secs);
}
