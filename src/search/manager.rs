use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::search::config::FulltextConfig;
use crate::search::engine::{EngineType, SearchEngine};
use crate::search::error::SearchError;
use crate::search::factory::SearchEngineFactory;
use crate::search::metadata::{IndexKey, IndexMetadata, IndexStatus};
use crate::search::result::{IndexStats, SearchResult};
use crate::storage::metadata::SchemaManager;

#[derive(Debug)]
pub struct FulltextIndexManager {
    engines: DashMap<IndexKey, Arc<dyn SearchEngine>>,
    metadata: DashMap<IndexKey, IndexMetadata>,
    base_path: PathBuf,
    default_engine: EngineType,
    config: FulltextConfig,
    schema_manager: Option<Arc<dyn SchemaManager>>,
}

impl FulltextIndexManager {
    pub fn new(config: FulltextConfig) -> Result<Self, SearchError> {
        let base_path = config.index_path.clone();

        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }

        Ok(Self {
            engines: DashMap::new(),
            metadata: DashMap::new(),
            base_path,
            default_engine: config.default_engine,
            config,
            schema_manager: None,
        })
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn set_schema_manager(&mut self, schema_manager: Arc<dyn SchemaManager>) {
        self.schema_manager = Some(schema_manager);
    }

    fn validate_space_exists(&self, space_id: u64) -> Result<(), SearchError> {
        if let Some(ref schema_manager) = self.schema_manager {
            let space_exists = schema_manager
                .get_space_by_id(space_id)
                .map_err(|e| SearchError::Internal(format!("Failed to validate space: {}", e)))?
                .is_some();

            if !space_exists {
                return Err(SearchError::SpaceNotFound(space_id));
            }
        }
        Ok(())
    }

    fn validate_tag_exists(&self, space_id: u64, tag_name: &str) -> Result<(), SearchError> {
        if let Some(ref schema_manager) = self.schema_manager {
            let space = schema_manager
                .get_space_by_id(space_id)
                .map_err(|e| SearchError::Internal(format!("Failed to validate tag: {}", e)))?;

            if let Some(space_info) = space {
                let tag_exists = space_info.tags.iter().any(|t| t.tag_name == tag_name);
                if !tag_exists {
                    return Err(SearchError::TagNotFound(format!(
                        "{}.{}",
                        space_id, tag_name
                    )));
                }
            }
        }
        Ok(())
    }

