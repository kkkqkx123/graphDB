use crate::api::core::{IndexManager, IndexSchema};
use crate::error::Result;
use std::collections::HashMap;
use tantivy::query::QueryParser;
use tantivy::schema::Value;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document_id: String,
    pub score: f32,
    pub fields: HashMap<String, String>,
    pub highlights: HashMap<String, String>,
}

#[derive(Debug)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub field_weights: HashMap<String, f32>,
    pub highlight: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        let mut field_weights = HashMap::new();
        field_weights.insert("raw_name".to_string(), 3.0);
        field_weights.insert("keywords".to_string(), 2.0);
        field_weights.insert("title".to_string(), 1.5);
        field_weights.insert("content".to_string(), 1.0);
        Self {
            limit: 10,
            offset: 0,
            field_weights,
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

    let query = parse_query(query_text, manager, schema, &options.field_weights)?;

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

        let mut fields = HashMap::new();
        let title_value = extract_field_value(&doc, schema.title);
        let content_value = extract_field_value(&doc, schema.content);
        let entity_type_value = extract_field_value(&doc, schema.entity_type);
        let raw_name_value = extract_field_value(&doc, schema.raw_name);
        let keywords_value = extract_field_value(&doc, schema.keywords);
        let file_path_value = extract_field_value(&doc, schema.file_path);
        let module_name_value = extract_field_value(&doc, schema.module_name);

        fields.insert("title".to_string(), title_value.clone());
        fields.insert("content".to_string(), content_value.clone());
        fields.insert("entity_type".to_string(), entity_type_value);
        fields.insert("raw_name".to_string(), raw_name_value.clone());
        fields.insert("keywords".to_string(), keywords_value.clone());
        fields.insert("file_path".to_string(), file_path_value);
        fields.insert("module_name".to_string(), module_name_value);

        let mut highlights = HashMap::new();
        if options.highlight {
            if !title_value.is_empty() {
                highlights.insert(
                    "title".to_string(),
                    highlight_text(&title_value, query_text),
                );
            }
            if !content_value.is_empty() {
                highlights.insert(
                    "content".to_string(),
                    highlight_text(&content_value, query_text),
                );
            }
            if !raw_name_value.is_empty() {
                highlights.insert(
                    "raw_name".to_string(),
                    highlight_text(&raw_name_value, query_text),
                );
            }
            if !keywords_value.is_empty() {
                highlights.insert(
                    "keywords".to_string(),
                    highlight_text(&keywords_value, query_text),
                );
            }
        }

        search_results.push(SearchResult {
            document_id,
            score,
            fields,
            highlights,
        });
    }

    Ok((search_results, max_score))
}

fn parse_query(
    query_text: &str,
    manager: &IndexManager,
    schema: &IndexSchema,
    field_weights: &HashMap<String, f32>,
) -> Result<Box<dyn tantivy::query::Query>> {
    let searchable_fields = schema.searchable_fields();
    let mut query_parser = QueryParser::for_index(manager.index(), searchable_fields.clone());

    for field in &searchable_fields {
        let field_name = schema.schema().get_field_name(*field).to_string();
        if let Some(weight) = field_weights.get(&field_name) {
            query_parser.set_field_boost(*field, *weight);
        }
    }

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
    use tempfile::tempdir;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 10);
        assert_eq!(options.offset, 0);
        assert!(options.field_weights.contains_key("raw_name"));
        assert!(options.field_weights.contains_key("content"));
        assert!(
            *options
                .field_weights
                .get("raw_name")
                .expect("raw_name weight")
                > *options
                    .field_weights
                    .get("content")
                    .expect("content weight")
        );
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
            fields.insert("title".to_string(), "Rust Programming".to_string());
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
