use crate::r#type::{EnrichedSearchResult, EnrichedSearchResults, SearchResults};
use serde_json::Value;
use std::collections::HashMap;

// Type aliases for complex types
type TagTransformFn = Box<dyn Fn(&Value) -> Value + Send + Sync>;
type TagFilterFn = Box<dyn Fn(&Value) -> bool + Send + Sync>;

/// 字段选择配置
#[derive(Debug, Clone)]
pub struct FieldSelector {
    pub field_path: String,
    pub alias: Option<String>,
    pub default_value: Option<Value>,
}

impl FieldSelector {
    pub fn new(field_path: &str) -> Self {
        FieldSelector {
            field_path: field_path.to_string(),
            alias: None,
            default_value: None,
        }
    }

    pub fn with_alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default_value = Some(default);
        self
    }
}

/// 标签整合配置
pub struct TagIntegrationConfig {
    pub tag_field: String,
    pub transform_fn: Option<TagTransformFn>,
    pub filter_fn: Option<TagFilterFn>,
}

impl Clone for TagIntegrationConfig {
    fn clone(&self) -> Self {
        TagIntegrationConfig {
            tag_field: self.tag_field.clone(),
            transform_fn: None,
            filter_fn: None,
        }
    }
}

impl std::fmt::Debug for TagIntegrationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagIntegrationConfig")
            .field("tag_field", &self.tag_field)
            .field("has_transform", &self.transform_fn.is_some())
            .field("has_filter", &self.filter_fn.is_some())
            .finish()
    }
}

impl TagIntegrationConfig {
    pub fn new(tag_field: &str) -> Self {
        TagIntegrationConfig {
            tag_field: tag_field.to_string(),
            transform_fn: None,
            filter_fn: None,
        }
    }

    pub fn with_transform<F>(mut self, transform: F) -> Self
    where
        F: Fn(&Value) -> Value + 'static + Send + Sync,
    {
        self.transform_fn = Some(Box::new(transform));
        self
    }

    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&Value) -> bool + 'static + Send + Sync,
    {
        self.filter_fn = Some(Box::new(filter));
        self
    }
}

/// 高亮配置
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub fields: Vec<String>,
    pub before_marker: String,
    pub after_marker: String,
    pub fragment_count: usize,
    pub fragment_length: usize,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        HighlightConfig {
            fields: Vec::new(),
            before_marker: "<em>".to_string(),
            after_marker: "</em>".to_string(),
            fragment_count: 3,
            fragment_length: 150,
        }
    }
}

impl HighlightConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_markers(mut self, before: &str, after: &str) -> Self {
        self.before_marker = before.to_string();
        self.after_marker = after.to_string();
        self
    }

    pub fn with_fragments(mut self, count: usize, length: usize) -> Self {
        self.fragment_count = count;
        self.fragment_length = length;
        self
    }
}

/// 元数据配置
#[derive(Debug, Clone)]
pub enum MetadataSource {
    Calculated(String),
    External(String),
    Statistical(String),
}

/// 文档丰富化器
#[derive(Default)]
pub struct Enricher {
    field_selectors: Vec<FieldSelector>,
    tag_configs: Vec<TagIntegrationConfig>,
    highlight_config: Option<HighlightConfig>,
    metadata_sources: Vec<MetadataSource>,
}

impl Enricher {
    pub fn new() -> Self {
        Enricher {
            field_selectors: Vec::new(),
            tag_configs: Vec::new(),
            highlight_config: None,
            metadata_sources: Vec::new(),
        }
    }

    pub fn with_field_selector(mut self, selector: FieldSelector) -> Self {
        self.field_selectors.push(selector);
        self
    }

    pub fn with_tag_config(mut self, config: TagIntegrationConfig) -> Self {
        self.tag_configs.push(config);
        self
    }

    pub fn with_highlight_config(mut self, config: HighlightConfig) -> Self {
        self.highlight_config = Some(config);
        self
    }

    pub fn with_metadata_source(mut self, source: MetadataSource) -> Self {
        self.metadata_sources.push(source);
        self
    }

