use crate::r#type::{SearchResults, EnrichedSearchResults, EnrichedSearchResult};

pub struct Enricher;

impl Enricher {
    pub fn apply_enrich(
        ids: &SearchResults,
        documents: &Vec<Option<serde_json::Value>>,
    ) -> EnrichedSearchResults {
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
        documents: &Vec<Option<serde_json::Value>>,
        metadata: &std::collections::HashMap<u64, serde_json::Value>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents.get(idx).cloned().flatten().unwrap_or(serde_json::json!({}));

            if let Some(meta) = metadata.get(&id) {
                if let serde_json::Value::Object(meta_obj) = meta {
                    for (key, value) in meta_obj {
                        doc_value[key] = value.clone();
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

    pub fn enrich_with_scores(
        ids: &SearchResults,
        documents: &Vec<Option<serde_json::Value>>,
        scores: &std::collections::HashMap<u64, f64>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let mut doc_value = documents.get(idx).cloned().flatten().unwrap_or(serde_json::json!({}));

            if let Some(&score) = scores.get(&id) {
                doc_value["_score"] = serde_json::json!(score);
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
        documents: &Vec<Option<serde_json::Value>>,
        highlights: &std::collections::HashMap<u64, std::collections::HashMap<String, Vec<String>>>,
    ) -> EnrichedSearchResults {
        if ids.is_empty() {
            return Vec::new();
        }

        let mut enriched: EnrichedSearchResults = Vec::new();

        for (idx, &id) in ids.iter().enumerate() {
            let doc = documents.get(idx).cloned().flatten();

            let highlight = if let Some(field_highlights) = highlights.get(&id) {
                let combined: Vec<String> = field_highlights.values()
                    .flatten()
                    .cloned()
                    .collect();
                if !combined.is_empty() {
                    Some(combined.join("..."))
                } else {
                    None
                }
            } else {
                None
            };

            enriched.push(EnrichedSearchResult {
                id,
                doc,
                highlight,
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
        assert_eq!(enriched[0].doc.as_ref().unwrap()["name"], "test1");
        assert_eq!(enriched[1].id, 1);
        assert_eq!(enriched[1].doc.as_ref().unwrap()["name"], "test2");
        assert_eq!(enriched[2].id, 2);
        assert_eq!(enriched[2].doc.as_ref().unwrap()["name"], "test3");
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
        assert_eq!(enriched[0].doc.as_ref().unwrap()["source"], "db1");
        assert_eq!(enriched[1].doc.as_ref().unwrap()["source"], "db2");
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
        assert_eq!(enriched[0].doc.as_ref().unwrap()["_score"], 0.95);
        assert_eq!(enriched[1].doc.as_ref().unwrap()["_score"], 0.85);
        assert_eq!(enriched[2].doc.as_ref().unwrap()["_score"], 0.75);
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
        field_highlights.insert("content".to_string(), vec!["<em>hello</em> world".to_string()]);
        highlights.insert(0, field_highlights);

        let enriched = Enricher::apply_highlight(&ids, &documents, &highlights);

        assert_eq!(enriched.len(), 2);
        assert!(enriched[0].highlight.is_some());
        assert!(enriched[1].highlight.is_none());
    }
}
