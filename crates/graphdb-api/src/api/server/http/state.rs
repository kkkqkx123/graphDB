use crate::api::server::HttpServer;
use crate::storage::StorageClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState<S: StorageClient + Clone + 'static> {
    pub server: Arc<HttpServer<S>>,
}

impl<S: StorageClient + Clone + 'static> AppState<S> {
    pub fn new(server: Arc<HttpServer<S>>) -> Self {
        Self { server }
    }
}
