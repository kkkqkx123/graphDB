use crate::encoder::Encoder;
use crate::error::Result;
use crate::DocId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub template: String,
    pub markup_open: String,
    pub markup_close: String,
    pub boundary: Option<HighlightBoundaryOptions>,
    pub clip: bool,
    pub merge: Option<String>,
    pub ellipsis: String,
    pub ellipsis_markup_length: usize,
}

impl HighlightConfig {
    pub fn from_options(options: &HighlightOptions) -> Result<Self> {
        let template = options.template.clone();

        let markup_open_pos = template.find("$1").ok_or_else(|| {
            crate::error::InversearchError::Encoder(crate::error::EncoderError::Encoding(
                "Invalid highlight template. The replacement pattern \"$1\" was not found"
                    .to_string(),
            ))
        })?;

        let markup_open = template[..markup_open_pos].to_string();
        let markup_close = template[markup_open_pos + 2..].to_string();

        let clip = options.clip.unwrap_or(true);
        let merge = if clip && !markup_open.is_empty() && !markup_close.is_empty() {
            Some(format!("{} {}", markup_close, markup_open))
        } else {
            None
        };

        let (ellipsis, ellipsis_markup_length) = if let Some(ellipsis_opts) = &options.ellipsis {
            let ellipsis_template = ellipsis_opts.template.clone();
            let ellipsis_markup_length = ellipsis_template.len() - 2;
            let ellipsis_pattern = ellipsis_opts.pattern.as_deref().unwrap_or("...");
            let ellipsis = ellipsis_template.replace("$1", ellipsis_pattern);
            (ellipsis, ellipsis_markup_length)
        } else {
            ("...".to_string(), 0)
        };

        Ok(HighlightConfig {
            template,
            markup_open,
            markup_close,
            boundary: options.boundary.clone(),
            clip,
            merge,
            ellipsis,
            ellipsis_markup_length,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub id: u64,
    pub doc: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EnrichedSearchResult {
    pub id: u64,
    pub doc: Option<serde_json::Value>,
    pub highlight: Option<String>,
}

pub type EnrichedSearchResults = Vec<EnrichedSearchResult>;

#[derive(Debug, Clone)]
pub struct FieldSearchResult {
    pub field: String,
    pub result: EnrichedSearchResults,
}

pub type FieldSearchResults = Vec<FieldSearchResult>;

#[derive(Debug, Clone, Default)]
pub struct EncoderCache {
    cache: HashMap<String, Vec<String>>,
}

impl EncoderCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_encode(&mut self, query: &str, encoder: &Encoder) -> Result<Vec<String>> {
        // Use a simple string representation of encoder config as key
        let key = "encoder_key".to_string(); // Simplified for now

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }

        let encoded = encoder.encode(query)?;
        self.cache.insert(key, encoded.clone());
        Ok(encoded)
    }
}

// ============================================================
// Added: Structured highlighting result type (Option A - Parallel Architecture)
// ============================================================

/// Structured information for individual matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMatch {
    /// Matching Raw Text
    pub text: String,
    /// Match start position (character level)
    pub start_pos: usize,
    /// End position of the match
    pub end_pos: usize,
    /// Matching query terms
    pub matched_query: String,
    /// Match Score (optional, for multi-match sorting)
    pub score: Option<f64>,
}

/// Individual field highlighting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldHighlight {
    /// field name
    pub field: String,
    /// All matches
    pub matches: Vec<HighlightMatch>,
    /// Full text after highlighting (optional, for direct front-end use)
    pub highlighted_text: Option<String>,
    /// List of matching query terms
    pub matched_queries: Vec<String>,
}

/// Individual document highlighting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHighlight {
    /// Document ID
    pub id: DocId,
    /// Highlighted results for each field
    pub fields: Vec<FieldHighlight>,
    /// Total matches
    pub total_matches: usize,
}

/// Search results (without highlighted base results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: DocId,
    pub score: Option<f64>,
    pub doc: Option<serde_json::Value>,
}

/// Full search results (with highlighting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultWithHighlight {
    pub results: Vec<SearchResult>,
    pub highlights: Vec<DocumentHighlight>,
    pub total: usize,
    pub query: String,
}
