use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;

use crate::sync::batch::{
    BatchConfig, BatchProcessor, GenericBatchProcessor, TransactionBatchBuffer, TransactionBuffer,
};
use crate::sync::compensation::CompensationManager;
use crate::sync::dead_letter_queue::{DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig};
use crate::sync::external_index::{IndexData, IndexKey, IndexOperation, FulltextClient, VectorClient};
use crate::sync::metrics::SyncMetrics;
use super::types::{ChangeContext, ChangeData, ChangeType, IndexType};
use crate::search::manager::FulltextIndexManager;
use crate::sync::retry::{with_retry, RetryConfig};

type FulltextProcessor = GenericBatchProcessor<FulltextClient>;
type VectorProcessor = GenericBatchProcessor<VectorClient>;

pub struct SyncCoordinator {
    fulltext_manager: Arc<FulltextIndexManager>,
    vector_manager: Option<Arc<vector_client::VectorManager>>,
    fulltext_processors: DashMap<(u64, String, String), Arc<FulltextProcessor>>,
    vector_processors: DashMap<(u64, String, String), Arc<VectorProcessor>>,
    transaction_buffers:
        DashMap<crate::transaction::types::TransactionId, Arc<TransactionBatchBuffer>>,
    config: BatchConfig,
    metrics: Arc<SyncMetrics>,
    dead_letter_queue: Arc<DeadLetterQueue>,
    compensation_manager: Option<Arc<CompensationManager>>,
}

