use crate::index::{IndexManager, IndexSchema};
use crate::error::Result;
use std::collections::HashMap;

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
        Self {
            limit: 10,
            offset: 0,
            field_weights: HashMap::new(),
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

    let query = parse_query(query_text, schema)?;

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
        fields.insert("title".to_string(), title_value.clone());
        fields.insert("content".to_string(), content_value.clone());

        let mut highlights = HashMap::new();
        if options.highlight {
            highlights.insert("title".to_string(), highlight_text(&title_value, query_text));
            highlights.insert("content".to_string(), highlight_text(&content_value, query_text));
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
    schema: &IndexSchema,
) -> Result<Box<dyn tantivy::query::Query>> {
    let terms: Vec<&str> = query_text.split_whitespace().collect();

    if terms.is_empty() {
        let empty_query = tantivy::query::EmptyQuery {};
        return Ok(Box::new(empty_query));
    }

    let mut clauses: Vec<(tantivy::query::Occur, Box<dyn tantivy::query::Query>)> = Vec::new();

    for term in &terms {
        let term_text = term.to_lowercase();

        let title_term = tantivy::Term::from_field_text(schema.title, &term_text);
        let title_query: Box<dyn tantivy::query::Query> = Box::new(tantivy::query::TermQuery::new(
            title_term,
            tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
        ));

        let content_term = tantivy::Term::from_field_text(schema.content, &term_text);
        let content_query: Box<dyn tantivy::query::Query> = Box::new(tantivy::query::TermQuery::new(
            content_term,
            tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
        ));

        let term_query = tantivy::query::BooleanQuery::new(vec![
            (tantivy::query::Occur::Should, title_query),
            (tantivy::query::Occur::Should, content_query),
        ]);

        clauses.push((tantivy::query::Occur::Should, Box::new(term_query)));
    }

    let boolean_query = tantivy::query::BooleanQuery::new(clauses);
    Ok(Box::new(boolean_query))
}

fn extract_field_value(doc: &tantivy::schema::TantivyDocument, field: tantivy::schema::Field) -> String {
    if let Some(value) = doc.get_first(field) {
        return value_to_string(value);
    }
    String::new()
}

fn value_to_string(value: &tantivy::schema::document::OwnedValue) -> String {
    match value {
        tantivy::schema::document::OwnedValue::Str(s) => s.clone(),
        _ => String::new(),
    }
}

fn highlight_text(_text: &String, _query: &str) -> String {
    String::new()
}
