use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    middleware,
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    timeout::TimeoutLayer,
    limit::RequestBodyLimitLayer,
};
use std::time::Duration;

use crate::storage::StorageClient;

use super::{
    state::AppState,
    handlers::{
        health,
        query,
        auth::{login, logout},
        transaction,
        schema,
        session::{create, get_session, delete_session},
    },
    middleware::{logging, error, auth::auth_middleware},
};

/// 创建路由器
///
/// 路由结构：
/// - /v1/health - 健康检查（公开）
/// - /v1/auth/* - 认证相关（公开）
/// - /v1/sessions/* - 会话管理（需要认证）
/// - /v1/query - 查询执行（需要认证）
/// - /v1/transactions/* - 事务管理（需要认证）
/// - /v1/schema/* - Schema 管理（需要认证）
pub fn create_router<S: StorageClient + Clone + Send + Sync + 'static>(
    state: AppState<S>,
) -> Router {
    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/health", get(health::check))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/sessions", post(create))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        .route("/query", post(query::execute))
        .route("/query/validate", post(query::validate))
        .route("/transactions", post(transaction::begin))
        .route("/transactions/:id/commit", post(transaction::commit))
        .route("/transactions/:id/rollback", post(transaction::rollback))
        .route("/schema/spaces", post(schema::create_space).get(schema::list_spaces))
        .route("/schema/spaces/:name", get(schema::get_space).delete(schema::drop_space))
        .route("/schema/spaces/:name/tags", post(schema::create_tag).get(schema::list_tags))
        .route("/schema/spaces/:name/edge-types", post(schema::create_edge_type).get(schema::list_edge_types))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // 合并所有路由，添加版本前缀
    Router::new()
        .nest("/v1", public_routes.merge(protected_routes))
        .layer(middleware::from_fn(logging::logging_middleware))
        .layer(middleware::from_fn(error::error_handling_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(create_cors_layer())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 10)) // 10MB 请求体限制
        .with_state(state)
}

/// 创建 CORS 配置层
///
/// 开发环境允许所有来源，生产环境应该配置具体来源
fn create_cors_layer() -> CorsLayer {
    // 注意：生产环境应该收紧这个配置
    // 例如：只允许特定域名访问
    CorsLayer::new()
        .allow_origin(Any) // 允许所有来源，生产环境应该改为具体域名
        .allow_methods(Any)
        .allow_headers(Any)
}
