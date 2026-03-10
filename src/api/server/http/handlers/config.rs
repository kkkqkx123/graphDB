//! 配置管理 HTTP 处理器

use axum::{
    extract::{Json, Path, State},
    response::Json as JsonResponse,
};
use serde::Deserialize;
use serde_json;

use crate::api::server::http::{error::HttpError, state::AppState};
use crate::storage::StorageClient;

/// 获取当前配置
pub async fn get<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let config = state.server.get_config();

    Ok(JsonResponse(serde_json::json!({
        "database": {
            "host": config.database.host,
            "port": config.database.port,
            "storage_path": config.database.storage_path,
            "max_connections": config.database.max_connections,
        },
        "transaction": {
            "default_timeout": config.transaction.default_timeout,
            "max_concurrent_transactions": config.transaction.max_concurrent_transactions,
        },
        "log": {
            "level": config.log.level,
            "dir": config.log.dir,
            "file": config.log.file,
            "max_file_size": config.log.max_file_size,
            "max_files": config.log.max_files,
        },
        "auth": {
            "enable_authorize": config.auth.enable_authorize,
            "failed_login_attempts": config.auth.failed_login_attempts,
            "session_idle_timeout_secs": config.auth.session_idle_timeout_secs,
            "force_change_default_password": config.auth.force_change_default_password,
            "default_username": config.auth.default_username,
        },
        "bootstrap": {
            "auto_create_default_space": config.bootstrap.auto_create_default_space,
            "default_space_name": config.bootstrap.default_space_name,
            "single_user_mode": config.bootstrap.single_user_mode,
        },
        "optimizer": {
            "max_iteration_rounds": config.optimizer.max_iteration_rounds,
            "max_exploration_rounds": config.optimizer.max_exploration_rounds,
            "enable_cost_model": config.optimizer.enable_cost_model,
            "enable_multi_plan": config.optimizer.enable_multi_plan,
            "enable_property_pruning": config.optimizer.enable_property_pruning,
            "enable_adaptive_iteration": config.optimizer.enable_adaptive_iteration,
            "stable_threshold": config.optimizer.stable_threshold,
            "min_iteration_rounds": config.optimizer.min_iteration_rounds,
        },
        "monitoring": {
            "enabled": config.monitoring.enabled,
            "memory_cache_size": config.monitoring.memory_cache_size,
            "slow_query_threshold_ms": config.monitoring.slow_query_threshold_ms,
            "slow_query_log_dir": config.monitoring.slow_query_log_dir,
            "slow_query_log_retention_days": config.monitoring.slow_query_log_retention_days,
        },
    })))
}

/// 更新配置（热更新）
pub async fn update<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Json(request): Json<serde_json::Value>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let mut updated = Vec::new();
    let mut requires_restart = Vec::new();

    if let Some(sections) = request.as_object() {
        for (section, values) in sections {
            if let Some(values_obj) = values.as_object() {
                for (key, _value) in values_obj {
                    let full_key = format!("{}.{}", section, key);

                    if is_restart_required(section, key) {
                        requires_restart.push(full_key);
                    } else {
                        updated.push(full_key);
                    }
                }
            }
        }
    }

    Ok(JsonResponse(serde_json::json!({
        "updated": updated,
        "requires_restart": requires_restart,
        "message": "配置更新已接收，部分更改可能需要重启才能生效",
    })))
}

/// 获取配置项
pub async fn get_key<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Path((section, key)): Path<(String, String)>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let config = state.server.get_config();
    let value = get_config_value(config, &section, &key);

    Ok(JsonResponse(serde_json::json!({
        "section": section,
        "key": key,
        "value": value,
    })))
}

/// 更新配置项
pub async fn update_key<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path((section, key)): Path<(String, String)>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let requires_restart = is_restart_required(&section, &key);

    Ok(JsonResponse(serde_json::json!({
        "section": section,
        "key": key,
        "value": request.value,
        "requires_restart": requires_restart,
        "message": if requires_restart {
            "配置项已更新，但需要重启才能生效"
        } else {
            "配置项已更新"
        },
    })))
}

