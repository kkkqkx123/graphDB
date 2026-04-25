// Service implementation - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

// Import generated proto types
use crate::api::server::proto::inversearch_service_server::{
    InversearchService as InversearchServiceTrait, InversearchServiceServer,
};
use crate::api::server::proto::*;

// Import core library types
use crate::api::core::{Index, SearchOptions};
use crate::IndexOptions;
use crate::ServiceConfig;

// Import storage module
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::factory::StorageFactory;

#[cfg(feature = "store-cold-warm-cache")]
use crate::storage::cold_warm_cache::ColdWarmCacheManager;

// Import config
use crate::config::Config;

/// Create storage based on configuration
#[allow(clippy::needless_return)]
pub async fn create_storage_from_config(
    config: &Config,
) -> Arc<dyn StorageInterface + Send + Sync> {
    match StorageFactory::from_config(config).await {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("Failed to create storage: {}", e);
            // Fallback to cold-warm cache
            #[cfg(feature = "store-cold-warm-cache")]
            {
                use crate::storage::cold_warm_cache::ColdWarmCacheManager;
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        ColdWarmCacheManager::new().await.unwrap()
                            as Arc<dyn StorageInterface + Send + Sync>
                    })
                })
            }
            #[cfg(not(feature = "store-cold-warm-cache"))]
            panic!("No fallback storage available: {}", e);
        }
    }
}

/// Inversearch gRPC service implementation
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    #[allow(dead_code)]
    storage: Arc<dyn StorageInterface + Send + Sync>,
    config: Config,
    start_time: Instant,
}

impl Default for InversearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl InversearchService {
    /// Create a new service instance with default configuration
    pub fn new() -> Self {
        let config = Config::default();
        Self::with_config(config)
    }

    /// Create a new service instance with custom configuration
    pub async fn with_config_async(config: Config) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = create_storage_from_config(&config).await;

