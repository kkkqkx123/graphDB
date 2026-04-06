use crate::error::Result;
use crate::api::core::{IndexManager, IndexManagerConfig, IndexSchema};
use std::collections::HashMap;
use tantivy::schema::Value;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub score: f32,
}

pub struct Bm25Index {
    manager: IndexManager,
    schema: IndexSchema,
}

impl Bm25Index {
    pub fn create<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let manager = IndexManager::create(&path)?;
        let schema = IndexSchema::new();
        Ok(Self { manager, schema })
    }

    pub fn create_with_config<P: AsRef<std::path::Path>>(
        path: P,
        config: IndexManagerConfig,
    ) -> Result<Self> {
        let manager = IndexManager::create_with_config(&path, config)?;
        let schema = IndexSchema::new();
        Ok(Self { manager, schema })
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let manager = IndexManager::open(&path)?;
        let schema = IndexSchema::new();
        Ok(Self { manager, schema })
    }

    pub fn open_with_config<P: AsRef<std::path::Path>>(
        path: P,
        config: IndexManagerConfig,
    ) -> Result<Self> {
        let manager = IndexManager::open_with_config(&path, config)?;
        let schema = IndexSchema::new();
        Ok(Self { manager, schema })
    }

    pub fn add_document(
        &self,
        document_id: &str,
        title: &str,
        content: &str,
    ) -> Result<()> {
        use crate::api::core::document::add_document;

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), title.to_string());
        fields.insert("content".to_string(), content.to_string());

        add_document(&self.manager, &self.schema, document_id, &fields)?;
        
        Ok(())
    }

    pub fn update_document(
        &self,
        document_id: &str,
        title: &str,
        content: &str,
    ) -> Result<()> {
        use crate::api::core::delete::delete_document;
        use crate::api::core::document::add_document;

        delete_document(&self.manager, &self.schema, document_id)?;

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), title.to_string());
        fields.insert("content".to_string(), content.to_string());

        add_document(&self.manager, &self.schema, document_id, &fields)?;
        
        Ok(())
    }

    pub fn delete_document(&self, document_id: &str) -> Result<()> {
        use crate::api::core::delete::delete_document;

        delete_document(&self.manager, &self.schema, document_id)?;
        
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        use tantivy::query::QueryParser;
        use tantivy::collector::TopDocs;

        let reader = self.manager.reader()?;
        let searcher = reader.searcher();
        
        let query_parser = QueryParser::for_index(
            self.manager.index(),
            vec![self.schema.title, self.schema.content],
        );
        let query = query_parser.parse_query(query).map_err(|e| {
            crate::error::Bm25Error::InvalidQuery(e.to_string())
        })?;
        
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        let results = top_docs
            .into_iter()
            .filter_map(|(score, doc_address)| {
                let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address).ok()?;
                
                let mut document_id: Option<String> = None;
                let mut title: Option<String> = None;
                let mut content: Option<String> = None;

                let schema = self.schema.schema();
                for (field, value) in doc.field_values() {
                    let field_name = schema.get_field_name(field);
                    match field_name {
                        "document_id" => {
                            if let Some(id) = value.as_str() {
                                document_id = Some(id.to_string());
                            }
                        }
                        "title" => {
                            if let Some(t) = value.as_str() {
                                title = Some(t.to_string());
                            }
                        }
                        "content" => {
                            if let Some(c) = value.as_str() {
                                content = Some(c.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                document_id.map(|id| SearchResult {
                    document_id: id,
                    title,
                    content,
                    score,
                })
            })
            .collect();

        Ok(results)
    }

    pub fn count(&self) -> Result<u64> {
        use crate::api::core::stats::get_stats;

        let stats = get_stats(&self.manager)?;
        
        Ok(stats.total_documents)
    }

    pub fn commit(&self) -> Result<()> {
        let mut writer = self.manager.writer()?;
        writer.commit()?;
        
        Ok(())
    }

    pub fn manager(&self) -> &IndexManager {
        &self.manager
    }

    pub fn schema(&self) -> &IndexSchema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_search() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let index = Bm25Index::create(&index_path).unwrap();
        
        index.add_document("1", "Rust Programming", "Rust is a systems programming language").unwrap();
        index.add_document("2", "Java Programming", "Java is an object-oriented language").unwrap();
        
        let results = index.search("Rust", 10).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "1");
        assert!(results[0].score > 0.0);
    }
}