    pub fn apply_enrich(ids: &SearchResults, documents: &[Option<Value>]) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let doc = documents.get(idx).cloned().flatten();
            enriched.push(EnrichedSearchResult {
                id,
                doc,
                highlight: None,
            });
        }

        enriched
    }

    pub fn enrich_with_metadata(
        ids: &SearchResults,
        documents: &[Option<Value>],
        metadata: &HashMap<u64, Value>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents
                .get(idx)
                .cloned()
                .flatten()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            if let Some(Value::Object(meta_obj)) = metadata.get(&id) {
                for (key, value) in meta_obj {
                    doc_value[key] = value.clone();
                }
            }

            enriched.push(EnrichedSearchResult {
                id,
                doc: Some(doc_value),
                highlight: None,
            });
        }

        enriched
    }

    pub fn enrich_with_scores(
        ids: &SearchResults,
        documents: &[Option<Value>],
        scores: &HashMap<u64, f64>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents
                .get(idx)
                .cloned()
                .flatten()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            if let Some(&score) = scores.get(&id) {
                if let Some(num) = serde_json::Number::from_f64(score) {
                    doc_value["_score"] = Value::Number(num);
                }
            }

            enriched.push(EnrichedSearchResult {
                id,
                doc: Some(doc_value),
                highlight: None,
            });
        }

        enriched
    }

    pub fn apply_highlight(
        ids: &SearchResults,
        documents: &[Option<Value>],
        highlights: &HashMap<u64, HashMap<String, Vec<String>>>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let doc = documents.get(idx).cloned().flatten();

            let highlight = if let Some(field_highlights) = highlights.get(&id) {
                let combined: Vec<String> = field_highlights.values().flatten().cloned().collect();
                if !combined.is_empty() {
                    Some(combined.join("..."))
                } else {
                    None
                }
            } else {
                None
            };

            enriched.push(EnrichedSearchResult { id, doc, highlight });
        }

        enriched
    }

    pub fn apply_field_selection(
        ids: &SearchResults,
        documents: &[Option<Value>],
        selectors: &[FieldSelector],
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let doc = documents.get(idx).cloned().flatten();

            if let Some(doc_value) = doc {
                let mut selected = Value::Object(serde_json::Map::new());

                for selector in selectors {
                    if let Some(field_name) = selector.alias.as_ref() {
                        if let Some(value) = doc_value.get(&selector.field_path) {
                            selected[field_name] = value.clone();
                        } else if let Some(default_val) = &selector.default_value {
                            selected[field_name] = default_val.clone();
                        }
                    } else {
                        if let Some(value) = doc_value.get(&selector.field_path) {
                            selected[&selector.field_path] = value.clone();
                        } else if let Some(default_val) = &selector.default_value {
                            selected[&selector.field_path] = default_val.clone();
                        }
                    }
                }

                enriched.push(EnrichedSearchResult {
                    id,
                    doc: Some(selected),
                    highlight: None,
                });
            } else {
                enriched.push(EnrichedSearchResult {
                    id,
                    doc: None,
                    highlight: None,
                });
            }
        }

        enriched
    }

    pub fn apply_tag_integration(
        ids: &SearchResults,
        documents: &[Option<Value>],
        tag_configs: &[TagIntegrationConfig],
        tag_data: &HashMap<u64, Vec<(String, Value)>>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents
                .get(idx)
                .cloned()
                .flatten()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            if let Some(tags) = tag_data.get(&id) {
                for config in tag_configs {
                    for (tag_field, tag_value) in tags {
                        if tag_field.as_str() == config.tag_field {
                            if let Some(ref filter) = config.filter_fn {
                                if !filter(tag_value) {
                                    continue;
                                }
                            }

                            let final_value = if let Some(ref transform) = config.transform_fn {
                                transform(tag_value)
                            } else {
                                tag_value.clone()
                            };

                            doc_value[&tag_field] = final_value;
                        }
                    }
                }
            }

            enriched.push(EnrichedSearchResult {
                id,
                doc: Some(doc_value),
                highlight: None,
            });
        }

        enriched
    }

    pub fn apply_metadata_enrichment(
        ids: &SearchResults,
        documents: &[Option<Value>],
        metadata_sources: &[MetadataSource],
        external_data: &HashMap<String, Value>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents
                .get(idx)
                .cloned()
                .flatten()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            for source in metadata_sources {
                match source {
                    MetadataSource::Calculated(field_name) => {
                        if let Value::Object(obj) = &doc_value {
                            let field_count = obj.len();
                            doc_value[field_name] =
                                Value::Number(serde_json::Number::from(field_count));
                        }
                    }
                    MetadataSource::External(field_name) => {
                        if let Some(external_val) = external_data.get(&id.to_string()) {
                            doc_value[field_name] = external_val.clone();
                        }
                    }
                    MetadataSource::Statistical(field_name) => {
                        doc_value[field_name] = Value::Number(serde_json::Number::from(idx as i64));
                    }
                }
            }

            enriched.push(EnrichedSearchResult {
                id,
                doc: Some(doc_value),
                highlight: None,
            });
        }

        enriched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_apply_enrich_basic() {
        let ids = vec![0, 1, 2];
        let documents = vec![
            Some(json!({"id": 1, "name": "test1"})),
            Some(json!({"id": 2, "name": "test2"})),
            Some(json!({"id": 3, "name": "test3"})),
        ];

        let enriched = Enricher::apply_enrich(&ids, &documents);

        assert_eq!(enriched.len(), 3);
        assert_eq!(enriched[0].id, 0);
        assert_eq!(
            enriched[0].doc.as_ref().expect("doc should exist")["name"],
            "test1"
        );
        assert_eq!(enriched[1].id, 1);
        assert_eq!(
            enriched[1].doc.as_ref().expect("doc should exist")["name"],
            "test2"
        );
        assert_eq!(enriched[2].id, 2);
        assert_eq!(
            enriched[2].doc.as_ref().expect("doc should exist")["name"],
            "test3"
        );
    }

    #[test]
    fn test_apply_enrich_empty() {
        let ids: Vec<u64> = Vec::new();
        let documents: Vec<Option<serde_json::Value>> = Vec::new();

        let enriched = Enricher::apply_enrich(&ids, &documents);

        assert!(enriched.is_empty());
    }

    #[test]
    fn test_enrich_with_metadata() {
        let ids = vec![0, 1];
        let documents = vec![
            Some(json!({"id": 1, "name": "test1"})),
            Some(json!({"id": 2, "name": "test2"})),
        ];

        let mut metadata = std::collections::HashMap::new();
        metadata.insert(0, json!({"source": "db1"}));
        metadata.insert(1, json!({"source": "db2"}));

        let enriched = Enricher::enrich_with_metadata(&ids, &documents, &metadata);

        assert_eq!(enriched.len(), 2);
        assert_eq!(
            enriched[0].doc.as_ref().expect("doc should exist")["source"],
            "db1"
        );
        assert_eq!(
            enriched[1].doc.as_ref().expect("doc should exist")["source"],
            "db2"
        );
    }

    #[test]
    fn test_enrich_with_scores() {
        let ids = vec![0, 1, 2];
        let documents = vec![
            Some(json!({"id": 1, "name": "test1"})),
            Some(json!({"id": 2, "name": "test2"})),
            Some(json!({"id": 3, "name": "test3"})),
        ];

        let mut scores = std::collections::HashMap::new();
        scores.insert(0, 0.95);
        scores.insert(1, 0.85);
        scores.insert(2, 0.75);

        let enriched = Enricher::enrich_with_scores(&ids, &documents, &scores);

        assert_eq!(enriched.len(), 3);
        assert_eq!(
            enriched[0].doc.as_ref().expect("doc should exist")["_score"],
            0.95
        );
        assert_eq!(
            enriched[1].doc.as_ref().expect("doc should exist")["_score"],
            0.85
        );
        assert_eq!(
            enriched[2].doc.as_ref().expect("doc should exist")["_score"],
            0.75
        );
    }

    #[test]
    fn test_apply_highlight() {
        let ids = vec![0, 1];
        let documents = vec![
            Some(json!({"id": 1, "content": "hello world"})),
            Some(json!({"id": 2, "content": "foo bar"})),
        ];

        let mut highlights = std::collections::HashMap::new();
        let mut field_highlights = std::collections::HashMap::new();
        field_highlights.insert(
            "content".to_string(),
            vec!["<em>hello</em> world".to_string()],
        );
        highlights.insert(0, field_highlights);

        let enriched = Enricher::apply_highlight(&ids, &documents, &highlights);

        assert_eq!(enriched.len(), 2);
        assert!(enriched[0].highlight.is_some());
        assert!(enriched[1].highlight.is_none());
    }
}
