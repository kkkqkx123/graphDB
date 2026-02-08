//! 全文索引模块
//!
//! 提供基于文本的全文索引功能，支持：
//! - 创建全文索引
//! - 删除全文索引
//! - 全文搜索查询
//! - 支持中英文分词
//!
//! 集成外部全文索引引擎（如 tantivy 或 rust-analyzer）

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::error::{DBError, DBResult};
use crate::core::Value;

#[derive(Error, Debug, Clone)]
pub enum FulltextIndexError {
    #[error("全文索引引擎错误: {0}")]
    EngineError(String),
    
    #[error("索引不存在: {0}")]
    IndexNotFound(String),
    
    #[error("索引已存在: {0}")]
    IndexAlreadyExists(String),
    
    #[error("文档格式错误: {0}")]
    DocumentFormatError(String),
    
    #[error("查询语法错误: {0}")]
    QuerySyntaxError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextIndexConfig {
    pub name: String,
    pub schema_type: FulltextSchemaType,
    pub schema_name: String,
    pub fields: Vec<String>,
    pub analyzer: Option<String>,
    pub case_sensitive: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FulltextSchemaType {
    Tag,
    Edge,
}

impl Default for FulltextSchemaType {
    fn default() -> Self {
        FulltextSchemaType::Tag
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextDocument {
    pub id: String,
    pub schema_type: FulltextSchemaType,
    pub schema_name: String,
    pub content: HashMap<String, Value>,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

impl FulltextDocument {
    pub fn new(
        id: String,
        schema_type: FulltextSchemaType,
        schema_name: String,
    ) -> Self {
        Self {
            id,
            schema_type,
            schema_name,
            content: HashMap::new(),
            indexed_at: chrono::Utc::now(),
        }
    }

    pub fn add_field(&mut self, field: String, value: Value) {
        self.content.insert(field, value);
    }

    pub fn get_text_content(&self) -> String {
        self.content
            .values()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Null(_) => None,
                _ => Some(v.to_string().unwrap_or_else(|_| format!("{:?}", v))),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextSearchResult {
    pub id: String,
    pub score: f32,
    pub highlights: HashMap<String, String>,
}

impl FulltextSearchResult {
    pub fn new(id: String, score: f32) -> Self {
        Self {
            id,
            score,
            highlights: HashMap::new(),
        }
    }

    pub fn add_highlight(&mut self, field: String, highlighted: String) {
        self.highlights.insert(field, highlighted);
    }
}

#[derive(Debug, Clone)]
pub struct FulltextQuery {
    pub index_name: String,
    pub query_string: String,
    pub fields: Option<Vec<String>>,
    pub limit: usize,
    pub offset: usize,
}

impl FulltextQuery {
    pub fn new(index_name: String, query_string: String) -> Self {
        Self {
            index_name,
            query_string,
            fields: None,
            limit: 100,
            offset: 0,
        }
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

pub trait FulltextIndexEngine: Send + Sync {
    fn create_index(&mut self, config: &FulltextIndexConfig) -> DBResult<()>;
    
    fn drop_index(&mut self, name: &str) -> DBResult<()>;
    
    fn index_document(&mut self, doc: &FulltextDocument) -> DBResult<()>;
    
    fn delete_document(&mut self, id: &str) -> DBResult<()>;
    
    fn search(&mut self, query: &FulltextQuery) -> DBResult<Vec<FulltextSearchResult>>;
    
    fn index_exists(&self, name: &str) -> bool;
    
    fn get_index_config(&self, name: &str) -> Option<FulltextIndexConfig>;
    
    fn list_index_configs(&self) -> Vec<FulltextIndexConfig>;
}

pub struct SimpleFulltextEngine {
    indices: HashMap<String, FulltextIndexConfig>,
    documents: HashMap<String, HashMap<String, FulltextDocument>>,
    inverted_index: HashMap<String, HashSet<String>>,
}

impl SimpleFulltextEngine {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for SimpleFulltextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FulltextIndexEngine for SimpleFulltextEngine {
    fn create_index(&mut self, config: &FulltextIndexConfig) -> DBResult<()> {
        if self.indices.contains_key(&config.name) {
            return Err(DBError::FulltextIndex(
                FulltextIndexError::IndexAlreadyExists(config.name.clone()),
            ));
        }

        self.indices.insert(config.name.clone(), config.clone());
        self.documents.insert(config.name.clone(), HashMap::new());

        Ok(())
    }

    fn drop_index(&mut self, name: &str) -> DBResult<()> {
        if !self.indices.contains_key(name) {
            return Err(DBError::FulltextIndex(
                FulltextIndexError::IndexNotFound(name.to_string()),
            ));
        }

        self.indices.remove(name);
        self.documents.remove(name);

        self.inverted_index.retain(|_, ids| {
            ids.retain(|id| !id.starts_with(&format!("{}:", name)));
            !ids.is_empty()
        });

        Ok(())
    }

    fn index_document(&mut self, doc: &FulltextDocument) -> DBResult<()> {
        let index_name = &doc.schema_name;
        if !self.indices.contains_key(index_name) {
            return Err(DBError::FulltextIndex(
                FulltextIndexError::IndexNotFound(index_name.clone()),
            ));
        }

        let doc_id = format!("{}:{}", index_name, doc.id);
        let text_content = doc.get_text_content();
        let tokens = Self::tokenize(&text_content);

        if let Some(index_docs) = self.documents.get_mut(index_name) {
            index_docs.insert(doc.id.clone(), doc.clone());
        }

        for token in tokens {
            self.inverted_index
                .entry(token)
                .or_insert_with(HashSet::new)
                .insert(doc_id.clone());
        }

        Ok(())
    }

    fn delete_document(&mut self, id: &str) -> DBResult<()> {
        for (index_name, index_docs) in &mut self.documents {
            let doc_id = format!("{}:{}", index_name, id);
            if index_docs.contains_key(id) {
                index_docs.remove(id);

                self.inverted_index.retain(|_, ids| {
                    ids.retain(|doc_id_ref| *doc_id_ref != doc_id);
                    !ids.is_empty()
                });

                return Ok(());
            }
        }

        Ok(())
    }

    fn search(&mut self, query: &FulltextQuery) -> DBResult<Vec<FulltextSearchResult>> {
        if !self.indices.contains_key(&query.index_name) {
            return Err(DBError::FulltextIndex(
                FulltextIndexError::IndexNotFound(query.index_name.clone()),
            ));
        }

        let query_tokens = Self::tokenize(&query.query_string);
        let mut results: HashMap<String, f32> = HashMap::new();

        for token in query_tokens {
            if let Some(doc_ids) = self.inverted_index.get(&token) {
                for doc_id in doc_ids {
                    let parts: Vec<&str> = doc_id.split(':').collect();
                    if parts.len() >= 2 && parts[0] == query.index_name {
                        let doc_key = parts[1..].join(":");
                        let score = results.entry(doc_key.clone()).or_insert(0.0);
                        *score += 1.0;
                    }
                }
            }
        }

        let mut search_results: Vec<FulltextSearchResult> = results
            .into_iter()
            .map(|(id, score)| FulltextSearchResult::new(id, score))
            .filter(|r| {
                if let Some(docs) = self.documents.get(&query.index_name) {
                    docs.contains_key(&r.id)
                } else {
                    false
                }
            })
            .collect();

        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        search_results.truncate(query.limit);

        let end = std::cmp::min(query.offset + query.limit, search_results.len());
        Ok(search_results[query.offset..end].to_vec())
    }

    fn index_exists(&self, name: &str) -> bool {
        self.indices.contains_key(name)
    }

    fn get_index_config(&self, name: &str) -> Option<FulltextIndexConfig> {
        self.indices.get(name).cloned()
    }
    
    fn list_index_configs(&self) -> Vec<FulltextIndexConfig> {
        self.indices.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct FulltextIndexManager {
    engine: Arc<Mutex<dyn FulltextIndexEngine>>,
    config: Arc<RwLock<FulltextIndexConfig>>,
}

impl FulltextIndexManager {
    pub fn new(engine: Arc<Mutex<dyn FulltextIndexEngine>>) -> Self {
        Self {
            engine,
            config: Arc::new(RwLock::new(FulltextIndexConfig {
                name: String::new(),
                schema_type: FulltextSchemaType::Tag,
                schema_name: String::new(),
                fields: Vec::new(),
                analyzer: None,
                case_sensitive: false,
                created_at: chrono::Utc::now(),
            })),
        }
    }

    pub fn create_fulltext_index(
        &mut self,
        name: String,
        schema_type: FulltextSchemaType,
        schema_name: String,
        fields: Vec<String>,
        analyzer: Option<String>,
    ) -> DBResult<()> {
        let config = FulltextIndexConfig {
            name: name.clone(),
            schema_type,
            schema_name,
            fields,
            analyzer,
            case_sensitive: false,
            created_at: chrono::Utc::now(),
        };

        let mut engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        engine.create_index(&config)
    }

    pub fn drop_fulltext_index(&mut self, name: &str) -> DBResult<()> {
        let mut engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        engine.drop_index(name)
    }

    pub fn list_fulltext_indexes(&self) -> DBResult<Vec<FulltextIndexConfig>> {
        let engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        Ok(engine.list_index_configs())
    }

    pub fn index_document(&self, doc: FulltextDocument) -> DBResult<()> {
        let mut engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        engine.index_document(&doc)
    }

    pub fn delete_document(&self, _index_name: &str, doc_id: &str) -> DBResult<()> {
        let mut engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        engine.delete_document(doc_id)
    }

    pub fn search(&self, query: FulltextQuery) -> DBResult<Vec<FulltextSearchResult>> {
        let mut engine = self.engine.lock().map_err(|e| {
            DBError::FulltextIndex(FulltextIndexError::EngineError(e.to_string()))
        })?;

        engine.search(&query)
    }

    pub fn fulltext_index_exists(&self, name: &str) -> bool {
        let engine = self.engine.lock().unwrap();
        engine.index_exists(name)
    }
}

impl Default for FulltextIndexManager {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(SimpleFulltextEngine::new())))
    }
}

pub fn create_default_fulltext_manager() -> FulltextIndexManager {
    FulltextIndexManager::default()
}
