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
use crate::storage::common::r#trait::StorageInterface;

#[cfg(feature = "store-memory")]
use crate::storage::memory::MemoryStorage;

#[cfg(feature = "store-file")]
use crate::storage::file::FileStorage;

#[cfg(feature = "store-redis")]
use crate::storage::redis::{RedisStorage, RedisStorageConfig};

#[cfg(feature = "store-wal")]
use crate::storage::wal_storage::WALStorage;

#[cfg(feature = "store-cached")]
use crate::storage::cached::CachedStorage;

// Import config
use crate::config::Config;
#[cfg(any(feature = "store-memory", feature = "store-file", feature = "store-redis", feature = "store-wal"))]
use crate::config::StorageBackend;

#[cfg(feature = "store-wal")]
use crate::storage::wal::WALConfig;

// Import index module's IndexOptions
use crate::index::IndexOptions;

/// Create storage based on configuration
pub async fn create_storage_from_config(config: &Config) -> Arc<RwLock<dyn StorageInterface + Send + Sync>> {
    if !config.storage.enabled {
        #[cfg(feature = "store-cached")]
        return Arc::new(RwLock::new(CachedStorage::new()));
        #[cfg(not(feature = "store-cached"))]
        panic!("No storage backend enabled");
    }

    match &config.storage.backend {
        #[cfg(feature = "store-memory")]
        StorageBackend::Memory => {
            Arc::new(RwLock::new(MemoryStorage::new()))
        }
        #[cfg(feature = "store-file")]
        StorageBackend::File => {
            let file_config = config.storage.file.as_ref()
                .map(|c| c.base_path.clone())
                .unwrap_or_else(|| "./data".to_string());
            Arc::new(RwLock::new(FileStorage::new(file_config)))
        }
        #[cfg(feature = "store-redis")]
        StorageBackend::Redis => {
            let redis_config = config.storage.redis.as_ref()
                .map(|c| RedisStorageConfig {
                    url: c.url.clone(),
                    pool_size: c.pool_size,
                    ..Default::default()
                })
                .unwrap_or_default();
            match RedisStorage::new(redis_config).await {
                Ok(storage) => Arc::new(RwLock::new(storage)),
                Err(e) => {
                    eprintln!("Failed to connect to Redis: {}, falling back to cached storage", e);
                    #[cfg(feature = "store-cached")]
                    return Arc::new(RwLock::new(CachedStorage::new()));
                    #[cfg(not(feature = "store-cached"))]
                    panic!("No fallback storage available");
                }
            }
        }
        #[cfg(feature = "store-wal")]
        StorageBackend::Wal => {
            let wal_config = config.storage.wal.as_ref()
                .map(|c| WALConfig {
                    base_path: std::path::PathBuf::from(&c.base_path),
                    max_wal_size: c.max_wal_size,
                    compression: c.compression,
                    snapshot_interval: c.snapshot_interval,
                    ..Default::default()
                })
                .unwrap_or_default();
            match WALStorage::new(wal_config).await {
                Ok(storage) => Arc::new(RwLock::new(storage)),
                Err(e) => {
                    eprintln!("Failed to initialize WAL storage: {}, falling back to cached storage", e);
                    #[cfg(feature = "store-cached")]
                    return Arc::new(RwLock::new(CachedStorage::new()));
                    #[cfg(not(feature = "store-cached"))]
                    panic!("No fallback storage available");
                }
            }
        }
        #[cfg(not(any(
            feature = "store-memory",
            feature = "store-file",
            feature = "store-redis",
            feature = "store-wal"
        )))]
        _ => {
            // 默认使用缓存存储
            #[cfg(feature = "store-cached")]
            {
                Arc::new(RwLock::new(CachedStorage::new()))
            }
            #[cfg(not(feature = "store-cached"))]
            panic!("No storage backend enabled");
        }
    }
}

/// Inversearch gRPC service implementation
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    #[allow(dead_code)]
    storage: Arc<RwLock<dyn StorageInterface + Send + Sync>>,
    config: Config,
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
        }
    }

    /// Create a new service instance with custom configuration (sync version)
    pub fn with_config(config: Config) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        
        #[cfg(feature = "store-cached")]
        let storage: Arc<RwLock<dyn StorageInterface + Send + Sync>> =
            Arc::new(RwLock::new(CachedStorage::new()));
        #[cfg(not(feature = "store-cached"))]
        let storage: Arc<RwLock<dyn StorageInterface + Send + Sync>> =
            panic!("No storage backend enabled");

        Self {
            index,
            storage,
            config,
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
            config: Config::default(),
        }
    }

    /// Create a new service instance with custom storage and config
    pub fn with_storage_and_config<S: StorageInterface + Send + Sync + 'static>(storage: S, config: Config) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = Arc::new(RwLock::new(storage));

        Self {
            index,
            storage,
            config,
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
        let _service = InversearchService::new();
        // Service should be created successfully
    }
}