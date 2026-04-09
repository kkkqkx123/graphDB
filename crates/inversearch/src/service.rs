// Service implementation - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

// Import generated proto types
use crate::proto::inversearch_service_server::{
    InversearchService as InversearchServiceTrait, InversearchServiceServer,
};
use crate::proto::*;

// Import core library types
use crate::{Index, SearchOptions};

// Import storage module
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::{StorageManager, StorageManagerBuilder};

#[cfg(feature = "store-cold-warm-cache")]
use crate::storage::cold_warm_cache::ColdWarmCacheManager;

#[cfg(feature = "store-file")]
use crate::storage::file::FileStorage;

#[cfg(feature = "store-redis")]
use crate::storage::redis::{RedisStorage, RedisStorageConfig};

#[cfg(feature = "store-wal")]
use crate::storage::wal::WALStorage;

// Import config
use crate::api::server::config::ServiceConfig;
use crate::config::Config;
use crate::config::StorageBackend;

#[cfg(feature = "store-wal")]
use crate::storage::wal::WALConfig;

// Import index module's IndexOptions
use crate::index::IndexOptions;

/// Create storage based on configuration
#[allow(clippy::needless_return)]
pub async fn create_storage_from_config(
    config: &Config,
) -> Arc<dyn StorageInterface + Send + Sync> {
    if !config.storage.enabled {
        // 存储未启用时，默认使用冷热缓存
        #[cfg(feature = "store-cold-warm-cache")]
        {
            return tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let manager = ColdWarmCacheManager::new().await.unwrap();
                    manager as Arc<dyn StorageInterface + Send + Sync>
                })
            });
        }
        #[cfg(not(feature = "store-cold-warm-cache"))]
        panic!("No storage backend enabled");
    }

    match &config.storage.backend {
        #[cfg(feature = "store-file")]
        StorageBackend::File => {
            let file_config = config
                .storage
                .file
                .as_ref()
                .map(|c| c.base_path.clone())
                .unwrap_or_else(|| "./data".to_string());
            Arc::new(FileStorage::new(file_config))
        }
        #[cfg(feature = "store-redis")]
        StorageBackend::Redis => {
            let redis_config = config
                .storage
                .redis
                .as_ref()
                .map(|c| RedisStorageConfig {
                    url: c.url.clone(),
                    pool_size: c.pool_size,
                    ..Default::default()
                })
                .unwrap_or_default();
            match RedisStorage::new(redis_config).await {
                Ok(storage) => Arc::new(storage),
                Err(e) => {
                    eprintln!(
                        "Failed to connect to Redis: {}. Storage will be unavailable.",
                        e
                    );
                    #[cfg(feature = "store-cold-warm-cache")]
                    {
                        eprintln!("Falling back to cold-warm cache...");
                        return tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                let manager = ColdWarmCacheManager::new().await.unwrap();
                                manager as Arc<dyn StorageInterface + Send + Sync>
                            })
                        });
                    }
                    #[cfg(not(feature = "store-cold-warm-cache"))]
                    panic!("No fallback storage available");
                }
            }
        }
        #[cfg(feature = "store-wal")]
        StorageBackend::Wal => {
            let wal_config = config
                .storage
                .wal
                .as_ref()
                .map(|c| WALConfig {
                    base_path: std::path::PathBuf::from(&c.base_path),
                    max_wal_size: c.max_wal_size,
                    compression: c.compression,
                    snapshot_interval: c.snapshot_interval,
                    ..Default::default()
                })
                .unwrap_or_default();
            match WALStorage::new(wal_config).await {
                Ok(storage) => Arc::new(storage),
                Err(e) => {
                    eprintln!("Failed to create WAL storage: {}, falling back to cold-warm cache", e);
                    #[cfg(feature = "store-cold-warm-cache")]
                    {
                        return tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                let manager = ColdWarmCacheManager::new().await.unwrap();
                                manager as Arc<dyn StorageInterface + Send + Sync>
                            })
                        });
                    }
                    #[cfg(not(feature = "store-cold-warm-cache"))]
                    panic!("No fallback storage available");
                }
            }
        }
        StorageBackend::ColdWarmCache => {
            // ColdWarmCacheManager 已经是 Arc<Self> 且实现了 StorageInterface
            // 直接返回即可，不需要额外的 RwLock 包装
            let manager = ColdWarmCacheManager::new().await.unwrap();
            // 将 Arc<ColdWarmCacheManager> 转换为 Arc<dyn StorageInterface + Send + Sync>
            manager as Arc<dyn StorageInterface + Send + Sync>
        }
    }
}

/// Inversearch gRPC service implementation
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    storage: StorageManager,
    config: Config,
    /// 是否启用存储同步
    storage_sync_enabled: bool,
    start_time: Instant,
}

impl Default for InversearchService {
    fn default() -> Self {
        // 创建同步版本，用于 Default trait
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        
        // 使用阻塞运行时创建存储
        let storage = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                StorageManagerBuilder::build_default().await
                    .expect("Failed to create storage")
            })
        });

        Self {
            index,
            storage,
            config: Config::default(),
            storage_sync_enabled: true,
            start_time: Instant::now(),
        }
    }
}

impl InversearchService {
    /// Create a new service instance with default configuration
    pub async fn new() -> Self {
        let config = Config::default();
        Self::with_config_async(config).await
    }

    /// Create a new service instance with custom configuration
    pub async fn with_config_async(config: Config) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage = StorageManagerBuilder::build_default().await
            .expect("Failed to create storage");
        
