use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::{StorageInfo, StorageInterface};
use redis::{AsyncCommands, Client as RedisClient, aio::MultiplexedConnection};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: usize,
    pub connection_timeout: Duration,
    pub key_prefix: String,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout: Duration::from_secs(5),
            key_prefix: "inversearch".to_string(),
        }
    }
}

pub struct RedisStorage {
    client: RedisClient,
    config: RedisStorageConfig,
    key_prefix: String,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let mut conn = client
            .get_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            config,
            key_prefix,
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn make_index_key(&self, term: &str) -> String {
        self.make_key(&format!("index:{}", term))
    }

    fn make_context_key(&self, context: &str, term: &str) -> String {
        self.make_key(&format!("ctx:{}:{}", context, term))
    }

    fn make_doc_key(&self, doc_id: DocId) -> String {
        self.make_key(&format!("doc:{}", doc_id))
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()).into())
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        if !keys.is_empty() {
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let mut conn = self.get_connection().await?;

        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

                let _: () = redis::cmd("SET")
                    .arg(&key)
                    .arg(&serialized)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            }
        }

        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

                let _: () = redis::cmd("SET")
                    .arg(&key)
                    .arg(&serialized)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let mut conn = self.get_connection().await?;

        let redis_key = if let Some(ctx_key) = ctx {
            self.make_context_key(ctx_key, key)
        } else {
            self.make_index_key(key)
        };

        let serialized: String = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        if serialized.is_empty() {
            return Ok(Vec::new());
        }

        let doc_ids: Vec<DocId> = serde_json::from_str(&serialized)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        let start = offset.min(doc_ids.len());
        let end = if limit > 0 {
            (start + limit).min(doc_ids.len())
        } else {
            doc_ids.len()
        };

        Ok(doc_ids[start..end].to_vec())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut conn = self.get_connection().await?;
        let mut results = Vec::new();

        for &id in ids {
            let key = self.make_doc_key(id);
            let serialized: String = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

            if !serialized.is_empty() {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::from_str(&serialized)
                        .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?),
                    highlight: None,
                });
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let key = self.make_doc_key(id);

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(exists)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        let mut conn = self.get_connection().await?;

        for &id in ids {
            let key = self.make_doc_key(id);
            let _: () = redis::cmd("DEL")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.destroy().await
    }

    async fn info(&self) -> Result<StorageInfo> {
        let mut conn = self.get_connection().await?;

        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: "0.1.0".to_string(),
            size: keys.len() as u64,
            document_count: 0,
            index_count: keys.len(),
            is_connected: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_redis_storage() {
        let config = RedisStorageConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            ..Default::default()
        };

        let mut storage = RedisStorage::new(config).await.unwrap();
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        storage.commit(&index, false, false).await.unwrap();

        let results = storage.get("hello", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        let has_result = storage.has(1).await.unwrap();
        assert!(has_result);
        assert!(!storage.has(3).await.unwrap());

        storage.destroy().await.unwrap();
    }
}
