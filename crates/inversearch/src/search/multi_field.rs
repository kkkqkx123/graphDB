//! Multi-Field Search
//!
//! Provide a unified search interface across multiple fields

use crate::error::Result;
use crate::{Document, SearchOptions, SearchResult};

/// Multi-field search configuration
#[derive(Clone)]
pub struct MultiFieldSearchConfig<'a> {
    document: &'a Document,
    fields: Vec<String>,
    weights: Vec<f32>,
    boost: std::collections::HashMap<String, f32>,
    limit: usize,
    offset: usize,
}

impl<'a> MultiFieldSearchConfig<'a> {
    /// Creating a new configuration
    pub fn new(document: &'a Document) -> Self {
        MultiFieldSearchConfig {
            document,
            fields: Vec::new(),
            weights: Vec::new(),
            boost: std::collections::HashMap::new(),
            limit: 100,
            offset: 0,
        }
    }

    /// Adding Fields
    pub fn add_field(mut self, name: &str) -> Self {
        self.fields.push(name.to_string());
        self.weights.push(1.0);
        self
    }

    /// Adding fields with weights
    pub fn add_field_with_weight(mut self, name: &str, weight: f32) -> Self {
        self.fields.push(name.to_string());
        self.weights.push(weight);
        self
    }

    /// Setting field weights
    pub fn set_weight(mut self, field: &str, weight: f32) -> Self {
        if let Some(idx) = self.fields.iter().position(|f| f == field) {
            self.weights[idx] = weight;
        }
        self
    }

    /// Setting up boost
    pub fn set_boost(mut self, field: &str, boost: f32) -> Self {
        self.boost.insert(field.to_string(), boost);
        self
    }

    /// Setting Limits
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Setting the offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Perform a search
    pub fn search(&self, query: &str) -> Result<SearchResult> {
        if query.is_empty() {
            return Ok(SearchResult {
                results: Vec::new(),
                total: 0,
                query: String::new(),
            });
        }

        let mut all_results = std::collections::HashMap::new();
        let mut field_scores = std::collections::HashMap::new();

        for (idx, field_name) in self.fields.iter().enumerate() {
            let weight = self.weights.get(idx).copied().unwrap_or(1.0);
            let boost = self.boost.get(field_name).copied().unwrap_or(1.0);
            let field_weight = weight * boost;

            if let Some(field) = self.document.field(field_name) {
                let search_opts = SearchOptions {
                    query: Some(query.to_string()),
                    limit: Some(self.limit * 2),
                    offset: Some(0),
                    resolve: Some(false),
                    ..Default::default()
                };

                let result = crate::search::search(field.index(), &search_opts)?;

                for &doc_id in &result.results {
                    *field_scores.entry(doc_id).or_insert(0.0) += field_weight;
                    *all_results.entry(doc_id).or_insert(0usize) += 1;
                }
            }
        }

        let mut scored: Vec<(u64, f32)> = field_scores.into_iter().collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let total = scored.len();
        let final_results: Vec<u64> = scored
            .into_iter()
            .skip(self.offset)
            .take(self.limit)
            .map(|(id, _)| id)
            .collect();

        Ok(SearchResult {
            results: final_results,
            total,
            query: query.to_string(),
        })
    }
}

/// Convenient multi-field search function
pub fn multi_field_search(
    document: &Document,
    query: &str,
    fields: &[&str],
) -> Result<SearchResult> {
    let config = MultiFieldSearchConfig::new(document);

    let mut config = config;
    for &field in fields {
        config = config.add_field(field);
    }

    config.search(query)
}

/// Multi-field search with weight configuration
pub fn multi_field_search_with_weights(
    document: &Document,
    query: &str,
    fields: &[(&str, f32)],
) -> Result<SearchResult> {
    let config = MultiFieldSearchConfig::new(document);

    let mut config = config;
    for &(field, weight) in fields {
        config = config.add_field_with_weight(field, weight);
    }

    config.search(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, DocumentConfig, FieldConfig};
    use serde_json::json;

    fn create_test_document() -> Document {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"));

        let mut doc = Document::new(config).unwrap();

        doc.add(
            1,
            &json!({"title": "Rust Programming", "content": "Learn Rust today"}),
        )
        .unwrap();
        doc.add(
            2,
            &json!({"title": "JavaScript Guide", "content": "JavaScript tutorial"}),
        )
        .unwrap();
        doc.add(
            3,
            &json!({"title": "Rust vs Go", "content": "Comparing Rust and Go"}),
        )
        .unwrap();

        doc
    }

    #[test]
    fn test_multi_field_search() {
        let doc = create_test_document();

        let result = multi_field_search(&doc, "Rust", &["title", "content"]).unwrap();

        assert!(result.results.contains(&1));
        assert!(result.results.contains(&3));
    }

    #[test]
    fn test_multi_field_search_with_weights() {
        let doc = create_test_document();

        let result =
            multi_field_search_with_weights(&doc, "Rust", &[("title", 2.0), ("content", 1.0)])
                .unwrap();

        assert!(result.results.contains(&1));
        assert!(result.results.contains(&3));
    }

    #[test]
    fn test_multi_field_config() {
        let doc = create_test_document();

        let config = MultiFieldSearchConfig::new(&doc)
            .add_field("title")
            .add_field_with_weight("content", 0.5)
            .set_boost("title", 2.0)
            .limit(10)
            .offset(0);

        let result = config.search("Rust").unwrap();

        assert!(!result.results.is_empty());
    }

    #[test]
    fn test_multi_field_empty_query() {
        let doc = create_test_document();

        let result = multi_field_search(&doc, "", &["title"]).unwrap();

        assert!(result.results.is_empty());
        assert_eq!(result.total, 0);
    }

    #[test]
    fn test_multi_field_limit() {
        let doc = create_test_document();

        let result = multi_field_search(&doc, "Rust", &["title", "content"]).unwrap();

        // There should be 2 documents found that contain Rust
        assert_eq!(result.results.len(), 2);
    }
}