        // 尝试从存储恢复索引数据
        let storage_sync_enabled = if config.storage.enabled {
            match storage.mount(&*index.read().await).await {
                Ok(_) => {
                    tracing::info!("Storage mounted successfully");
                    true
                }
                Err(e) => {
                    tracing::warn!("Failed to mount storage: {}, continuing without persistence", e);
                    false
                }
            }
        } else {
            false
        };

        Self {
            index,
            storage,
            config,
            storage_sync_enabled,
            start_time: Instant::now(),
        }
    }

    /// Create a new service instance with custom storage
    pub fn with_storage(storage: StorageManager) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));

        Self {
            index,
            storage,
            config: Config::default(),
            storage_sync_enabled: true,
            start_time: Instant::now(),
        }
    }

    /// Create a new service instance with custom storage and config
    pub fn with_storage_and_config(
        storage: StorageManager,
        config: Config,
    ) -> Self {
        let index = Index::new(IndexOptions::default()).expect("Failed to create index");
        let index = Arc::new(RwLock::new(index));
        let storage_sync_enabled = config.storage.enabled;

        Self {
            index,
            storage,
            config,
            storage_sync_enabled,
            start_time: Instant::now(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// 同步索引到存储
    async fn sync_to_storage(&self, replace: bool, append: bool) -> Result<(), crate::error::InversearchError> {
        if !self.storage_sync_enabled {
            return Ok(());
        }

        let index = self.index.read().await;
        match self.storage.commit(&index, replace, append).await {
            Ok(_) => {
                tracing::debug!("Index synced to storage successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to sync index to storage: {}", e);
                Err(e)
            }
        }
    }

    /// 从存储同步索引
    #[allow(dead_code)]
    async fn sync_from_storage(&self) -> Result<(), crate::error::InversearchError> {
        if !self.storage_sync_enabled {
            return Ok(());
        }

        let index = self.index.read().await;
        match self.storage.mount(&index).await {
            Ok(_) => {
                tracing::debug!("Index synced from storage successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to sync index from storage: {}", e);
                Err(e)
            }
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

        // 先修改索引
        let add_result = {
            let mut index = self.index.write().await;
            index.add(req.id, &req.content, false)
        };

        // 如果索引修改成功，同步到存储
        match add_result {
            Ok(_) => {
                if self.storage_sync_enabled {
                    if let Err(e) = self.sync_to_storage(false, true).await {
                        tracing::warn!("Failed to sync to storage after add: {}", e);
                        // 继续返回成功，因为索引操作已成功
                    }
                }
                Ok(Response::new(AddDocumentResponse {
                    success: true,
                    error: String::new(),
                }))
            }
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

        // 先修改索引
        let update_result = {
            let mut index = self.index.write().await;
            index.update(req.id, &req.content)
        };

        // 如果索引修改成功，同步到存储
        match update_result {
            Ok(_) => {
                if self.storage_sync_enabled {
                    if let Err(e) = self.sync_to_storage(false, true).await {
                        tracing::warn!("Failed to sync to storage after update: {}", e);
                    }
                }
                Ok(Response::new(UpdateDocumentResponse {
                    success: true,
                    error: String::new(),
                }))
            }
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

        // 先修改索引
        let remove_result = {
            let mut index = self.index.write().await;
            index.remove(req.id, false)
        };

        // 如果索引修改成功，同步到存储
        match remove_result {
            Ok(_) => {
                if self.storage_sync_enabled {
                    // 从存储中删除文档
                    if let Err(e) = self.storage.remove_documents(&[req.id]).await {
                        tracing::warn!("Failed to remove document from storage: {}", e);
                    }
                    // 同步索引状态
                    if let Err(e) = self.sync_to_storage(false, true).await {
                        tracing::warn!("Failed to sync to storage after remove: {}", e);
                    }
                }
                Ok(Response::new(RemoveDocumentResponse {
                    success: true,
                    error: String::new(),
                }))
            }
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
        {
            let mut index = self.index.write().await;
            index.clear();
        }

        // 同步清空存储
        if self.storage_sync_enabled {
            if let Err(e) = self.storage.clear().await {
                tracing::warn!("Failed to clear storage: {}", e);
            }
        }

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
        
        // 计算索引大小（主索引 + 上下文索引的条目数）
        let index_size = index.map.index.len() + index.ctx.index.len();
        
        // 缓存大小（如果有缓存）
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
        
        // 多维度健康检查
        let is_healthy = !index.map.index.is_empty() || !index.ctx.index.is_empty()
            && document_count < u32::MAX as usize;
        
        // 计算运行时间
        let uptime = self.start_time.elapsed().as_secs();

        Ok(Response::new(HealthCheckResponse {
            healthy: is_healthy,
            document_count: document_count as u64,
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
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
                2 => {
                    match index.remove(op.document_id, false) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            failed_count += 1;
                            errors.push(format!("Remove {} failed: {}", op.document_id, e));
                        }
                    }
                }
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
}

/// Run the gRPC server
pub async fn run_server(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()?;
    let service = InversearchService::new().await;

    tracing::info!("Inversearch service listening on {}", addr);

    Server::builder()
        .add_service(InversearchServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Run the gRPC server with custom storage
pub async fn run_server_with_storage(
    config: ServiceConfig,
    storage: StorageManager,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
    }

    #[tokio::test]
    async fn test_service_creation() {
        let _service = InversearchService::new().await;
        // Service should be created successfully
    }
}
