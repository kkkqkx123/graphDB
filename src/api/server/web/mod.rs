//! Web Management Module
//!
//! Provides Web management interface API for GraphDB frontend:
//! - Query history management
//! - Query favorites management
//! - Extended Schema management
//! - Data browsing
//! - Graph data queries

pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod storage;

use axum::Router;
use std::sync::Arc;

use crate::api::server::http::AppState;
use crate::storage::StorageClient;

use self::storage::SqliteStorage;

/// Web module state
#[derive(Clone)]
pub struct WebState<S: StorageClient + 'static> {
    /// Core application state
    pub app_state: AppState<S>,
    /// Metadata storage
    pub metadata_storage: Arc<SqliteStorage>,
}

impl<S: StorageClient + 'static> WebState<S> {
    pub async fn new(app_state: AppState<S>, storage_path: &str) -> Result<Self, error::WebError> {
        let metadata_storage = Arc::new(SqliteStorage::new(storage_path).await?);

        Ok(Self {
            app_state,
            metadata_storage,
        })
    }
}

/// Create Web management router
pub fn create_router<S: StorageClient + Clone + Send + Sync + 'static>(
    web_state: WebState<S>,
) -> Router {
    Router::new()
        .nest("/v1/queries", handlers::metadata::create_router(web_state.clone()))
        .nest("/v1/schema", handlers::schema_ext::create_router(web_state.clone()))
        .nest("/v1/data", handlers::data_browser::create_router(web_state.clone()))
        .nest("/v1/graph", handlers::graph_data::create_router(web_state))
}

/// Create Web management router (no state version for compatibility)
pub fn create_router_no_state() -> Router {
    Router::new()
}