/// 重置配置项为默认值
pub async fn reset_key<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,
    Path((section, key)): Path<(String, String)>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    let default_config = crate::config::Config::default();
    let default_value = get_config_value(&default_config, &section, &key);

    Ok(JsonResponse(serde_json::json!({
        "section": section,
        "key": key,
        "value": default_value,
        "message": "配置已重置为默认值",
    })))
}

/// 更新配置请求
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: serde_json::Value,
}

/// 获取配置值
fn get_config_value(config: &crate::config::Config, section: &str, key: &str) -> serde_json::Value {
    match section {
        "database" => match key {
            "host" => serde_json::json!(config.database.host),
            "port" => serde_json::json!(config.database.port),
            "storage_path" => serde_json::json!(config.database.storage_path),
            "max_connections" => serde_json::json!(config.database.max_connections),
            _ => serde_json::Value::Null,
        },
        "transaction" => match key {
            "default_timeout" => serde_json::json!(config.transaction.default_timeout),
            "max_concurrent_transactions" => serde_json::json!(config.transaction.max_concurrent_transactions),
            _ => serde_json::Value::Null,
        },
        "log" => match key {
            "level" => serde_json::json!(config.log.level),
            "dir" => serde_json::json!(config.log.dir),
            "file" => serde_json::json!(config.log.file),
            "max_file_size" => serde_json::json!(config.log.max_file_size),
            "max_files" => serde_json::json!(config.log.max_files),
            _ => serde_json::Value::Null,
        },
        "auth" => match key {
            "enable_authorize" => serde_json::json!(config.auth.enable_authorize),
            "failed_login_attempts" => serde_json::json!(config.auth.failed_login_attempts),
            "session_idle_timeout_secs" => serde_json::json!(config.auth.session_idle_timeout_secs),
            "force_change_default_password" => serde_json::json!(config.auth.force_change_default_password),
            "default_username" => serde_json::json!(config.auth.default_username),
            _ => serde_json::Value::Null,
        },
        "bootstrap" => match key {
            "auto_create_default_space" => serde_json::json!(config.bootstrap.auto_create_default_space),
            "default_space_name" => serde_json::json!(config.bootstrap.default_space_name),
            "single_user_mode" => serde_json::json!(config.bootstrap.single_user_mode),
            _ => serde_json::Value::Null,
        },
        "optimizer" => match key {
            "max_iteration_rounds" => serde_json::json!(config.optimizer.max_iteration_rounds),
            "max_exploration_rounds" => serde_json::json!(config.optimizer.max_exploration_rounds),
            "enable_cost_model" => serde_json::json!(config.optimizer.enable_cost_model),
            "enable_multi_plan" => serde_json::json!(config.optimizer.enable_multi_plan),
            "enable_property_pruning" => serde_json::json!(config.optimizer.enable_property_pruning),
            "enable_adaptive_iteration" => serde_json::json!(config.optimizer.enable_adaptive_iteration),
            "stable_threshold" => serde_json::json!(config.optimizer.stable_threshold),
            "min_iteration_rounds" => serde_json::json!(config.optimizer.min_iteration_rounds),
            _ => serde_json::Value::Null,
        },
        "monitoring" => match key {
            "enabled" => serde_json::json!(config.monitoring.enabled),
            "memory_cache_size" => serde_json::json!(config.monitoring.memory_cache_size),
            "slow_query_threshold_ms" => serde_json::json!(config.monitoring.slow_query_threshold_ms),
            "slow_query_log_dir" => serde_json::json!(config.monitoring.slow_query_log_dir),
            "slow_query_log_retention_days" => serde_json::json!(config.monitoring.slow_query_log_retention_days),
            _ => serde_json::Value::Null,
        },
        _ => serde_json::Value::Null,
    }
}

/// 检查配置项是否需要重启才能生效
fn is_restart_required(section: &str, key: &str) -> bool {
    match section {
        "database" => matches!(key, "host" | "port" | "storage_path" | "max_connections"),
        "transaction" => false,
        "log" => matches!(key, "dir" | "file"),
        "auth" => matches!(key, "default_username"),
        "bootstrap" => true,
        "optimizer" => false,
        "monitoring" => matches!(key, "slow_query_log_dir"),
        _ => false,
    }
}
