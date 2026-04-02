// Service implementation - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

// Import generated proto types
use crate::proto::inversearch_service_server::{
    InversearchService as InversearchServiceTrait,
    InversearchServiceServer,
};
use crate::proto::*;

// Import core library types
use crate::{
    Index, SearchOptions,
};

// Import storage module
use crate::storage::{StorageInterface, MemoryStorage};

// Import index module's IndexOptions
use crate::index::IndexOptions;

/// Inversearch gRPC service implementation
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    #[allow(dead_code)]
    storage: Arc<RwLock<dyn StorageInterface + Send + Sync>>,
}

impl Default for InversearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl InversearchService {
    /// Create a new service instance
    pub fn new() -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage: Arc<RwLock<dyn StorageInterface + Send + Sync>> =
            Arc::new(RwLock::new(MemoryStorage::new()));

        Self {
            index,
            storage,
        }
    }

    /// Create a new service instance with custom storage
    pub fn with_storage<S: StorageInterface + Send + Sync + 'static>(storage: S) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = Arc::new(RwLock::new(storage));

        Self {
            index,
            storage,
        }
    }
}

#[tonic::async_trait]
impl InversearchServiceTrait for InversearchService {
    async fn add_document(
        &self,
        request: Request<AddDocumentRequest>,
    ) -> Result<Response<AddDocumentResponse>, Status> {
        let req = request.into_inner();

        let mut index = self.index.write().await;
        match index.add(req.id, &req.content, false) {
            Ok(_) => Ok(Response::new(AddDocumentResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(AddDocumentResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn update_document(
        &self,
        request: Request<UpdateDocumentRequest>,
    ) -> Result<Response<UpdateDocumentResponse>, Status> {
        let req = request.into_inner();

        let mut index = self.index.write().await;
        match index.update(req.id, &req.content) {
            Ok(_) => Ok(Response::new(UpdateDocumentResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UpdateDocumentResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn remove_document(
        &self,
        request: Request<RemoveDocumentRequest>,
    ) -> Result<Response<RemoveDocumentResponse>, Status> {
        let req = request.into_inner();

        let mut index = self.index.write().await;
        match index.remove(req.id, false) {
            Ok(_) => Ok(Response::new(RemoveDocumentResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RemoveDocumentResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();

        let index = self.index.read().await;

        // Build search options
        let mut search_opts = SearchOptions::default();
        search_opts.query = Some(req.query);
        search_opts.limit = Some(req.limit as usize);
        search_opts.offset = Some(req.offset as usize);
        search_opts.context = Some(req.context);
        search_opts.suggest = Some(req.suggest);
        search_opts.resolve = Some(req.resolve);
        search_opts.enrich = Some(req.enrich);
        search_opts.cache = Some(req.cache);

        // Perform search
        let result = index.search(&search_opts);

        match result {
            Ok(search_result) => {
                let results: Vec<u64> = search_result.results.iter().map(|&id| id as u64).collect();
                Ok(Response::new(SearchResponse {
                    results,
                    total: search_result.total as u32,
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SearchResponse {
                results: vec![],
                total: 0,
                error: e.to_string(),
            })),
        }
    }

    async fn clear_index(
        &self,
        _request: Request<ClearIndexRequest>,
    ) -> Result<Response<ClearIndexResponse>, Status> {
        let mut index = self.index.write().await;
        index.clear();

        Ok(Response::new(ClearIndexResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn get_stats(
        &self,
        _request: Request<GetStatsRequest>,
    ) -> Result<Response<GetStatsResponse>, Status> {
        let index = self.index.read().await;
        // Use document_count() to get the actual document count
        let document_count = index.document_count();

        Ok(Response::new(GetStatsResponse {
            document_count: document_count as u64,
            index_size: 0, // TODO: implement actual index size calculation
            cache_size: 0, // TODO: implement cache size tracking
            error: String::new(),
        }))
    }
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
        }
    }
}

/// Run the gRPC server
pub async fn run_server(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;
    let service = InversearchService::new();

    tracing::info!("Inversearch service listening on {}", addr);

    Server::builder()
        .add_service(InversearchServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Run the gRPC server with custom storage
pub async fn run_server_with_storage<S: StorageInterface + Send + Sync + 'static>(
    config: ServiceConfig,
    storage: S,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;
    let service = InversearchService::with_storage(storage);

    tracing::info!("Inversearch service listening on {}", addr);

    Server::builder()
        .add_service(InversearchServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 50051);
    }

    #[test]
    fn test_service_creation() {
        let service = InversearchService::new();
        // Service should be created successfully
    }
}