impl std::fmt::Debug for SyncCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncCoordinator")
            .field("fulltext_processors_count", &self.fulltext_processors.len())
            .field("vector_processors_count", &self.vector_processors.len())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl SyncCoordinator {
    pub fn new(fulltext_manager: Arc<FulltextIndexManager>, config: BatchConfig) -> Self {
        let metrics = Arc::new(SyncMetrics::new());
        let dead_letter_queue = Arc::new(DeadLetterQueue::new(
            DeadLetterQueueConfig::default()
        ));
        
        let compensation_manager = CompensationManager::new(
            dead_letter_queue.clone(),
            metrics.clone(),
        );

        Self {
            fulltext_manager,
            vector_manager: None,
            fulltext_processors: DashMap::new(),
            vector_processors: DashMap::new(),
            transaction_buffers: DashMap::new(),
            config,
            metrics,
            dead_letter_queue,
            compensation_manager: Some(Arc::new(compensation_manager)),
        }
    }

    pub fn with_vector_manager(
        mut self,
        vector_manager: Arc<vector_client::VectorManager>,
    ) -> Self {
        self.vector_manager = Some(vector_manager);
        self
    }

    pub fn metrics(&self) -> &Arc<SyncMetrics> {
        &self.metrics
    }

    pub fn dead_letter_queue(&self) -> &Arc<DeadLetterQueue> {
        &self.dead_letter_queue
    }

    pub fn compensation_manager(&self) -> Option<&Arc<CompensationManager>> {
        self.compensation_manager.as_ref()
    }

    pub fn fulltext_manager(&self) -> &Arc<FulltextIndexManager> {
        &self.fulltext_manager
    }



    fn get_or_create_fulltext_processor(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<FulltextProcessor>> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        if let Some(processor) = self.fulltext_processors.get(&key) {
            return Some(processor.clone());
        }

        let engine = self
            .fulltext_manager
            .get_engine(space_id, tag_name, field_name)?;

        let fulltext_client = Arc::new(FulltextClient::new(
            space_id,
            tag_name.to_string(),
            field_name.to_string(),
            engine,
        ));

        let processor = Arc::new(GenericBatchProcessor::new(
            fulltext_client,
            self.config.clone(),
        ));

        self.fulltext_processors
            .insert(key.clone(), processor.clone());

        Some(processor)
    }

    fn get_or_create_vector_processor(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<VectorProcessor>> {
        let vector_manager = self.vector_manager.as_ref()?;

        let key = (space_id, tag_name.to_string(), field_name.to_string());

        if let Some(processor) = self.vector_processors.get(&key) {
            return Some(processor.clone());
        }

        let vector_client = Arc::new(VectorClient::new(
            space_id,
            tag_name.to_string(),
            field_name.to_string(),
            vector_manager.clone(),
        ));

        let processor = Arc::new(GenericBatchProcessor::new(
            vector_client,
            self.config.clone(),
        ));

        self.vector_processors
            .insert(key.clone(), processor.clone());

        Some(processor)
    }

    pub async fn on_change(&self, ctx: ChangeContext) -> Result<(), SyncCoordinatorError> {
        let operation = self.create_operation(&ctx)?;

        match ctx.index_type {
            IndexType::Fulltext => {
                if let Some(processor) = self.get_or_create_fulltext_processor(
                    ctx.space_id,
                    &ctx.tag_name,
                    &ctx.field_name,
                ) {
                    processor.add(operation).await?;
                }
            }
            IndexType::Vector => {
                if let Some(processor) = self.get_or_create_vector_processor(
                    ctx.space_id,
                    &ctx.tag_name,
                    &ctx.field_name,
                ) {
                    processor.add(operation).await?;
                }
            }
        }

        Ok(())
    }

    fn create_operation(
        &self,
        ctx: &ChangeContext,
    ) -> Result<IndexOperation, SyncCoordinatorError> {
        let data = match &ctx.data {
            ChangeData::Fulltext(text) => IndexData::Fulltext(text.clone()),
            ChangeData::Vector(vector) => IndexData::Vector(vector.clone()),
        };

        let key = IndexKey::new(
            ctx.space_id,
            ctx.tag_name.clone(),
            ctx.field_name.clone(),
        );

        let operation = match ctx.change_type {
            ChangeType::Insert => IndexOperation::Insert {
                key,
                id: ctx.vertex_id.clone(),
                data,
                payload: HashMap::new(),
            },
            ChangeType::Update => IndexOperation::Update {
                key,
                id: ctx.vertex_id.clone(),
                data,
                payload: HashMap::new(),
            },
            ChangeType::Delete => IndexOperation::Delete {
                key,
                id: ctx.vertex_id.clone(),
            },
        };

        Ok(operation)
    }

    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &crate::core::Value,
        properties: &[(String, crate::core::Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncCoordinatorError> {
        let vid_str = vertex_id.to_string().unwrap_or_default();

        for (field_name, value) in properties {
            if self
                .fulltext_manager
                .get_engine(space_id, tag_name, field_name)
                .is_some()
            {
                if let crate::core::Value::String(text) = value {
                    let ctx = ChangeContext::new_fulltext(
                        space_id,
                        tag_name,
                        field_name,
                        change_type,
                        vid_str.clone(),
                        text.clone(),
                    );
                    self.on_change(ctx).await?;
                }
            }

            if let Some(ref _vm) = self.vector_manager {
                if let Some(vector) = value.as_vector() {
                    let ctx = ChangeContext::new_vector(
                        space_id,
                        tag_name,
                        field_name,
                        change_type,
                        vid_str.clone(),
                        vector,
                    );
                    self.on_change(ctx).await?;
                }
            }
        }

        Ok(())
    }

    /// 缓冲索引操作
    pub fn buffer_operation(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        ctx: ChangeContext,
    ) -> Result<(), SyncCoordinatorError> {
        // 创建索引操作
        let operation = self.create_operation(&ctx)?;

        // 获取或创建事务缓冲区
        let buffer = self
            .transaction_buffers
            .entry(txn_id)
            .or_insert_with(|| {
                Arc::new(TransactionBatchBuffer::new_without_processor())
            })
            .clone();

        // 添加操作到缓冲区
        futures::executor::block_on(buffer.prepare(txn_id, operation))
            .map_err(SyncCoordinatorError::BatchError)?;

        Ok(())
    }

    /// Prepare 阶段：验证所有操作
    pub async fn prepare_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        // 检查是否有该事务的缓冲区
        if let Some(buffer) = self.transaction_buffers.get(&txn_id) {
            let count = buffer.pending_count(txn_id);
            log::debug!("Transaction {:?} prepared with {} operations", txn_id, count);
        }
        Ok(())
    }

    /// Commit 阶段：应用所有操作
    pub async fn commit_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        let start_time = Instant::now();
        self.metrics.record_active_transaction_start();

        let result = if let Some((_, buffer)) = self.transaction_buffers.remove(&txn_id) {
            // 获取所有缓冲的操作（按 key 分组）
            let grouped_ops = buffer
                .take_operations(txn_id)
                .map_err(SyncCoordinatorError::BatchError)?;

            // 对每个 key 执行对应的操作
            for (key, operations) in grouped_ops {
                // 记录操作类型
                for op in &operations {
                    match op {
                        IndexOperation::Insert { .. } => {
                            self.metrics.record_index_operation("insert");
                        }
                        IndexOperation::Update { .. } => {
                            self.metrics.record_index_operation("update");
                        }
                        IndexOperation::Delete { .. } => {
                            self.metrics.record_index_operation("delete");
                        }
                    }
                }

                // 根据操作类型判断是全文索引还是向量索引
                let is_vector = operations.iter().any(|op| {
                    matches!(
                        op,
                        IndexOperation::Insert { data: IndexData::Vector(_), .. }
                            | IndexOperation::Update { data: IndexData::Vector(_), .. }
                    )
                });

                // 创建重试配置
                let retry_config = RetryConfig::new(3, Duration::from_millis(100), Duration::from_secs(5));
                
                if is_vector {
                    // 向量索引处理（带重试）
                    if let Some(processor) = self.get_or_create_vector_processor(
                        key.space_id,
                        &key.tag_name,
                        &key.field_name,
                    ) {
                        let ops_clone = operations.clone();
                        let retry_config_clone = retry_config.clone();
                        let metrics_clone = self.metrics.clone();
                        let dlq_clone = self.dead_letter_queue.clone();
                        
                        match with_retry(
                            || async {
                                processor.add_batch(ops_clone.clone()).await
                            },
                            &retry_config_clone,
                        )
                        .await
                        {
                            Ok(_) => {
                                metrics_clone.record_retry_success();
                            }
                            Err(e) => {
                                metrics_clone.record_retry_failure();
                                // 将失败的操作加入死信队列
                                for op in operations {
                                    let entry = DeadLetterEntry::new(
                                        op,
                                        format!("Index sync failed after retries: {:?}", e),
                                        retry_config_clone.max_retries,
                                    );
                                    dlq_clone.add(entry);
                                }
                                return Err(SyncCoordinatorError::BatchError(
                                    crate::sync::batch::BatchError::InvalidOperation(
                                        format!("Failed to sync index operations: {:?}", e)
                                    )
                                ));
                            }
                        }
                    }
                } else {
                    // 全文索引处理（带重试）
                    if let Some(processor) = self.get_or_create_fulltext_processor(
                        key.space_id,
                        &key.tag_name,
                        &key.field_name,
                    ) {
                        let ops_clone = operations.clone();
                        let retry_config_clone = retry_config.clone();
                        let metrics_clone = self.metrics.clone();
                        let dlq_clone = self.dead_letter_queue.clone();
                        
                        match with_retry(
                            || async {
                                processor.add_batch(ops_clone.clone()).await
                            },
                            &retry_config_clone,
                        )
                        .await
                        {
                            Ok(_) => {
                                metrics_clone.record_retry_success();
                            }
                            Err(e) => {
                                metrics_clone.record_retry_failure();
                                // 将失败的操作加入死信队列
                                for op in operations {
                                    let entry = DeadLetterEntry::new(
                                        op,
                                        format!("Index sync failed after retries: {:?}", e),
                                        retry_config_clone.max_retries,
                                    );
                                    dlq_clone.add(entry);
                                }
                                return Err(SyncCoordinatorError::BatchError(
                                    crate::sync::batch::BatchError::InvalidOperation(
                                        format!("Failed to sync index operations: {:?}", e)
                                    )
                                ));
                            }
                        }
                    }
                }
            }

            log::debug!("Transaction {:?} committed", txn_id);
            Ok(())
        } else {
            Ok(())
        };

        // 记录指标
        self.metrics.record_active_transaction_end();
        self.metrics.record_processing_time(start_time.elapsed());
        
        match &result {
            Ok(_) => self.metrics.record_transaction_commit(),
            Err(_) => self.metrics.record_transaction_rollback(),
        }

        result
    }

    /// Rollback 阶段：丢弃缓冲区
    pub async fn rollback_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        if let Some((_, buffer)) = self.transaction_buffers.remove(&txn_id) {
            let count = buffer.pending_count(txn_id);
            log::debug!("Transaction {:?} rolled back ({} operations discarded)", txn_id, count);
            buffer.rollback(txn_id).await.map_err(SyncCoordinatorError::BatchError)?;
        }
        Ok(())
    }

    pub async fn commit_all(&self) -> Result<(), SyncCoordinatorError> {
        for entry in self.fulltext_processors.iter() {
            let processor: &Arc<FulltextProcessor> = entry.value();
            processor.commit_all().await?;
        }

        for entry in self.vector_processors.iter() {
            let processor: &Arc<VectorProcessor> = entry.value();
            processor.commit_all().await?;
        }

        Ok(())
    }

    pub async fn start_background_tasks(&self) {
        log::info!("Starting background tasks for sync coordinator");

        // 启动所有处理器的背景任务
        for entry in self.fulltext_processors.iter() {
            let processor: Arc<FulltextProcessor> = entry.value().clone();
            processor.start_background_task().await;
        }

        for entry in self.vector_processors.iter() {
            let processor: Arc<VectorProcessor> = entry.value().clone();
            processor.start_background_task().await;
        }

        // 启动补偿背景任务（如果启用了补偿管理器）
        if let Some(compensation_manager) = &self.compensation_manager {
            let cm_clone = compensation_manager.clone();
            tokio::spawn(async move {
                cm_clone.start_background_task(Duration::from_secs(60)).await;
            });
            log::info!("Started compensation background task");
        }

        // 启动死信队列自动清理任务
        if self.dead_letter_queue.is_auto_cleanup_enabled() {
            let dlq_clone = self.dead_letter_queue.clone();
            let cleanup_interval = self.dead_letter_queue.get_cleanup_interval();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(cleanup_interval);
                loop {
                    interval.tick().await;
                    let removed = dlq_clone.cleanup();
                    if removed > 0 {
                        log::info!("Auto-cleaned {} dead letter entries", removed);
                    }
                }
            });
            log::info!("Started dead letter queue cleanup task");
        }

        log::info!("All background tasks started");
    }

    pub async fn stop_background_tasks(&self) {
        log::info!("Stopping background tasks for sync coordinator");

        // 停止所有处理器的背景任务
        for entry in self.fulltext_processors.iter() {
            let processor: &Arc<FulltextProcessor> = entry.value();
            processor.stop_background_task().await;
        }

        for entry in self.vector_processors.iter() {
            let processor: &Arc<VectorProcessor> = entry.value();
            processor.stop_background_task().await;
        }

        // 注意：补偿任务和清理任务是 tokio::spawn 的，会在 coordinator 销毁时自动停止

        log::info!("All background tasks stopped");
    }

    pub fn is_auto_cleanup_enabled(&self) -> bool {
        self.dead_letter_queue.is_auto_cleanup_enabled()
    }

    pub fn get_cleanup_interval(&self) -> Duration {
        self.dead_letter_queue.get_cleanup_interval()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncCoordinatorError {
    #[error("Index error: {0}")]
    IndexError(#[from] crate::sync::external_index::ExternalIndexError),

    #[error("Batch error: {0}")]
    BatchError(#[from] crate::sync::batch::BatchError),

    #[error("Fulltext coordinator error: {0}")]
    FulltextError(#[from] crate::core::error::CoordinatorError),

    #[error("Vector coordinator error: {0}")]
    VectorError(#[from] crate::core::error::VectorCoordinatorError),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
