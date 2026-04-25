use crate::api::core::{IndexManager, IndexManagerConfig, IndexSchema};
use crate::error::Result;
use std::collections::HashMap;
use tantivy::schema::Value;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub entity_type: Option<String>,
    pub raw_name: Option<String>,
    pub keywords: Option<String>,
    pub file_path: Option<String>,
    pub module_name: Option<String>,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SearchResultWithHighlights {
    pub document_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub entity_type: Option<String>,
    pub raw_name: Option<String>,
    pub keywords: Option<String>,
    pub file_path: Option<String>,
    pub module_name: Option<String>,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
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

    pub fn add_document(&self, document_id: &str, title: &str, content: &str) -> Result<()> {
        use crate::api::core::document::add_document;

        let mut fields = HashMap::new();
        if !title.is_empty() {
            fields.insert("title".to_string(), title.to_string());
        }
        if !content.is_empty() {
            fields.insert("content".to_string(), content.to_string());
        }

        add_document(&self.manager, &self.schema, document_id, &fields)?;

        Ok(())
    }

    pub fn add_document_with_fields(
        &self,
        document_id: &str,
        fields: &HashMap<String, String>,
    ) -> Result<()> {
        use crate::api::core::document::add_document;

        add_document(&self.manager, &self.schema, document_id, fields)?;
        Ok(())
    }

    pub fn update_document(&self, document_id: &str, title: &str, content: &str) -> Result<()> {
        use crate::api::core::delete::delete_document;
        use crate::api::core::document::add_document;

        delete_document(&self.manager, &self.schema, document_id)?;

        let mut fields = HashMap::new();
        if !title.is_empty() {
            fields.insert("title".to_string(), title.to_string());
        }
        if !content.is_empty() {
            fields.insert("content".to_string(), content.to_string());
        }

        add_document(&self.manager, &self.schema, document_id, &fields)?;

        Ok(())
    }

    pub fn update_document_with_fields(
        &self,
        document_id: &str,
        fields: &HashMap<String, String>,
    ) -> Result<()> {
        use crate::api::core::delete::delete_document;
        use crate::api::core::document::add_document;

        delete_document(&self.manager, &self.schema, document_id)?;
        add_document(&self.manager, &self.schema, document_id, fields)?;
        Ok(())
    }

    pub fn delete_document(&self, document_id: &str) -> Result<()> {
        use crate::api::core::delete::delete_document;

        delete_document(&self.manager, &self.schema, document_id)?;

        Ok(())
    }

    fn extract_doc_fields(
        &self,
        doc: &tantivy::TantivyDocument,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let mut title: Option<String> = None;
        let mut content: Option<String> = None;
        let mut entity_type: Option<String> = None;
        let mut raw_name: Option<String> = None;
        let mut keywords: Option<String> = None;
        let mut file_path: Option<String> = None;
        let mut module_name: Option<String> = None;

        let schema = self.schema.schema();
        for (field, value) in doc.field_values() {
            let field_name = schema.get_field_name(field);
            match field_name {
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
                "entity_type" => {
                    if let Some(e) = value.as_str() {
                        entity_type = Some(e.to_string());
                    }
                }
                "raw_name" => {
                    if let Some(r) = value.as_str() {
                        raw_name = Some(r.to_string());
                    }
                }
                "keywords" => {
                    if let Some(k) = value.as_str() {
                        keywords = Some(k.to_string());
                    }
                }
                "file_path" => {
                    if let Some(f) = value.as_str() {
                        file_path = Some(f.to_string());
                    }
                }
                "module_name" => {
                    if let Some(m) = value.as_str() {
                        module_name = Some(m.to_string());
                    }
                }
                _ => {}
            }
        }

        (
            title,
            content,
            entity_type,
            raw_name,
            keywords,
            file_path,
            module_name,
        )
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        use tantivy::collector::TopDocs;
        use tantivy::query::QueryParser;

        let reader = self.manager.reader()?;
        let searcher = reader.searcher();

        let searchable_fields = self.schema.searchable_fields();
        let query_parser = QueryParser::for_index(self.manager.index(), searchable_fields);
        let query = query_parser
            .parse_query(query)
            .map_err(|e| crate::error::Bm25Error::InvalidQuery(e.to_string()))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let results = top_docs
            .into_iter()
            .filter_map(|(score, doc_address)| {
                let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address).ok()?;

                let mut document_id: Option<String> = None;

                let schema = self.schema.schema();
                for (field, value) in doc.field_values() {
                    let field_name = schema.get_field_name(field);
                    if field_name == "document_id" {
                        if let Some(id) = value.as_str() {
                            document_id = Some(id.to_string());
                        }
                    }
                }

                let (title, content, entity_type, raw_name, keywords, file_path, module_name) =
                    self.extract_doc_fields(&doc);

                document_id.map(|id| SearchResult {
                    document_id: id,
                    title,
                    content,
                    entity_type,
                    raw_name,
                    keywords,
                    file_path,
                    module_name,
                    score,
                    highlights: None,
                })
            })
            .collect();

        Ok(results)
    }

    pub fn search_with_highlights(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultWithHighlights>> {
        use tantivy::collector::TopDocs;
        use tantivy::query::QueryParser;
        use tantivy::snippet::SnippetGenerator;

        let reader = self.manager.reader()?;
        let searcher = reader.searcher();

        let searchable_fields = self.schema.searchable_fields();
        let query_parser = QueryParser::for_index(self.manager.index(), searchable_fields);
        let query = query_parser
            .parse_query(query)
            .map_err(|e| crate::error::Bm25Error::InvalidQuery(e.to_string()))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address).ok();
            if let Some(doc) = doc {
                let mut document_id: Option<String> = None;

                let schema = self.schema.schema();
                for (field, value) in doc.field_values() {
                    let field_name = schema.get_field_name(field);
                    if field_name == "document_id" {
                        if let Some(id) = value.as_str() {
                            document_id = Some(id.to_string());
                        }
                    }
                }

                let (title, content, entity_type, raw_name, keywords, file_path, module_name) =
                    self.extract_doc_fields(&doc);

                if let Some(id) = document_id {
                    let mut highlights = Vec::new();

                    if content.is_some() {
                        let mut snippet_gen =
                            SnippetGenerator::create(&searcher, &*query, self.schema.content)
                                .map_err(|e| crate::error::Bm25Error::TantivyError(e.into()))?;
                        snippet_gen.set_max_num_chars(100);

                        let snippet = snippet_gen.snippet_from_doc(&doc);
                        let highlighted = snippet.to_html();
                        if !highlighted.is_empty() {
                            highlights.push(highlighted);
                        }
                    }

                    results.push(SearchResultWithHighlights {
                        document_id: id,
                        title,
                        content,
                        entity_type,
                        raw_name,
                        keywords,
                        file_path,
                        module_name,
                        score,
                        highlights: if highlights.is_empty() {
                            None
                        } else {
                            Some(highlights)
                        },
                    });
                }
            }
        }

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
        self.manager.clear_reader_cache();

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

        index
            .add_document(
                "1",
                "Rust Programming",
                "Rust is a systems programming language",
            )
            .unwrap();
        index
            .add_document(
                "2",
                "Java Programming",
                "Java is an object-oriented language",
            )
            .unwrap();

        let results = index.search("Rust", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "1");
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_add_document_with_fields() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("test_fields");

        let index = Bm25Index::create(&index_path).unwrap();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "calculate_total".to_string());
        fields.insert(
            "content".to_string(),
            "Calculates the total price".to_string(),
        );
        fields.insert("entity_type".to_string(), "function".to_string());
        fields.insert("raw_name".to_string(), "calculate_total".to_string());
        fields.insert("keywords".to_string(), "calculate total price".to_string());
        fields.insert("file_path".to_string(), "src/calculator.rs".to_string());
        fields.insert("module_name".to_string(), "math".to_string());

        index.add_document_with_fields("1", &fields).unwrap();

        let results = index.search("calculate", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].entity_type, Some("function".to_string()));
        assert_eq!(results[0].raw_name, Some("calculate_total".to_string()));
    }

    #[test]
    fn test_chinese_search() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("test_chinese");

        let index = Bm25Index::create(&index_path).unwrap();

        let mut fields = HashMap::new();
        fields.insert("content".to_string(), "计算总价".to_string());
        fields.insert("raw_name".to_string(), "calculate_total".to_string());

        index.add_document_with_fields("1", &fields).unwrap();

        let results = index.search("计算", 10).unwrap();
        assert!(!results.is_empty(), "Should find results for Chinese query");
    }
}
