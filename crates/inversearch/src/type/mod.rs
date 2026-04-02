use serde::{Deserialize, Serialize};

pub type DocId = u64;
pub type SearchResults = Vec<DocId>;
pub type IntermediateSearchResults = Vec<Vec<DocId>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOptions {
    pub preset: Option<String>,
    pub context: Option<ContextOptions>,
    pub encoder: Option<EncoderOptions>,
    pub resolution: Option<usize>,
    pub tokenize: Option<String>,
    pub fastupdate: Option<bool>,
    pub keystore: Option<usize>,
    pub rtl: Option<bool>,
    pub cache: Option<usize>,
    pub commit: Option<bool>,
    pub priority: Option<usize>,
}

impl Default for IndexOptions {
    fn default() -> Self {
        IndexOptions {
            preset: None,
            context: None,
            encoder: None,
            resolution: Some(9),
            tokenize: Some("strict".to_string()),
            fastupdate: Some(false),
            keystore: None,
            rtl: Some(false),
            cache: None,
            commit: Some(true),
            priority: Some(4),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOptions {
    pub depth: Option<usize>,
    pub bidirectional: Option<bool>,
    pub resolution: Option<usize>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        ContextOptions {
            depth: Some(1),
            bidirectional: Some(true),
            resolution: Some(3),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub resolution: Option<usize>,
    pub context: Option<bool>,
    pub suggest: Option<bool>,
    pub resolve: Option<bool>,
    pub enrich: Option<bool>,
    pub cache: Option<bool>,
    pub tag: Option<Vec<TagOption>>,
    pub field: Option<Vec<FieldOption>>,
    pub pluck: Option<String>,
    pub merge: Option<bool>,
    pub boost: Option<i32>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            query: None,
            limit: Some(100),
            offset: Some(0),
            resolution: None,
            context: None,
            suggest: Some(false),
            resolve: Some(true),
            enrich: Some(false),
            cache: Some(false),
            tag: None,
            field: None,
            pluck: None,
            merge: Some(false),
            boost: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOption {
    pub field: String,
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub suggest: Option<bool>,
    pub enrich: Option<bool>,
    pub cache: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagOption {
    pub field: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderOptions {
    pub rtl: Option<bool>,
    pub dedupe: Option<bool>,
    pub split: Option<String>,
    pub numeric: Option<bool>,
    pub normalize: Option<bool>,
    pub prepare: Option<String>,
    pub finalize: Option<String>,
    pub filter: Option<Vec<String>>,
    pub matcher: Option<std::collections::HashMap<String, String>>,
    pub mapper: Option<std::collections::HashMap<char, char>>,
    pub stemmer: Option<std::collections::HashMap<String, String>>,
    pub replacer: Option<Vec<(String, String)>>,
    pub minlength: Option<usize>,
    pub maxlength: Option<usize>,
    pub cache: Option<bool>,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        EncoderOptions {
            rtl: Some(false),
            dedupe: Some(true),
            split: None,
            numeric: Some(true),
            normalize: Some(true),
            prepare: None,
            finalize: None,
            filter: None,
            matcher: None,
            mapper: None,
            stemmer: None,
            replacer: None,
            minlength: Some(1),
            maxlength: Some(1024),
            cache: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOptions {
    pub template: String,
    pub boundary: Option<HighlightBoundaryOptions>,
    pub clip: Option<bool>,
    pub merge: Option<bool>,
    pub ellipsis: Option<HighlightEllipsisOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightBoundaryOptions {
    pub before: Option<i32>,
    pub after: Option<i32>,
    pub total: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightEllipsisOptions {
    pub template: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedSearchResult {
    pub id: DocId,
    pub doc: Option<serde_json::Value>,
    pub highlight: Option<String>,
}

pub type EnrichedSearchResults = Vec<EnrichedSearchResult>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    pub field: Option<String>,
    pub tag: Option<String>,
    pub result: SearchResults,
}

pub type DocumentSearchResults = Vec<DocumentSearchResult>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedDocumentSearchResult {
    pub field: Option<String>,
    pub tag: Option<String>,
    pub result: EnrichedSearchResults,
}

pub type EnrichedDocumentSearchResults = Vec<EnrichedDocumentSearchResult>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedDocumentSearchEntry {
    pub id: DocId,
    pub doc: Option<serde_json::Value>,
    pub field: Option<Vec<String>>,
    pub tag: Option<Vec<String>>,
    pub highlight: Option<std::collections::HashMap<String, String>>,
}

pub type MergedDocumentSearchResults = Vec<MergedDocumentSearchEntry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_options_default() {
        let opts = IndexOptions::default();
        assert_eq!(opts.resolution, Some(9));
        assert_eq!(opts.tokenize, Some("strict".to_string()));
    }

    #[test]
    fn test_search_options_default() {
        let opts = SearchOptions::default();
        assert_eq!(opts.limit, Some(100));
        assert_eq!(opts.offset, Some(0));
    }

    #[test]
    fn test_encoder_options_default() {
        let opts = EncoderOptions::default();
        assert_eq!(opts.dedupe, Some(true));
        assert_eq!(opts.numeric, Some(true));
    }
}
