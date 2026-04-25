use crate::api::core::{IndexManager, IndexSchema};
use crate::error::Result;
use tantivy::query::QueryParser;
use tantivy::schema::Value;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document_id: String,
    pub score: f32,
    pub tag_name: String,
    pub field_name: String,
    pub content: String,
    pub highlights: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub highlight: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            offset: 0,
            highlight: false,
        }
    }
}

pub fn search(
    manager: &IndexManager,
    schema: &IndexSchema,
    query_text: &str,
    options: &SearchOptions,
) -> Result<(Vec<SearchResult>, f32)> {
    let reader = manager.reader()?;
    let searcher = reader.searcher();

    let query = parse_query(query_text, manager, schema)?;

    let limit = options.limit + options.offset;
    let top_docs = tantivy::collector::TopDocs::with_limit(limit);

    let results = searcher.search(&query, &top_docs)?;

    let mut search_results = Vec::new();
    let mut max_score = 0.0f32;

    for (score, doc_address) in results.into_iter().skip(options.offset) {
        if score > max_score {
            max_score = score;
        }

        let doc = searcher.doc(doc_address)?;
        let document_id = extract_field_value(&doc, schema.document_id);
        let tag_name = extract_field_value(&doc, schema.tag_name);
        let field_name = extract_field_value(&doc, schema.field_name);
        let content = extract_field_value(&doc, schema.content);

        let highlights = if options.highlight && !content.is_empty() {
            Some(vec![highlight_text(&content, query_text)])
        } else {
            None
        };

        search_results.push(SearchResult {
            document_id,
            score,
            tag_name,
            field_name,
            content,
            highlights,
        });
    }

    Ok((search_results, max_score))
}

fn parse_query(
    query_text: &str,
    manager: &IndexManager,
    schema: &IndexSchema,
) -> Result<Box<dyn tantivy::query::Query>> {
    let searchable_fields = schema.searchable_fields();
    let query_parser = QueryParser::for_index(manager.index(), searchable_fields.clone());

    let query = query_parser
        .parse_query(query_text)
        .map_err(|e| crate::error::Bm25Error::InvalidQuery(e.to_string()))?;

    Ok(query)
}

fn extract_field_value(
    doc: &tantivy::schema::TantivyDocument,
    field: tantivy::schema::Field,
) -> String {
    if let Some(value) = doc.get_first(field) {
        return compact_value_to_string(&value);
    }
    String::new()
}

fn compact_value_to_string(value: &tantivy::schema::document::CompactDocValue) -> String {
    value.as_str().map(|s| s.to_string()).unwrap_or_default()
}

fn highlight_text(_text: &str, _query: &str) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 10);
        assert_eq!(options.offset, 0);
        assert!(!options.highlight);
    }

    #[test]
    fn test_search_basic() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_search");
        let manager = IndexManager::create(&path)?;
        let schema = IndexSchema::new();

        let mut writer = manager.writer()?;
        let doc = schema.to_document("1", &{
            let mut fields = HashMap::new();
            fields.insert("tag_name".to_string(), "person".to_string());
            fields.insert("field_name".to_string(), "description".to_string());
            fields.insert(
                "content".to_string(),
                "Rust is a systems programming language".to_string(),
            );
            fields
        });
        writer.add_document(doc)?;
        writer.commit()?;
        manager.clear_reader_cache();

        let options = SearchOptions::default();
        let (results, _) = search(&manager, &schema, "Rust", &options)?;
        assert!(!results.is_empty());

        Ok(())
    }
}
