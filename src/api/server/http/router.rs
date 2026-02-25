use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    middleware,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;

use crate::storage::StorageClient;

use super::{
    state::AppState,
    handlers::{
        health,
        query,
        auth,
        session::{create, get_session, delete_session},
    },
    middleware::{logging, error},
};

pub fn create_router<S: StorageClient + Clone + Send + Sync + 'static>(
    state: AppState<S>,
) -> Router {
    Router::new()
        .route("/health", get(health::check))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/sessions", post(create))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        .route("/query", post(query::execute))
        .route("/query/validate", post(query::validate))
        .layer(middleware::from_fn(logging::logging_middleware))
        .layer(middleware::from_fn(error::error_handling_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .with_state(state)
}
