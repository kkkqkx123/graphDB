use super::config::Config;
use super::proto::Bm25Service as Bm25ServiceTrait;
use super::proto::Bm25ServiceServer;
use super::proto::{
    BatchIndexDocumentsRequest, BatchIndexDocumentsResponse, ClearIndexRequest, ClearIndexResponse,
    CommitIndexRequest, CommitIndexResponse, DeleteDocumentRequest, DeleteDocumentResponse,
    GetStatsRequest, GetStatsResponse, IndexDocumentRequest, IndexDocumentResponse, SearchRequest,
    SearchResponse,
};
use crate::api::core::{batch, delete, document, search, stats};
use crate::api::core::{IndexManager, IndexSchema};
use crate::config::StorageType;
use crate::storage::{MutableStorageManager, StorageManagerBuilder};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

pub struct BM25Service {
    _config: Config,
    index_path: PathBuf,
    indexes: Arc<RwLock<HashMap<String, (IndexManager, IndexSchema)>>>,
    storage: Option<Arc<MutableStorageManager>>,
}

impl BM25Service {
    pub fn new(config: Config) -> Self {
        let index_path = PathBuf::from(&config.index.index_path);
        Self {
            _config: config,
            index_path,
            indexes: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
        }
    }

    /// Create a service and initialize the storage tier
    pub async fn with_storage(config: Config) -> Result<Self, anyhow::Error> {
        let index_path = PathBuf::from(&config.index.index_path);

        // Create a storage manager based on the configuration
        let storage_manager = match config.storage.storage_type {
            StorageType::Tantivy => {
                #[cfg(feature = "storage-tantivy")]
                {
                    let tantivy_config = crate::storage::tantivy::TantivyStorageConfig {
                        index_path: std::path::PathBuf::from(&config.storage.tantivy.index_path),
                        writer_memory_mb: config.storage.tantivy.writer_memory_mb,
                    };
                    Arc::new(StorageManagerBuilder::build_mutable_tantivy(
                        tantivy_config,
                    )?)
                }
                #[cfg(not(feature = "storage-tantivy"))]
                {
                    return Err(anyhow::anyhow!("Tantivy storage is not enabled"));
                }
            }
            StorageType::Redis => {
                #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
                {
                    let redis_config = crate::storage::redis::RedisStorageConfig {
                        url: config.storage.redis.url.clone(),
                        pool_size: config.storage.redis.pool_size,
                        connection_timeout: std::time::Duration::from_secs(
                            config.storage.redis.connection_timeout_secs,
                        ),
                        key_prefix: config.storage.redis.key_prefix.clone(),
                        min_idle: config.storage.redis.min_idle,
                        max_lifetime: config
                            .storage
                            .redis
                            .max_lifetime_secs
                            .map(std::time::Duration::from_secs),
                        connection_timeout_bb8: std::time::Duration::from_secs(
                            config.storage.redis.connection_timeout_secs,
                        ),
                    };
                    Arc::new(StorageManagerBuilder::build_mutable_redis(redis_config).await?)
                }
                #[cfg(feature = "storage-tantivy")]
                {
                    // When storage-tantivy is enabled (whether storage-redis is enabled or not), the
                    // DefaultStorage is TantivyStorage, so it can't use the Redis storage manager
                    // This uses Tantivy as a fallback
                    let tantivy_config = crate::storage::tantivy::TantivyStorageConfig {
                        index_path: std::path::PathBuf::from(&config.storage.tantivy.index_path),
                        writer_memory_mb: config.storage.tantivy.writer_memory_mb,
                    };
                    Arc::new(StorageManagerBuilder::build_mutable_tantivy(
                        tantivy_config,
                    )?)
                }
                #[cfg(not(any(feature = "storage-redis", feature = "storage-tantivy")))]
                {
                    return Err(anyhow::anyhow!("No storage backend is enabled"));
                }
            }
        };

        // Initializing Storage
        storage_manager.init().await?;

        Ok(Self {
            _config: config,
            index_path,
            indexes: Arc::new(RwLock::new(HashMap::new())),
            storage: Some(storage_manager),
        })
    }