        Self {
            index,
            storage,
            config,
            start_time: Instant::now(),
        }
    }

    /// Create a new service instance with custom configuration (sync version)
    pub fn with_config(config: Config) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));

        #[cfg(feature = "store-cold-warm-cache")]
        let storage: Arc<dyn StorageInterface + Send + Sync> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = ColdWarmCacheManager::new().await.unwrap();
                manager as Arc<dyn StorageInterface + Send + Sync>
            })
        });
        #[cfg(not(feature = "store-cold-warm-cache"))]
        let storage: Arc<dyn StorageInterface + Send + Sync> = panic!("No storage backend enabled");

        Self {
            index,
            storage,
            config,
            start_time: Instant::now(),
        }
    }

    /// Create a new service instance with custom storage
    pub fn with_storage<S: StorageInterface + Send + Sync + 'static>(storage: S) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = Arc::new(storage);

        Self {
            index,
            storage,
            config: Config::default(),
            start_time: Instant::now(),
        }
    }

    /// Create a new service instance with custom storage and config
    pub fn with_storage_and_config<S: StorageInterface + Send + Sync + 'static>(
        storage: S,
        config: Config,
    ) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = Arc::new(storage);

        Self {
            index,
            storage,
            config,
            start_time: Instant::now(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
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

    async fn batch_operation(
        &self,
        request: Request<BatchOperationRequest>,
    ) -> Result<Response<BatchOperationResponse>, Status> {
        let req = request.into_inner();
        let mut index = self.index.write().await;

        let mut success_count = 0u32;
        let mut failed_count = 0u32;
        let mut errors = Vec::new();

        for op in req.operations {
            match op.operation_type {
                // Add operation
                1 => {
                    if let Some(doc) = op.document {
                        match index.add(doc.id, &doc.content, false) {
                            Ok(_) => success_count += 1,
                            Err(e) => {
                                failed_count += 1;
                                errors.push(format!("Add {} failed: {}", doc.id, e));
                            }
                        }
                    } else {
                        failed_count += 1;
                        errors.push("Add operation missing document".to_string());
                    }
                }
                // Remove operation
                2 => match index.remove(op.document_id, false) {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Remove {} failed: {}", op.document_id, e));
                    }
                },
                // Update operation
                3 => {
                    if let Some(doc) = op.document {
                        match index.update(doc.id, &doc.content) {
                            Ok(_) => success_count += 1,
                            Err(e) => {
                                failed_count += 1;
                                errors.push(format!("Update {} failed: {}", doc.id, e));
                            }
                        }
                    } else {
                        failed_count += 1;
                        errors.push("Update operation missing document".to_string());
                    }
                }
                _ => {
                    failed_count += 1;
                    errors.push(format!("Unknown operation type: {}", op.operation_type));
                }
            }
        }

        Ok(Response::new(BatchOperationResponse {
            success_count,
            failed_count,
            errors,
        }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();

        let index = self.index.read().await;

        // Build search options
        let search_opts = SearchOptions {
            query: Some(req.query),
            limit: Some(req.limit as usize),
            offset: Some(req.offset as usize),
            context: Some(req.context),
            suggest: Some(req.suggest),
            resolve: Some(req.resolve),
            enrich: Some(req.enrich),
            cache: Some(req.cache),
            ..Default::default()
        };

        // Perform search
        let result = index.search(&search_opts);

        match result {
            Ok(search_result) => {
                let results: Vec<u64> = search_result.results.to_vec();
                Ok(Response::new(SearchResponse {
                    results,
                    total: search_result.total as u32,
                    error: String::new(),
                    highlights: vec![],
                }))
            }
            Err(e) => Ok(Response::new(SearchResponse {
                results: vec![],
                total: 0,
                error: e.to_string(),
                highlights: vec![],
            })),
        }
    }

    async fn suggest(
        &self,
        request: Request<SuggestRequest>,
    ) -> Result<Response<SuggestResponse>, Status> {
        let req = request.into_inner();

        let index = self.index.read().await;

        // Build search options for suggestion
        let search_opts = SearchOptions {
            query: Some(req.query.clone()),
            limit: Some(req.limit as usize),
            suggest: Some(true),
            ..Default::default()
        };

        // Perform search
        let result = index.search(&search_opts);

        match result {
            Ok(search_result) => {
                // Convert results to suggestions (using result IDs as suggestions)
                let suggestions: Vec<String> = search_result
                    .results
                    .iter()
                    .map(|id| id.to_string())
                    .collect();
                let total = suggestions.len() as u32;

                Ok(Response::new(SuggestResponse {
                    suggestions,
                    total,
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SuggestResponse {
                suggestions: vec![],
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
        let document_count = index.document_count();

        // Calculate index size (primary index + number of entries in context index)
        let index_size = index.map.index.len() + index.ctx.index.len();

        // Cache size (if cached)
        let cache_size = index.cache.as_ref().map(|c| c.len()).unwrap_or(0);

        Ok(Response::new(GetStatsResponse {
            document_count: document_count as u64,
            index_size: index_size as u64,
            cache_size: cache_size as u64,
            error: String::new(),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let index = self.index.read().await;
        let document_count = index.document_count();

        // Multi-dimensional Health Screening
        let is_healthy = !index.map.index.is_empty()
            || !index.ctx.index.is_empty() && document_count < u32::MAX as usize;

        // Calculating Runtime
        let uptime = self.start_time.elapsed().as_secs();

        Ok(Response::new(HealthCheckResponse {
            healthy: is_healthy,
            document_count: document_count as u64,
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

/// Run the gRPC server
pub async fn run_server(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;
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
    let addr = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;
    let service = InversearchService::with_storage(storage);

    tracing::info!("Inversearch service listening on {}", addr);

    Server::builder()
        .add_service(InversearchServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Run the gRPC server with custom service instance
pub async fn run_server_with_service(
    config: ServiceConfig,
    service: InversearchService,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;

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
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
    }
}
