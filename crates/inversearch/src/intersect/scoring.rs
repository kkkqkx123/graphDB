//! Scoring Module
//!
//! Implement advanced scoring algorithms, including TF-IDF, BM25, etc.

use crate::r#type::IntermediateSearchResults;
use std::collections::HashMap;

/// ID with rating
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredId {
    pub id: u64,
    pub score: f32,
    pub count: usize,
    pub positions: Vec<usize>,
}

/// Scoring algorithm trait
pub trait ScoringAlgorithm {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32;
    fn name(&self) -> &str;
}

/// Scoring configuration
#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub boost_factor: f32,
    pub min_score: f32,
    pub max_score: f32,
    pub field_weights: HashMap<String, f32>,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        ScoreConfig {
            boost_factor: 1.0,
            min_score: 0.0,
            max_score: 1.0,
            field_weights: HashMap::new(),
        }
    }
}

/// TF-IDF scoring algorithm
pub struct TfIdfScorer {
    document_frequency: HashMap<String, usize>,
    total_documents: usize,
}

impl TfIdfScorer {
    pub fn new(document_frequency: HashMap<String, usize>, total_documents: usize) -> Self {
        TfIdfScorer {
            document_frequency,
            total_documents,
        }
    }

    /// Calculate word frequency
    fn calculate_tf(&self, _term: &str, doc: &ScoredId) -> f32 {
        // Simplified implementation: using counts as word frequencies
        doc.count as f32
    }

    /// Calculate the inverse document frequency
    fn calculate_idf(&self, term: &str) -> f32 {
        let df = self.document_frequency.get(term).unwrap_or(&1);
        ((self.total_documents as f32) / (*df as f32)).ln().max(0.0)
    }
}

impl ScoringAlgorithm for TfIdfScorer {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32 {
        let mut score = 0.0;

        for term in query_terms {
            let tf = self.calculate_tf(term, doc);
            let idf = self.calculate_idf(term);
            score += tf * idf;
        }

        // Application Configuration
        score
            * config
                .boost_factor
                .max(config.min_score)
                .min(config.max_score)
    }

    fn name(&self) -> &str {
        "TF-IDF"
    }
}

/// BM25 scoring algorithm
pub struct Bm25Scorer {
    document_length: HashMap<u64, usize>,
    average_document_length: f32,
    k1: f32,
    b: f32,
}

impl Bm25Scorer {
    pub fn new(
        document_length: HashMap<u64, usize>,
        average_document_length: f32,
        k1: f32,
        b: f32,
    ) -> Self {
        Bm25Scorer {
            document_length,
            average_document_length,
            k1,
            b,
        }
    }

    /// Calculating BM25 scores
    fn calculate_bm25(&self, term_freq: usize, doc_id: u64, _doc_count: usize) -> f32 {
        let doc_len = *self.document_length.get(&doc_id).unwrap_or(&0) as f32;
        let normalized_length = doc_len / self.average_document_length;

        let numerator = term_freq as f32 * (self.k1 + 1.0);
        let denominator = term_freq as f32 + self.k1 * (1.0 - self.b + self.b * normalized_length);

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
}

impl ScoringAlgorithm for Bm25Scorer {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32 {
        let mut score = 0.0;

        for _term in query_terms {
            // Simplified implementation: assume one occurrence of each query term
            let term_freq = 1;
            let bm25_score = self.calculate_bm25(term_freq, doc.id, 0);
            score += bm25_score;
        }

        // Application Configuration
        score
            * config
                .boost_factor
                .max(config.min_score)
                .min(config.max_score)
    }

    fn name(&self) -> &str {
        "BM25"
    }
}

/// Rating Manager
pub struct ScoreManager {
    algorithms: HashMap<String, Box<dyn ScoringAlgorithm>>,
    default_algorithm: String,
}

impl Default for ScoreManager {
    fn default() -> Self {
        let mut manager = ScoreManager {
            algorithms: HashMap::new(),
            default_algorithm: "tfidf".to_string(),
        };

        // Add default algorithm
        manager.add_algorithm("tfidf", Box::new(TfIdfScorer::new(HashMap::new(), 1000)));
        manager.add_algorithm(
            "bm25",
            Box::new(Bm25Scorer::new(HashMap::new(), 100.0, 1.2, 0.75)),
        );

        manager
    }
}

impl ScoreManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_algorithm(&mut self, name: &str, algorithm: Box<dyn ScoringAlgorithm>) {
        self.algorithms.insert(name.to_string(), algorithm);
    }

    pub fn score_documents(
        &self,
        documents: Vec<ScoredId>,
        query_terms: &[String],
        algorithm: Option<&str>,
        config: &ScoreConfig,
    ) -> Vec<ScoredId> {
        let algorithm_name = algorithm.unwrap_or(&self.default_algorithm);

        if let Some(scorer) = self.algorithms.get(algorithm_name) {
            documents
                .into_iter()
                .map(|mut doc| {
                    doc.score = scorer.calculate_score(&doc, query_terms, config);
                    doc
                })
                .collect()
        } else {
            documents
        }
    }

    pub fn get_available_algorithms(&self) -> Vec<&str> {
        self.algorithms.keys().map(|s| s.as_str()).collect()
    }
}

/// Rate search results
pub fn score_search_results(
    results: &IntermediateSearchResults,
    query_terms: &[String],
    algorithm: Option<&str>,
    config: &ScoreConfig,
) -> Vec<ScoredId> {
    let mut scored_results = Vec::new();

    for result_array in results {
        for (pos, &id) in result_array.iter().enumerate() {
            let scored_id = ScoredId {
                id,
                score: 0.5, // Default Score
                count: result_array.len(),
                positions: vec![pos],
            };
            scored_results.push(scored_id);
        }
    }

    // Scoring with the Scoring Manager
    let manager = ScoreManager::new();
    manager.score_documents(scored_results, query_terms, algorithm, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_manager() {
        let manager = ScoreManager::new();
        let algorithms = manager.get_available_algorithms();
        assert!(algorithms.contains(&"tfidf"));
        assert!(algorithms.contains(&"bm25"));
    }

    #[test]
    fn test_tfidf_scorer() {
        let mut df = HashMap::new();
        df.insert("test".to_string(), 10);

        let scorer = TfIdfScorer::new(df, 100);
        let doc = ScoredId {
            id: 1,
            score: 0.0,
            count: 5,
            positions: vec![0, 1, 2],
        };

        let config = ScoreConfig::default();
        let score = scorer.calculate_score(&doc, &["test".to_string()], &config);
        assert!(score >= 0.0);
    }
}
