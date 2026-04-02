use serde::{Deserialize, Serialize};

/// BM25 algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    pub k1: f32,
    pub b: f32,
    pub avg_doc_length: f32,
    pub field_weights: FieldWeights,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Bm25Config {
            k1: 1.2,
            b: 0.75,
            avg_doc_length: 100.0,
            field_weights: FieldWeights::default(),
        }
    }
}

/// Field weights for search scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldWeights {
    pub title: f32,
    pub content: f32,
}

impl Default for FieldWeights {
    fn default() -> Self {
        FieldWeights {
            title: 2.0,
            content: 1.0,
        }
    }
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_highlight: bool,
    pub highlight_fragment_size: usize,
    pub enable_spell_check: bool,
    pub fuzzy_matching: bool,
    pub fuzzy_distance: u8,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            default_limit: 10,
            max_limit: 100,
            enable_highlight: true,
            highlight_fragment_size: 200,
            enable_spell_check: false,
            fuzzy_matching: false,
            fuzzy_distance: 2,
        }
    }
}