    fn get_space_storage_path(&self, space_id: u64) -> Result<PathBuf, SearchError> {
        if let Some(ref schema_manager) = self.schema_manager {
            if let Some(space_info) = schema_manager
                .get_space_by_id(space_id)
                .map_err(|e| SearchError::Internal(format!("Failed to get space: {}", e)))?
            {
                if let Some(ref custom_path) = space_info.storage_path {
                    let fulltext_path = custom_path.join("fulltext");
                    if !fulltext_path.exists() {
                        std::fs::create_dir_all(&fulltext_path)?;
                    }
                    return Ok(fulltext_path);
                }

                use crate::core::types::space::IsolationLevel;
                match space_info.isolation_level {
                    IsolationLevel::Device => {
                        if let Some(ref custom_path) = space_info.storage_path {
                            let fulltext_path = custom_path.join("fulltext");
                            if !fulltext_path.exists() {
                                std::fs::create_dir_all(&fulltext_path)?;
                            }
                            return Ok(fulltext_path);
                        }
                    }
                    IsolationLevel::Directory => {
                        let space_path = self.base_path.join(format!("space_{}", space_id));
                        if !space_path.exists() {
                            std::fs::create_dir_all(&space_path)?;
                        }
                        return Ok(space_path);
                    }
                    IsolationLevel::Shared => {}
                }
            }
        }
        Ok(self.base_path.clone())
    }

    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, SearchError> {
        self.validate_space_exists(space_id)?;
        self.validate_tag_exists(space_id, tag_name)?;

        let key = IndexKey::new(space_id, tag_name, field_name);
        let index_id = key.to_index_id();

        if self.engines.contains_key(&key) {
            return Err(SearchError::IndexAlreadyExists(index_id));
        }

        let engine_type = engine_type.unwrap_or(self.default_engine);

        let storage_path = self.get_space_storage_path(space_id)?;

        let engine = SearchEngineFactory::from_config(
            engine_type,
            &index_id,
            &storage_path,
            &self.config,
        )?;

        let metadata = IndexMetadata {
            index_id: index_id.clone(),
            index_name: format!("idx_{}_{}_{}", space_id, tag_name, field_name),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            engine_type,
            storage_path: storage_path.join(&index_id).to_string_lossy().to_string(),
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            doc_count: 0,
            status: IndexStatus::Active,
            engine_config: None,
        };

        self.engines.insert(key.clone(), engine);
        self.metadata.insert(key, metadata);

        Ok(index_id)
    }

    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn SearchEngine>> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.engines.get(&key).map(|e| Arc::clone(&*e))
    }

    pub fn get_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<IndexMetadata> {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.metadata.get(&key).map(|m| m.clone())
    }

    pub fn has_index(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        let key = IndexKey::new(space_id, tag_name, field_name);
        self.engines.contains_key(&key)
    }

    pub fn get_space_indexes(&self, space_id: u64) -> Vec<IndexMetadata> {
        self.metadata
            .iter()
            .filter(|entry| entry.value().space_id == space_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        let key = IndexKey::new(space_id, tag_name, field_name);

        if let Some((_, engine)) = self.engines.remove(&key) {
            engine.close().await?;
        }

        self.metadata.remove(&key);

        let index_id = key.to_index_id();
        let index_path = self.base_path.join(&index_id);
        if index_path.exists() {
            tokio::fs::remove_dir_all(&index_path).await?;
        }

        let bin_path = index_path.with_extension("bin");
        if bin_path.exists() {
            tokio::fs::remove_file(&bin_path).await?;
        }

        Ok(())
    }

    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let engine = self
            .get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| {
                SearchError::IndexNotFound(format!("{}.{}.{}", space_id, tag_name, field_name))
            })?;

        engine.search(query, limit).await
    }

    pub async fn get_stats(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<IndexStats, SearchError> {
        let engine = self
            .get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| {
                SearchError::IndexNotFound(format!("{}.{}.{}", space_id, tag_name, field_name))
            })?;

        engine.stats().await
    }

    pub async fn commit_all(&self) -> Result<(), SearchError> {
        for entry in self.engines.iter() {
            entry.value().commit().await?;
        }
        Ok(())
    }

    pub async fn close_all(&self) -> Result<(), SearchError> {
        for entry in self.engines.iter() {
            entry.value().close().await?;
        }
        self.engines.clear();
        self.metadata.clear();
        Ok(())
    }

    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.metadata
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 为边属性建立索引
    pub async fn index_edge_property(
        &self,
        space_id: u64,
        edge_type: &str,
        field_name: &str,
        doc_id: &str,
        text: &str,
    ) -> Result<(), SearchError> {
        let key = IndexKey::new(space_id, edge_type, field_name);

        if let Some(engine) = self.engines.get(&key) {
            engine.index(doc_id, text).await?;
        }
        // 如果索引不存在，静默忽略（边可能没有配置全文索引）
        Ok(())
    }

    /// 删除边的所有全文索引
    pub async fn delete_edge_index(
        &self,
        space_id: u64,
        edge_type: &str,
        doc_id: &str,
    ) -> Result<(), SearchError> {
        // 获取该边类型的所有字段索引
        let edge_indexes: Vec<_> = self
            .metadata
            .iter()
            .filter(|entry| {
                entry.value().space_id == space_id && entry.value().tag_name == edge_type
            })
            .map(|entry| entry.key().clone())
            .collect();

        // 删除每个字段索引中的边文档
        for key in edge_indexes {
            if let Some(engine) = self.engines.get(&key) {
                engine.delete(doc_id).await.ok(); // 忽略删除失败
            }
        }

        Ok(())
    }

    /// Drop all indexes for a space
    ///
    /// This method should be called when dropping a space to ensure
    /// all associated index data is cleaned up.
    pub async fn drop_space_indexes(&self, space_id: u64) -> Result<(), SearchError> {
        let space_indexes: Vec<(IndexKey, IndexMetadata)> = self
            .metadata
            .iter()
            .filter(|entry| entry.value().space_id == space_id)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        for (key, metadata) in space_indexes {
            if let Some((_, engine)) = self.engines.remove(&key) {
                engine.close().await.ok();
            }
            self.metadata.remove(&key);

            let storage_path = PathBuf::from(&metadata.storage_path);
            if storage_path.exists() {
                tokio::fs::remove_dir_all(&storage_path).await.ok();
            }

            let bin_path = storage_path.with_extension("bin");
            if bin_path.exists() {
                tokio::fs::remove_file(&bin_path).await.ok();
            }
        }

        if let Some(ref schema_manager) = self.schema_manager {
            if let Some(space_info) = schema_manager
                .get_space_by_id(space_id)
                .map_err(|e| SearchError::Internal(format!("Failed to get space: {}", e)))?
            {
                if let Some(ref custom_path) = space_info.storage_path {
                    let fulltext_path = custom_path.join("fulltext");
                    if fulltext_path.exists() {
                        tokio::fs::remove_dir_all(&fulltext_path).await.ok();
                    }
                } else if space_info.isolation_level == crate::core::types::space::IsolationLevel::Directory {
                    let space_path = self.base_path.join(format!("space_{}", space_id));
                    if space_path.exists() {
                        tokio::fs::remove_dir_all(&space_path).await.ok();
                    }
                }
            }
        }

        Ok(())
    }
}
