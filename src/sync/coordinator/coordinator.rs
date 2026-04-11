use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::RwLock;

use crate::sync::batch::{
    BatchConfig, BatchProcessor, GenericBatchProcessor, TransactionBatchBuffer, TransactionBuffer,
};
use crate::sync::external_index::{ExternalIndexClient, IndexData, IndexOperation, FulltextClient, VectorClient};
use super::types::{ChangeContext, ChangeData, ChangeType, IndexType};
use crate::search::manager::FulltextIndexManager;

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
    mode: Arc<RwLock<crate::sync::SyncMode>>,
}

impl std::fmt::Debug for SyncCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncCoordinator")
            .field("fulltext_processors_count", &self.fulltext_processors.len())
            .field("vector_processors_count", &self.vector_processors.len())
            .field("config", &self.config)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl SyncCoordinator {
    pub fn new(fulltext_manager: Arc<FulltextIndexManager>, config: BatchConfig) -> Self {
        Self {
            fulltext_manager,
            vector_manager: None,
            fulltext_processors: DashMap::new(),
            vector_processors: DashMap::new(),
            transaction_buffers: DashMap::new(),
            config,
            mode: Arc::new(RwLock::new(crate::sync::SyncMode::Async)),
        }
    }

    pub fn with_vector_manager(
        mut self,
        vector_manager: Arc<vector_client::VectorManager>,
    ) -> Self {
        self.vector_manager = Some(vector_manager);
        self
    }

    pub fn with_mode(mut self, mode: crate::sync::SyncMode) -> Self {
        self.mode = Arc::new(RwLock::new(mode));
        self
    }

    pub fn mode(&self) -> Arc<RwLock<crate::sync::SyncMode>> {
        self.mode.clone()
    }

    pub async fn set_mode(&self, mode: crate::sync::SyncMode) {
        let mut m = self.mode.write().await;
        *m = mode;
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
        let mode = *self.mode.read().await;

        if mode == crate::sync::SyncMode::Off {
            return Ok(());
        }

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

        let operation = match ctx.change_type {
            ChangeType::Insert => IndexOperation::Insert {
                id: ctx.vertex_id.clone(),
                data,
                payload: HashMap::new(),
            },
            ChangeType::Update => IndexOperation::Update {
                id: ctx.vertex_id.clone(),
                data,
                payload: HashMap::new(),
            },
            ChangeType::Delete => IndexOperation::Delete {
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
        let mode = *self.mode.read().await;

        if mode == crate::sync::SyncMode::Off {
            return Ok(());
        }

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

    pub async fn prepare_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        ctx: ChangeContext,
    ) -> Result<(), SyncCoordinatorError> {
        let operation = self.create_operation(&ctx)?;

        let processor: Arc<dyn BatchProcessor> = match ctx.index_type {
            IndexType::Fulltext => {
                match self.get_or_create_fulltext_processor(
                    ctx.space_id,
                    &ctx.tag_name,
                    &ctx.field_name,
                ) {
                    Some(p) => p as Arc<dyn BatchProcessor>,
                    None => return Ok(()),
                }
            }
            IndexType::Vector => {
                match self.get_or_create_vector_processor(
                    ctx.space_id,
                    &ctx.tag_name,
                    &ctx.field_name,
                ) {
                    Some(p) => p as Arc<dyn BatchProcessor>,
                    None => return Ok(()),
                }
            }
        };

        let buffer = self
            .transaction_buffers
            .entry(txn_id)
            .or_insert_with(|| Arc::new(TransactionBatchBuffer::new(processor)));

        buffer.prepare(txn_id, operation).await?;

        Ok(())
    }

    pub async fn commit_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        if let Some((_, buffer)) = self.transaction_buffers.remove(&txn_id) {
            TransactionBuffer::commit(&*buffer, txn_id).await?;
        }
        Ok(())
    }

    pub async fn rollback_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        self.transaction_buffers.remove(&txn_id);
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
        for entry in self.fulltext_processors.iter() {
            let processor: Arc<FulltextProcessor> = entry.value().clone();
            processor.start_background_task().await;
        }

        for entry in self.vector_processors.iter() {
            let processor: Arc<VectorProcessor> = entry.value().clone();
            processor.start_background_task().await;
        }
    }

    pub async fn stop_background_tasks(&self) {
        for entry in self.fulltext_processors.iter() {
            let processor: &Arc<FulltextProcessor> = entry.value();
            processor.stop_background_task().await;
        }

        for entry in self.vector_processors.iter() {
            let processor: &Arc<VectorProcessor> = entry.value();
            processor.stop_background_task().await;
        }
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
