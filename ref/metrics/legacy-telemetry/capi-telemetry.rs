//! C API Telemetry Module
//!
//! Provides C interface for accessing telemetry metrics data

use crate::api::core::telemetry::init_global_recorder;
use crate::api::embedded::c_api::error::graphdb_error_code_t;
use crate::api::embedded::telemetry::EmbeddedTelemetry;
use crate::core::stats::GlobalMetrics;
use std::ffi::{c_char, c_int, CStr, CString};

/// Initialize telemetry system
///
/// # Returns
/// - Success: GRAPHDB_OK
#[no_mangle]
pub extern "C" fn graphdb_telemetry_init() -> c_int {
    init_global_recorder();
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// Get all metrics in JSON format
///
/// # Arguments
/// - `out_json`: Output parameter for JSON string (must be freed with graphdb_free_string)
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `out_json` must be a valid pointer to store the result
/// - The returned string must be freed using `graphdb_free_string`
#[no_mangle]
pub unsafe extern "C" fn graphdb_telemetry_get_json(out_json: *mut *mut c_char) -> c_int {
    if out_json.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let snapshot = EmbeddedTelemetry::get_metrics();
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => match CString::new(json) {
            Ok(cstr) => {
                *out_json = cstr.into_raw();
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(_) => graphdb_error_code_t::GRAPHDB_NOMEM as c_int,
        },
        Err(_) => graphdb_error_code_t::GRAPHDB_ERROR as c_int,
    }
}

/// Get metrics in Prometheus text format
///
/// # Arguments
/// - `out_text`: Output parameter for text string (must be freed with graphdb_free_string)
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `out_text` must be a valid pointer to store the result
/// - The returned string must be freed using `graphdb_free_string`
#[no_mangle]
pub unsafe extern "C" fn graphdb_telemetry_get_text(out_text: *mut *mut c_char) -> c_int {
    if out_text.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let text = EmbeddedTelemetry::export_to_text();
    match CString::new(text) {
        Ok(cstr) => {
            *out_text = cstr.into_raw();
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(_) => graphdb_error_code_t::GRAPHDB_NOMEM as c_int,
    }
}

/// Get a specific counter value
///
/// # Arguments
/// - `name`: Counter name (null-terminated string)
/// - `out_value`: Output parameter for counter value
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code (GRAPHDB_NOTFOUND if counter doesn't exist)
///
/// # Safety
/// - `name` must be a valid null-terminated string
/// - `out_value` must be a valid pointer to store the result
#[no_mangle]
pub unsafe extern "C" fn graphdb_telemetry_get_counter(
    name: *const c_char,
    out_value: *mut u64,
) -> c_int {
    if name.is_null() || out_value.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return graphdb_error_code_t::GRAPHDB_ERROR as c_int,
    };

    match EmbeddedTelemetry::get_counter(name_str) {
        Some(value) => {
            *out_value = value;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
    }
}

/// Get a specific gauge value
///
/// # Arguments
/// - `name`: Gauge name (null-terminated string)
/// - `out_value`: Output parameter for gauge value
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code (GRAPHDB_NOTFOUND if gauge doesn't exist)
///
/// # Safety
/// - `name` must be a valid null-terminated string
/// - `out_value` must be a valid pointer to store the result
#[no_mangle]
pub unsafe extern "C" fn graphdb_telemetry_get_gauge(
    name: *const c_char,
    out_value: *mut f64,
) -> c_int {
    if name.is_null() || out_value.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return graphdb_error_code_t::GRAPHDB_ERROR as c_int,
    };

    match EmbeddedTelemetry::get_gauge(name_str) {
        Some(value) => {
            *out_value = value;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
    }
}

/// Get global query count
///
/// # Returns
/// - Total query count
#[no_mangle]
pub extern "C" fn graphdb_global_metrics_query_count() -> u64 {
    GlobalMetrics::global().get_query_count()
}

/// Get global storage stats
///
/// # Arguments
/// - `out_used_bytes`: Output parameter for used storage bytes
/// - `out_total_bytes`: Output parameter for total storage bytes
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code if pointers are null
///
/// # Safety
/// - `out_used_bytes` and `out_total_bytes` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn graphdb_global_metrics_storage_stats(
    out_used_bytes: *mut u64,
    out_total_bytes: *mut u64,
) -> c_int {
    if out_used_bytes.is_null() || out_total_bytes.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // Note: Storage stats require access to Storage instance
    // For now, return 0 as placeholder
    *out_used_bytes = 0;
    *out_total_bytes = 0;
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// Get recent slow queries count
///
/// # Arguments
/// - `limit`: Maximum number of slow queries to retrieve
///
/// # Returns
/// - Number of slow queries available
#[no_mangle]
pub extern "C" fn graphdb_stats_manager_slow_query_count(_limit: usize) -> usize {
    // Note: This would require access to StatsManager instance
    // For now, return 0 as placeholder
    0
}

/// Get recent errors count
///
/// # Arguments
/// - `limit`: Maximum number of errors to retrieve
///
/// # Returns
/// - Number of errors available
#[no_mangle]
pub extern "C" fn graphdb_stats_manager_error_count(_limit: usize) -> usize {
    // Note: This would require access to StatsManager instance
    // For now, return 0 as placeholder
    0
}

/// Check if telemetry is initialized
///
/// # Returns
/// - 1 if initialized
/// - 0 if not initialized
#[no_mangle]
pub extern "C" fn graphdb_telemetry_is_initialized() -> c_int {
    if EmbeddedTelemetry::is_initialized() {
        1
    } else {
        0
    }
}

/// Get filtered metrics in JSON format
///
/// # Arguments
/// - `prefix`: Metric name prefix filter (null-terminated string)
/// - `out_json`: Output parameter for JSON string (must be freed with graphdb_free_string)
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `prefix` must be a valid null-terminated string
/// - `out_json` must be a valid pointer to store the result
/// - The returned string must be freed using `graphdb_free_string`
#[no_mangle]
pub unsafe extern "C" fn graphdb_telemetry_get_filtered_json(
    prefix: *const c_char,
    out_json: *mut *mut c_char,
) -> c_int {
    if prefix.is_null() || out_json.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let prefix_str = match CStr::from_ptr(prefix).to_str() {
        Ok(s) => s,
        Err(_) => return graphdb_error_code_t::GRAPHDB_ERROR as c_int,
    };

    let snapshot = EmbeddedTelemetry::get_metrics_filtered(prefix_str);
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => match CString::new(json) {
            Ok(cstr) => {
                *out_json = cstr.into_raw();
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(_) => graphdb_error_code_t::GRAPHDB_NOMEM as c_int,
        },
        Err(_) => graphdb_error_code_t::GRAPHDB_ERROR as c_int,
    }
}