    async fn get_or_create_index(
        &self,
        index_name: &str,
    ) -> Result<(IndexManager, IndexSchema), Status> {
        let mut indexes = self.indexes.write().await;

        if let Some((manager_ref, schema_ref)) = indexes.get(index_name) {
            return Ok((manager_ref.clone(), schema_ref.clone()));
        }

        let index_path = self.index_path.join(index_name);
        let manager = IndexManager::create(&index_path)
            .map_err(|e| Status::internal(format!("Failed to create index: {}", e)))?;
        let schema = IndexSchema::new();

        indexes.insert(index_name.to_string(), (manager.clone(), schema.clone()));
        Ok((manager, schema))
    }
}

#[tonic::async_trait]
impl Bm25ServiceTrait for BM25Service {
    async fn index_document(
        &self,
        request: Request<IndexDocumentRequest>,
    ) -> Result<Response<IndexDocumentResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "Received index document request: index={}, id={}",
            req.index_name,
            req.document_id
        );

        let (manager, schema) = self.get_or_create_index(&req.index_name).await?;
        let fields: HashMap<String, String> = req.fields.into_iter().collect();

        // Using Storage Layer Integration
        if let Some(ref storage) = self.storage {
            let avg_doc_length = self._config.bm25.avg_doc_length;
            document::add_document_with_storage(
                &manager,
                storage,
                &schema,
                &req.document_id,
                &fields,
                avg_doc_length,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to index document: {}", e)))?;
        } else {
            document::add_document(&manager, &schema, &req.document_id, &fields)
                .map_err(|e| Status::internal(format!("Failed to index document: {}", e)))?;
        }

        Ok(Response::new(IndexDocumentResponse {
            success: true,
            message: "Document indexed successfully".to_string(),
        }))
    }

    async fn batch_index_documents(
        &self,
        request: Request<BatchIndexDocumentsRequest>,
    ) -> Result<Response<BatchIndexDocumentsResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "Received batch index documents request: index={}, count={}",
            req.index_name,
            req.documents.len()
        );

        let (manager, schema) = self.get_or_create_index(&req.index_name).await?;
        let documents: Vec<(String, HashMap<String, String>)> = req
            .documents
            .into_iter()
            .map(|doc| {
                let fields: HashMap<String, String> = doc.fields.into_iter().collect();
                (doc.document_id, fields)
            })
            .collect();

        // Using Storage Layer Integration
        let count = if let Some(ref storage) = self.storage {
            let avg_doc_length = self._config.bm25.avg_doc_length;
            batch::batch_add_documents_with_storage(
                &manager,
                storage,
                &schema,
                documents,
                avg_doc_length,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to batch index documents: {}", e)))?
        } else {
            batch::batch_add_documents(&manager, &schema, documents)
                .map_err(|e| Status::internal(format!("Failed to batch index documents: {}", e)))?
        };

        Ok(Response::new(BatchIndexDocumentsResponse {
            success: true,
            message: "Documents indexed successfully".to_string(),
            indexed_count: count as i32,
        }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "Received search request: index={}, query={}",
            req.index_name,
            req.query
        );

        let (manager, schema) = self.get_or_create_index(&req.index_name).await?;

        let options = search::SearchOptions {
            limit: if req.limit > 0 {
                req.limit as usize
            } else {
                10
            },
            offset: req.offset as usize,
            field_weights: req.field_weights.into_iter().collect(),
            highlight: req.highlight,
        };

        let (results, max_score) = search::search(&manager, &schema, &req.query, &options)
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let total_count = results.len();
        let search_results = results
            .into_iter()
            .map(|r| super::proto::SearchResult {
                document_id: r.document_id,
                score: r.score,
                fields: r.fields,
                highlights: r.highlights,
            })
            .collect();

        Ok(Response::new(SearchResponse {
            results: search_results,
            total: total_count as i32,
            max_score,
        }))
    }

    async fn delete_document(
        &self,
        request: Request<DeleteDocumentRequest>,
    ) -> Result<Response<DeleteDocumentResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "Received delete document request: index={}, id={}",
            req.index_name,
            req.document_id
        );

        let (manager, schema) = self.get_or_create_index(&req.index_name).await?;

        // Using Storage Layer Integration
        if let Some(ref storage) = self.storage {
            delete::delete_document_with_storage(&manager, storage, &schema, &req.document_id)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete document: {}", e)))?;
        } else {
            delete::delete_document(&manager, &schema, &req.document_id)
                .map_err(|e| Status::internal(format!("Failed to delete document: {}", e)))?;
        }

        Ok(Response::new(DeleteDocumentResponse {
            success: true,
            message: "Document deleted successfully".to_string(),
        }))
    }

    async fn get_stats(
        &self,
        request: Request<GetStatsRequest>,
    ) -> Result<Response<GetStatsResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Received get stats request: index={}", req.index_name);

        let (manager, _schema) = self.get_or_create_index(&req.index_name).await?;
        let index_stats = stats::get_stats(&manager)
            .map_err(|e| Status::internal(format!("Failed to get stats: {}", e)))?;

        Ok(Response::new(GetStatsResponse {
            total_documents: index_stats.total_documents as i64,
            total_terms: index_stats.total_terms as i64,
            avg_document_length: index_stats.avg_document_length,
        }))
    }

    async fn clear_index(
        &self,
        request: Request<ClearIndexRequest>,
    ) -> Result<Response<ClearIndexResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Received clear index request: index={}", req.index_name);

        // Get document count before clearing
        let (manager, _schema) = self.get_or_create_index(&req.index_name).await?;
        let stats = stats::get_stats(&manager)
            .map_err(|e| Status::internal(format!("Failed to get stats: {}", e)))?;

        let cleared_count = stats.total_documents as i32;

        // Delete index directory
        let index_path = self.index_path.join(&req.index_name);
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path).map_err(|e| {
                Status::internal(format!("Failed to remove index directory: {}", e))
            })?;

            // Remove index reference from memory
            let mut indexes = self.indexes.write().await;
            indexes.remove(&req.index_name);

            tracing::info!(
                "Cleared index '{}': {} documents removed",
                req.index_name,
                cleared_count
            );
        }

        Ok(Response::new(ClearIndexResponse {
            success: true,
            message: format!("Index '{}' cleared successfully", req.index_name),
            cleared_count,
        }))
    }

    async fn commit_index(
        &self,
        request: Request<CommitIndexRequest>,
    ) -> Result<Response<CommitIndexResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Received commit index request: index={}", req.index_name);

        let (manager, _schema) = self.get_or_create_index(&req.index_name).await?;

        let stats_before = stats::get_stats(&manager)
            .map_err(|e| Status::internal(format!("Failed to get stats: {}", e)))?;

        let mut writer = manager
            .writer()
            .map_err(|e| Status::internal(format!("Failed to get writer: {}", e)))?;

        writer
            .commit()
            .map_err(|e| Status::internal(format!("Failed to commit: {}", e)))?;

        manager
            .reload_reader()
            .map_err(|e| Status::internal(format!("Failed to reload reader: {}", e)))?;

        tracing::info!(
            "Committed index '{}': {} documents committed",
            req.index_name,
            stats_before.total_documents
        );

        Ok(Response::new(CommitIndexResponse {
            success: true,
            message: format!("Index '{}' committed successfully", req.index_name),
            committed_documents: stats_before.total_documents as i64,
        }))
    }
}

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    let addr = config.server.address;
    tracing::info!("BM25 service listening on {}", addr);

    // Create a service and initialize the storage tier
    let bm25_service = BM25Service::with_storage(config).await?;

    Server::builder()
        .add_service(Bm25ServiceServer::new(bm25_service))
        .serve(addr)
        .await?;

    Ok(())
}
